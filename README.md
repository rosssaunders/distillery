# Distillery

**Distill PR diffs into reviewable narratives.** Built for the AI code review era.

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

Large PRs are overwhelming. AI-generated code makes it worse—hundreds of lines where the actual change is buried in boilerplate. Distillery uses AI to transform raw diffs into structured narratives that tell you **what matters** and **what you can skim**.

## Features

- **Focus Section** — Instantly see THE key change and where to look
- **Significance Tags** — Each diff block marked as `★ KEY`, standard, or `· noise`
- **Dependency-Ordered** — Changes presented root-first, not alphabetically
- **Review Actions** — Generate "Request Changes", clarification questions, or follow-up issues directly
- **Progress Tracking** — Mark diffs as reviewed, track completion
- **Keyboard-Driven** — Full vim-style navigation

## Demo

```
┌─────────────────────────────────────────────────────────────────┐
│ Distillery │ owner/repo#123                                     │
│ Add rate limiting to API endpoints                              │
├──────────────────┬──────────────────────────────────────────────┤
│ PROGRESS 3/12    │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│                  │ ⚡ FOCUS: Adds rate limiting middleware       │
│ ▶ Rate Limiting  │ 👁 Review: src/middleware/rate_limit.rs      │
│   3/5 diffs      │ ⏭ Skim: Import changes in 4 files            │
│   → ★ middleware │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│     · imports    │                                              │
│   ✓ config       │ FEATURE 1: Rate Limiting                     │
│                  │ Prevents API abuse by limiting requests...   │
│   Auth Changes   │                                              │
│   2/3 diffs      │ ┌─ ★ KEY rate_limit.rs [root]                │
│                  │ │ WHY: Core rate limiting logic              │
├──────────────────┴──────────────────────────────────────────────┤
│ j/k Scroll │ h/l Diff │ n/p Feature │ v Viewed │ o PRs │ q Quit │
└─────────────────────────────────────────────────────────────────┘
```

## Installation

### From source

```bash
git clone https://github.com/rosssaunders/distillery
cd dstl
cargo install --path .
```

### Prerequisites

- [GitHub CLI](https://cli.github.com/) (`gh`) — authenticated
- OpenAI API key (set `OPENAI_API_KEY` environment variable)

```bash
# Authenticate GitHub CLI
gh auth login

# Set OpenAI API key
export OPENAI_API_KEY=sk-...
```

## Usage

```bash
# Start with repo selector (browse your repos)
dstl

# Start with PR picker for a specific repo
dstl owner/repo

# Load a specific PR directly
dstl owner/repo#123

# Or use a GitHub URL
dstl https://github.com/owner/repo/pull/123
```

### Options

```
Options:
  -R, --repo <REPO>        Repo for PR picker (owner/repo format)
  -m, --model <MODEL>      OpenAI model to use [default: gpt-4.1]
      --cache              Use cached response (skip LLM call)
      --cache-file <FILE>  Path to cache file [default: .dstl-cache.json]
  -h, --help               Print help
```

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll down / up |
| `Space` / `b` | Page down / up |
| `h` / `l` | Previous / next diff block |
| `n` / `p` | Next / previous feature |
| `v` | Mark current diff as viewed |

### Actions

| Key | Action |
|-----|--------|
| `1` | Select "Request Changes" action |
| `2` | Select "Clarification Questions" action |
| `3` | Select "Next PR" (follow-up issue) action |
| `Enter` | Edit selected action text |
| `Ctrl+S` | Submit action to GitHub |
| `Esc` | Exit edit mode |

### Navigation

| Key | Action |
|-----|--------|
| `o` | Open PR picker (current repo) |
| `O` | Open repo selector |
| `r` | Refresh current list |
| `q` | Quit |

## How It Works

1. **Fetches** PR metadata and diff via GitHub CLI
2. **Analyzes** with OpenAI to identify:
   - Logical groupings (features/concerns)
   - Dependency order (root changes first)
   - Significance (key vs noise)
   - Risks and test suggestions
3. **Renders** an interactive TUI for efficient review

## Configuration

Create a `.env` file in your working directory:

```env
OPENAI_API_KEY=sk-your-key-here
```

## Why "Distillery"?

Like a distillery extracts the essence from raw ingredients, this tool extracts the essence from raw diffs—separating the key changes from the noise, leaving you with something refined and reviewable.

## License

MIT

## Contributing

Contributions welcome! Please open an issue first to discuss what you'd like to change.
