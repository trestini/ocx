# ocx

[![Alpha](https://img.shields.io/badge/status-alpha-red.svg)]()

CLI tool to manage OpenCode agent configuration.

## Overview

`ocx` helps you set up and manage OpenCode projects and their agents. It follows the same dispatcher pattern as `git` and `cargo` — each subcommand delegates to a dedicated binary (`ocx-new`, `ocx-agent`, etc.).

## Commands

| Command | Description |
|---|---|
| `ocx new` | Initialize a new OpenCode project in the current directory |
| `ocx new -a <agents>` | Initialize a project and add system agents |
| `ocx agent list` / `ocx agent ls` | List agents in the local project |
| `ocx agent list --system` | List all system-wide installed agents |
| `ocx agent add <name>` | Add a system agent to the current project |
| `ocx agent create <name>` | Create a new local agent with custom config |
| `ocx agent export <name>` | Export a local agent as markdown |
| `ocx agent remove <name>` / `ocx agent rm <name>` | Remove an agent from the project |

## Installation

```bash
cargo install --path .
```

## Project Structure

```
ocx/               # Workspace root
├── ocx/           # Dispatcher binary
├── ocx-new/       # Project initialization
├── ocx-agent/     # Agent management
└── ocx-common/    # Shared library
```
