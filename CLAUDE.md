# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Distillery (`dstl`) is a Rust CLI tool that transforms GitHub pull request diffs into structured, reviewable narratives using OpenAI's API. It presents an interactive TUI that orders changes by logical dependency, tags significance levels, and groups related changes into features.

## Build Commands

```bash
cargo build              # Development build
cargo build --release    # Release build
cargo run                # Run with default args
cargo run -- owner/repo#123  # Run with specific PR
cargo install --path .   # Install locally
```

## Testing and Linting

```bash
cargo test               # Run tests
cargo test -- --nocapture  # Run tests with output
cargo fmt                # Format code
cargo clippy             # Lint code
```

No custom configuration files exist for rustfmt or clippy; defaults are used.

## Prerequisites

- GitHub CLI (`gh`) must be installed and authenticated (`gh auth login`)
- `OPENAI_API_KEY` environment variable must be set

## Architecture

This codebase follows the **Model-View-Update (MVU)** pattern (Elm-like architecture) with unidirectional data flow:

```
Input Event → Action → Update (state mutation) → Command (async side effect) → Action
                            ↓
                        Render UI
```

### Key Directories

- **`src/main.rs`** - Entry point, CLI parsing, event loop orchestration
- **`src/app.rs`** - `App` state container and `AppState` enum (finite state machine)
- **`src/action.rs`** - `Action` enum: events that flow into the update function
- **`src/command.rs`** - `Command` enum: async operations (GitHub API, OpenAI, file I/O)
- **`src/config.rs`** - `AppConfig` struct for CLI options and environment

### Domain Layer (`src/domain/`)

- **`types.rs`** - Core data structures: `Story`, `Feature`, `DiffBlock`, `PrContext`, `ReviewAction`
- **`github.rs`** - GitHub CLI wrapper (`gh` subprocess calls for PR/repo fetching, review posting)
- **`llm.rs`** - OpenAI API integration with JSON Schema structured outputs
- **`prompt.rs`** - System and user prompt construction for LLM analysis

### UI Layer (`src/ui/`)

- **`layout.rs`** - Main render dispatcher based on app state
- **`components/`** - Modular ratatui components: `header`, `sidebar`, `document`, `picker`, `repo_selector`, `keybindings`, `loading`, `error`

### Update Layer (`src/update/`)

- **`mod.rs`** - Routes actions to appropriate handlers
- **`viewing.rs`** - Keyboard input handling in viewing mode
- **`editing.rs`** - Text input handling for review actions
- **`actions.rs`** - State transitions when async data loads
- **`picker.rs`**, **`repo.rs`** - Selection navigation logic

### Data Flow Example

1. User presses key → `Action::Input(KeyEvent)`
2. `update()` in `update/mod.rs` routes to handler
3. Handler mutates `App` state, returns `Option<Command>`
4. `execute_command()` runs async operation (e.g., `Command::FetchPr`)
5. Operation completes → returns `Action::PrLoaded(Result<PrContext>)`
6. `update()` processes result, transitions state
7. `render()` draws UI from current `App` state

## Core Types

```rust
// AI analysis output
Story { summary, focus, narrative: Vec<Feature>, ... }

// Grouped changes
Feature { title, why, changes, risks, diff_blocks: Vec<DiffBlock> }

// Individual diff with metadata
DiffBlock { label, role: DiffRole, significance: Significance, hunks }

// PR metadata from GitHub
PrContext { owner, repo, number, title, body, diff, author, ... }

// Possible states (FSM)
AppState { RepoSelector, PrPicker, LoadingPr, GeneratingStory, Viewing, EditingAction, Error, ... }
```

## Key Patterns

- **Finite State Machine**: `AppState` enum makes impossible states unrepresentable
- **Command Pattern**: State changes emit `Command`s executed asynchronously; results return as `Action`s
- **Component-Based UI**: Each ratatui component handles its own rendering with `Rect` constraints
- **Result<T, E> + anyhow**: Error handling with context propagation
