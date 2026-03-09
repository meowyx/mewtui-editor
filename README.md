# mewtui

A three-pane terminal code editor built in Rust. Shell on the left, code editor in the middle, file tree on the right. Runs inside your existing terminal.


![Three-pane editor layout: shell, code editor, file tree](https://raw.githubusercontent.com/meowyx/mewtui-editor/main/image.png)

The shell is a real PTY. You can run `cargo build`, `git status`, or even `claude` inside it while your code and project files stay visible.

## Install

### Prerequisites

- **Rust** (1.85+) 
- **macOS or Linux** (PTY support required, Windows not yet supported)
- A terminal emulator (iTerm2, Terminal.app, Alacritty, Kitty, etc.)

### Install from crates.io

```bash
cargo install mewtui
```

This puts `mewtui` in `~/.cargo/bin/` (already in your PATH if you installed Rust via rustup). Then from any directory:

```bash
cd my-project
mewtui
```

The file tree automatically roots to wherever you launch from.

## Keybindings

### Global (work from any pane)

| Key | Action |
|---|---|
| `Shift+Tab` | Cycle focus: Shell → Editor → File Tree |
| `Ctrl+Q` | Quit |
| `Ctrl+T` | Cycle through themes |
| `Ctrl+S` | Save current file |

### Shell pane

When the shell is focused, all keys go directly to the PTY except the global bindings above. This means:

| Key | Action |
|---|---|
| `Tab` | Shell autocomplete (not intercepted) |
| `Ctrl+C` | Sends SIGINT to shell process |
| `Ctrl+D` | EOF / exit shell |
| Arrow keys | Shell history / cursor movement |
| Any typing | Goes to shell |

The shell is a real terminal. Run anything: `cargo`, `git`, `npm`, `python`, `claude`.

### Editor pane

| Key | Action |
|---|---|
| Arrow keys | Move cursor |
| `Home` / `End` | Jump to start / end of line |
| `Enter` | New line |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| Any character | Insert at cursor |

### Text selection (editor)

| Key | Action |
|---|---|
| `Ctrl+A` | Toggle mark: drop anchor at cursor, or clear it |
| Arrow keys | Move cursor to extend selection (shown as inverted colors) |
| `Ctrl+C` | Copy selection to system clipboard |
| `Ctrl+X` | Cut selection (copy + delete) |
| `Ctrl+V` | Paste from system clipboard |
| `Esc` | Clear selection |

Copied text goes to the system clipboard. Paste anywhere on your Mac with `Cmd+V`.

### File tree pane

| Key | Action |
|---|---|
| `Up` / `Down` | Navigate entries |
| `Enter` | Open file in editor / expand-collapse directory |

The file tree syncs with the shell's working directory. When you `cd` in the shell, the file tree updates automatically.

## Themes

20 built-in themes. Cycle through them with `Ctrl+T`.

### Modern dark (pretty ones first)

| Theme | Vibe |
|---|---|
| **dracula** | Purple tones. The classic dark theme. **(default)** |
| **catppuccin** | Pastel dark (Mocha variant). Soft and warm. |
| **tokyo-night** | Purple and blue. Calm Tokyo evening. |
| **rose-pine** | Muted rose and gold. Elegant and refined. |
| **synthwave** | Neon pink and cyan. Retro-future vibes. |
| **night-owl** | Rich blues and greens. Deep ocean feel. |
| **kanagawa** | Inspired by the Great Wave. Warm ink tones. |
| **ayu-dark** | Deep dark with vibrant orange accents. |
| **everforest** | Nature greens. Gentle on the eyes. |
| **one-dark** | Atom / VS Code One Dark. |
| **monokai** | Sublime Text classic. Pink keywords, green functions. |
| **github-dark** | GitHub's dark theme. Clean and modern. |
| **gruvbox** | Retro groove. Warm browns and oranges. |
| **nord** | Arctic blue. Calm and muted. |
| **solarized** | Ethan Schoonover's precision-crafted dark palette. |

### Retro

| Theme | Vibe |
|---|---|
| **phosphor** | Green on black. 90s hacker. The Matrix. |
| **amber** | Burnt orange on black. Old CRT warmth. |
| **cobalt** | Cyan on navy. Cool blue phosphor. |
| **terminal** | White on black. No personality, maximum readability. |
| **void** | Pure black, bright white, red cursor. Brutalist. |

## Syntax highlighting

Powered by tree-sitter (same engine as Neovim). Supported languages:

- Rust
- JavaScript / TypeScript
- Python
- JSON
- TOML
- Go

Highlighting is automatic based on file extension.

## Architecture

```
src/
├── main.rs               # Entry point, event loop, terminal setup
├── app.rs                # App state, focus management, key dispatch
├── ui.rs                 # Ratatui rendering, three-pane layout
├── theme.rs              # 20 theme definitions
├── editor/
│   ├── mod.rs
│   ├── buffer.rs         # Rope-backed text buffer, cursor, selection
│   └── highlight.rs      # Tree-sitter syntax highlighting
├── shell/
│   ├── mod.rs
│   └── pty.rs            # PTY spawn, VTE ANSI parser, terminal screen
└── filetree/
    └── mod.rs            # Directory traversal, expand/collapse
```

### Key design decisions

- **Rope data structure** (ropey) for the text buffer. Efficient for large files and frequent edits
- **VTE parser** for shell output. Handles ANSI colors, cursor movement, screen clearing, 256-color and 24-bit RGB
- **Tree-sitter** for syntax highlighting. Incremental parsing, same quality as Neovim
- **No async runtime.** Uses threads + channels for PTY I/O, keeping the architecture simple
- **Single AppState** as source of truth. Every frame redraws from state

## If something goes wrong

| Problem | Fix |
|---|---|
| App crashes, terminal looks garbled | Type `reset` and hit Enter (even if you can't see it) |
| Can't find the quit key | `Ctrl+Q` always works |
| `Ctrl+Q` doesn't work somehow | `Ctrl+C` from the editor or file tree pane also quits |
| Terminal completely frozen | Close the terminal tab/window, open a new one |

The app cannot damage your system. It just renders characters and manages a text buffer.

## Contributing

Contributions are welcome! Here's how to get started:

```bash
git clone https://github.com/meowyx/mewtui-editor
cd mewtui-editor
cargo run
```

1. **Fork** the repo and clone it locally
2. Create a **feature branch** (`git checkout -b my-feature`)
3. Make your changes and test them (`cargo run`)
4. Make sure it builds clean (`cargo build --release`)
5. Commit and **push** to your fork
6. Open a **pull request** with a clear description of what you changed and why

### Ideas for contributions

- New syntax highlighting languages (tree-sitter grammars)
- New themes
- Search and replace (`Ctrl+F`)
- Line numbers toggle
- Tab / indent support
- Mouse scroll support
- Linux testing and fixes
- Windows support (alternative to PTY)

### Guidelines

- Keep it simple. mewtui is meant to be lightweight and focused.
- No external runtime dependencies (no tokio, no async).
- Test your changes in at least one terminal emulator before opening a PR.
- If you're adding a theme, follow the existing `Theme` struct format in `src/theme.rs`.

### Reporting bugs

Open an [issue](https://github.com/meowyx/mewtui-editor/issues) with:
- What you expected to happen
- What actually happened
- Your OS, terminal emulator, and Rust version (`rustc --version`)

## License

MIT
