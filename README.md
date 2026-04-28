<div align="center">
![](https://github.com/amanaii/Krate/blob/main/assets/krate.png)

  # Krate

Fast terminal UI downloader and updater for Minecraft server plugins.

![Rust](https://img.shields.io/badge/Rust-2024-f74c00?style=for-the-badge&logo=rust&logoColor=white)
![Ratatui](https://img.shields.io/badge/TUI-Ratatui-00a884?style=for-the-badge)
![Tokio](https://img.shields.io/badge/Runtime-Tokio-2f81f7?style=for-the-badge)
![License](https://img.shields.io/badge/License-MIT-lightgrey?style=for-the-badge)

Search, pick, download, and update Minecraft server plugins from a clean TUI.

</div>

## Overview

Krate is a small Rust CLI for managing Minecraft server plugin jars directly from your terminal. It searches plugin providers, lets you choose versions and loaders in a TUI, downloads selected plugin jars into the current directory, and records installed plugins in a local `.krate.json` manifest for later updates.

## Features

- Terminal UI built with Ratatui and Crossterm.
- Search and install one or more plugins at once.
- Update installed plugins from the local `.krate.json` manifest.
- Provider selection between Modrinth and CurseForge.
- Download progress with bordered progress bars.
- Version and loader selection for Minecraft targets.
- Safe filename handling for downloaded jars.

## Providers

| Provider | Default | Notes |
| --- | --- | --- |
| Modrinth | Yes | Works without extra setup. |
| CurseForge | No | Requires `CURSEFORGE_API_KEY` in your environment. |

Supported loader choices shown by Krate include:

```text
paper, purpur, spigot, bukkit, folia, fabric, forge, neoforge
```

## Requirements

- Rust toolchain with edition 2024 support.
- Network access to the selected provider API.
- For CurseForge only: a `CURSEFORGE_API_KEY` environment variable.

## Build

Clone the project, then build with Cargo:

```bash
cargo build --release
```

The optimized binary is created at:

```bash
target/release/krate
```

Run tests:

```bash
cargo test
```

Run from source during development:

```bash
cargo run -- --help
```

## Usage

Show help:

```bash
krate --help
```

Search and install plugins:

```bash
krate get luckperms
```

Krate opens a TUI where you can:

- Move with Up and Down.
- Press Space or Enter to configure a plugin.
- Press `/` to search again.
- Press `i` to install selected plugins.
- Press Esc or `q` to leave.

Choose provider server:

```bash
krate server
```

Update installed plugins:

```bash
krate update
```

Update to a specific Minecraft version and loader:

```bash
krate update --version 1.21.11 --loader paper
```

Short flags also work:

```bash
krate update -v 1.21.11 -l paper
```

## Workflow

1. Choose a provider with `krate server` if you do not want the default Modrinth provider.
2. Run `krate get <query>` inside your Minecraft server plugin directory.
3. Select plugin versions and loaders in the TUI.
4. Krate downloads jars into the current directory.
5. Krate writes `.krate.json` so future `krate update` runs know what was installed.

## Local Files

Krate uses two main local files:

| File | Purpose |
| --- | --- |
| `.krate.json` | Per-directory install manifest for downloaded plugins. |
| `config.json` | User config stored under your OS config directory at `krate/config.json`. |

The config currently stores the selected provider.

Example config:

```json
{
  "provider": "modrinth"
}
```

## CurseForge Setup

CurseForge API access requires an API key:

```bash
export CURSEFORGE_API_KEY="your-api-key"
krate server
krate get essentials
```

Select `CurseForge` in the provider TUI after setting the variable.

## Commands

| Command | Description |
| --- | --- |
| `krate get <query>` | Search and download one or more plugins. |
| `krate server` | Choose the plugin provider server. |
| `krate update` | Update installed plugins in the current directory. |
| `krate update -v <version> -l <loader>` | Update directly to a target Minecraft version and loader. |

## Development

Format code:

```bash
cargo fmt
```

Run checks:

```bash
cargo test
```

Useful source layout:

```text
src/cli.rs                 CLI arguments and commands
src/commands/get.rs        Search, selection, and install flow
src/commands/update.rs     Manifest-based update flow
src/commands/server.rs     Provider selection
src/downloader.rs          Download and manifest recording
src/providers/             Modrinth and CurseForge API clients
src/tui/mod.rs             Terminal UI primitives
```

## Status

Krate is early-stage software. Expect the command surface and manifest format to evolve.
