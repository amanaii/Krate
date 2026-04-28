use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectAction {
    Confirm,
    Install,
    Search,
    Back,
    Quit,
}

#[derive(Debug, Clone)]
pub struct SelectResult {
    pub indices: Vec<usize>,
    pub action: SelectAction,
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn select_many_marked(
        &mut self,
        title: &str,
        items: &[String],
        selected: &[bool],
    ) -> Result<SelectResult> {
        self.run_select(
            title,
            items,
            SelectMode::Many,
            None,
            Some(selected),
            "Up/Down move  Enter/Space configure  / search  i install  Esc quit",
        )
    }

    pub fn input(&mut self, title: &str, initial: &str) -> Result<Option<String>> {
        let mut value = initial.to_string();

        loop {
            self.terminal.draw(|frame| {
                let area = frame.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(5),
                        Constraint::Min(1),
                        Constraint::Length(2),
                    ])
                    .split(area);

                render_header(frame, chunks[0], title);
                frame.render_widget(
                    Paragraph::new(value.as_str())
                        .block(Block::default().title("Search").borders(Borders::ALL)),
                    chunks[1],
                );
                frame.render_widget(Paragraph::new("Enter search  Esc cancel"), chunks[3]);
            })?;

            if !event::poll(Duration::from_millis(200))? {
                continue;
            }
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Enter => {
                    let trimmed = value.trim();
                    if !trimmed.is_empty() {
                        return Ok(Some(trimmed.to_string()));
                    }
                }
                KeyCode::Esc => return Ok(None),
                KeyCode::Backspace => {
                    value.pop();
                }
                KeyCode::Char(ch) => value.push(ch),
                _ => {}
            }
        }
    }

    pub fn select_one(&mut self, title: &str, items: &[String]) -> Result<SelectResult> {
        self.run_select(
            title,
            items,
            SelectMode::One,
            None,
            None,
            "Up/Down move  Enter select  Esc back",
        )
    }

    pub fn select_one_sticky(
        &mut self,
        title: &str,
        items: &[String],
        selected_index: Option<usize>,
    ) -> Result<SelectResult> {
        self.run_select(
            title,
            items,
            SelectMode::StickyOne,
            selected_index,
            None,
            "Up/Down move  Space/Enter save  Esc/q done",
        )
    }

    pub fn message(&mut self, title: &str, lines: &[String]) -> Result<()> {
        self.status(title, lines, "Enter/Esc/q close")?;

        loop {
            if !event::poll(Duration::from_millis(200))? {
                continue;
            }
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }

    pub fn status(&mut self, title: &str, lines: &[String], help: &str) -> Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(2),
                ])
                .split(area);

            render_header(frame, chunks[0], title);
            let body = lines.join("\n");
            frame.render_widget(
                Paragraph::new(body)
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: true }),
                chunks[1],
            );
            frame.render_widget(Paragraph::new(help), chunks[2]);
        })?;
        Ok(())
    }

    pub fn progress(
        &mut self,
        title: &str,
        lines: &[String],
        ratio: Option<f64>,
        label: &str,
        help: &str,
    ) -> Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(3),
                    Constraint::Length(2),
                ])
                .split(area);

            render_header(frame, chunks[0], title);
            let body = lines.join("\n");
            frame.render_widget(
                Paragraph::new(body)
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: true }),
                chunks[1],
            );

            let gauge = Gauge::default()
                .block(Block::default().title("Progress").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
                .label(label.to_string())
                .ratio(ratio.unwrap_or(0.0).clamp(0.0, 1.0));
            frame.render_widget(gauge, chunks[2]);
            frame.render_widget(Paragraph::new(help), chunks[3]);
        })?;
        Ok(())
    }

    fn run_select(
        &mut self,
        title: &str,
        items: &[String],
        mode: SelectMode,
        selected_index: Option<usize>,
        selected_items: Option<&[bool]>,
        help: &str,
    ) -> Result<SelectResult> {
        if items.is_empty() {
            return Ok(SelectResult {
                indices: Vec::new(),
                action: SelectAction::Back,
            });
        }

        let mut state = ListState::default();
        let initial = selected_index.unwrap_or(0).min(items.len() - 1);
        state.select(Some(initial));
        let mut selected = selected_items
            .map(|items| items.to_vec())
            .unwrap_or_else(|| vec![false; items.len()]);
        selected.resize(items.len(), false);
        if mode == SelectMode::StickyOne {
            selected[initial] = true;
        }

        loop {
            self.draw_select(&mut state, title, items, &selected, help)?;

            if !event::poll(Duration::from_millis(200))? {
                continue;
            }

            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Up => {
                    let current = state.selected().unwrap_or(0);
                    state.select(Some(current.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let current = state.selected().unwrap_or(0);
                    state.select(Some((current + 1).min(items.len() - 1)));
                }
                KeyCode::Char(' ') if mode == SelectMode::Many => {
                    return Ok(SelectResult {
                        indices: state.selected().into_iter().collect(),
                        action: SelectAction::Confirm,
                    });
                }
                KeyCode::Enter => match mode {
                    SelectMode::One => {
                        return Ok(SelectResult {
                            indices: state.selected().into_iter().collect(),
                            action: SelectAction::Confirm,
                        });
                    }
                    SelectMode::Many => {
                        return Ok(SelectResult {
                            indices: state.selected().into_iter().collect(),
                            action: SelectAction::Confirm,
                        });
                    }
                    SelectMode::StickyOne => {
                        if let Some(index) = state.selected() {
                            selected.fill(false);
                            selected[index] = true;
                        }
                    }
                },
                KeyCode::Char(' ') if mode == SelectMode::StickyOne => {
                    if let Some(index) = state.selected() {
                        selected.fill(false);
                        selected[index] = true;
                    }
                }
                KeyCode::Esc => {
                    return Ok(SelectResult {
                        indices: selected_indices(&selected),
                        action: SelectAction::Back,
                    });
                }
                KeyCode::Char('/') if mode == SelectMode::Many => {
                    return Ok(SelectResult {
                        indices: selected_indices(&selected),
                        action: SelectAction::Search,
                    });
                }
                KeyCode::Char('i') if mode == SelectMode::Many => {
                    return Ok(SelectResult {
                        indices: selected_indices(&selected),
                        action: SelectAction::Install,
                    });
                }
                KeyCode::Char('q') => {
                    return Ok(SelectResult {
                        indices: selected_indices(&selected),
                        action: SelectAction::Quit,
                    });
                }
                _ => {}
            }
        }
    }

    fn draw_select(
        &mut self,
        state: &mut ListState,
        title: &str,
        items: &[String],
        selected: &[bool],
        help: &str,
    ) -> Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(2),
                ])
                .split(area);

            render_header(frame, chunks[0], title);

            let list_items: Vec<ListItem> = items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let marker = if selected[index] { "[x]" } else { "[ ]" };
                    ListItem::new(format!("{marker} {item}"))
                })
                .collect();
            let list = List::new(list_items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");
            frame.render_stateful_widget(list, chunks[1], state);
            frame.render_widget(Paragraph::new(help), chunks[2]);
        })?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectMode {
    One,
    Many,
    StickyOne,
}

fn selected_indices(selected: &[bool]) -> Vec<usize> {
    selected
        .iter()
        .enumerate()
        .filter_map(|(index, value)| value.then_some(index))
        .collect()
}

fn render_header(frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, title: &str) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "Krate",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::raw(title),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}
