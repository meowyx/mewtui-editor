use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::{Buffer, SyntaxHighlighter};
use crate::editor::highlight::ColorSpan;
use crate::filetree::FileTree;
use crate::shell::ShellPane;
use crate::theme::{ALL_THEMES, Theme};

#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    Shell,
    Editor,
    FileTree,
}

pub struct App {
    pub focus: Focus,
    pub should_quit: bool,
    pub theme_index: usize,
    pub file_tree: FileTree,
    pub buffer: Buffer,
    pub status_msg: Option<String>,
    pub highlighter: SyntaxHighlighter,
    pub highlight_spans: Vec<ColorSpan>,
    highlight_dirty: bool,
    pub shell: Option<ShellPane>,
}

impl App {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        App {
            focus: Focus::Shell,
            should_quit: false,
            theme_index: 0,
            file_tree: FileTree::new(cwd),
            buffer: Buffer::new(),
            status_msg: None,
            highlighter: SyntaxHighlighter::new(),
            highlight_spans: Vec::new(),
            highlight_dirty: true,
            shell: None,
        }
    }

    /// Initialize the shell pane with the given size.
    pub fn init_shell(&mut self, rows: u16, cols: u16) {
        match ShellPane::new(rows, cols) {
            Ok(shell) => {
                self.shell = Some(shell);
            }
            Err(e) => {
                self.status_msg = Some(format!("Shell error: {e}"));
            }
        }
    }

    pub fn current_theme(&self) -> &'static Theme {
        &ALL_THEMES[self.theme_index]
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Shell => Focus::Editor,
            Focus::Editor => Focus::FileTree,
            Focus::FileTree => Focus::Shell,
        };
    }

    pub fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % ALL_THEMES.len();
        self.highlight_dirty = true;
    }

    /// Sync file tree with shell's cwd if it changed (via OSC 7).
    pub fn sync_shell_cwd(&mut self) {
        if let Some(shell) = &mut self.shell {
            if let Some(new_cwd) = shell.take_cwd() {
                // Canonicalize to prevent path traversal via symlinks
                let canonical = new_cwd.canonicalize().unwrap_or(new_cwd);
                if canonical != self.file_tree.root && canonical.is_dir() {
                    self.file_tree = FileTree::new(canonical);
                }
            }
        }
    }

    /// Recompute syntax highlights if needed (call before rendering).
    pub fn update_highlights(&mut self) {
        if !self.highlight_dirty {
            return;
        }
        self.highlight_dirty = false;

        let ext = self
            .buffer
            .file_path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if ext.is_empty() {
            self.highlight_spans.clear();
            return;
        }

        let source = self.buffer.rope.to_string();
        let theme = self.current_theme();
        self.highlight_spans = self.highlighter.highlight(&source, ext, theme);
    }

    fn open_selected_file(&mut self) {
        let Some(entry) = self.file_tree.selected_entry() else {
            return;
        };

        if entry.is_dir {
            self.file_tree.toggle_expand();
            return;
        }

        let path = entry.path.clone();
        match self.buffer.open_file(path) {
            Ok(()) => {
                self.focus = Focus::Editor;
                self.highlight_dirty = true;
                let name = self.buffer.file_path.as_ref()
                    .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                    .unwrap_or_default();
                self.status_msg = Some(format!("Opened {name}"));
            }
            Err(e) => {
                self.status_msg = Some(format!("Error: {e}"));
            }
        }
    }

    // --- Clipboard operations ---

    fn copy_selection(&mut self) {
        if let Some(text) = self.buffer.selected_text() {
            match Clipboard::new().and_then(|mut cb| cb.set_text(&text)) {
                Ok(()) => {
                    let len = text.len();
                    self.status_msg = Some(format!("Copied {len} chars"));
                }
                Err(e) => {
                    self.status_msg = Some(format!("Clipboard error: {e}"));
                }
            }
        }
    }

    fn cut_selection(&mut self) {
        if let Some(text) = self.buffer.delete_selection() {
            self.highlight_dirty = true;
            match Clipboard::new().and_then(|mut cb| cb.set_text(&text)) {
                Ok(()) => {
                    let len = text.len();
                    self.status_msg = Some(format!("Cut {len} chars"));
                }
                Err(e) => {
                    self.status_msg = Some(format!("Clipboard error: {e}"));
                }
            }
        }
    }

    fn paste(&mut self) {
        match Clipboard::new().and_then(|mut cb| cb.get_text()) {
            Ok(text) => {
                // If there's an active selection, delete it first
                if self.buffer.anchor.is_some() {
                    self.buffer.delete_selection();
                }
                self.buffer.insert_str(&text);
                self.highlight_dirty = true;
                self.status_msg = Some(format!("Pasted {} chars", text.len()));
            }
            Err(e) => {
                self.status_msg = Some(format!("Clipboard error: {e}"));
            }
        }
    }

    /// Forward a key event to the shell PTY as raw bytes.
    fn forward_key_to_shell(&mut self, key: KeyEvent) {
        let Some(shell) = &mut self.shell else {
            return;
        };

        let bytes: Vec<u8> = match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    let ctrl = (c as u8).wrapping_sub(b'a').wrapping_add(1);
                    vec![ctrl]
                } else {
                    let mut buf = [0u8; 4];
                    let s = c.encode_utf8(&mut buf);
                    s.as_bytes().to_vec()
                }
            }
            KeyCode::Enter => vec![b'\r'],
            KeyCode::Backspace => vec![0x7f],
            KeyCode::Tab => vec![b'\t'],
            KeyCode::Esc => vec![0x1b],
            KeyCode::Up => b"\x1b[A".to_vec(),
            KeyCode::Down => b"\x1b[B".to_vec(),
            KeyCode::Right => b"\x1b[C".to_vec(),
            KeyCode::Left => b"\x1b[D".to_vec(),
            KeyCode::Home => b"\x1b[H".to_vec(),
            KeyCode::End => b"\x1b[F".to_vec(),
            KeyCode::Delete => b"\x1b[3~".to_vec(),
            KeyCode::PageUp => b"\x1b[5~".to_vec(),
            KeyCode::PageDown => b"\x1b[6~".to_vec(),
            _ => return,
        };

        shell.write(&bytes);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Global keybinds — always intercepted regardless of focus
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                self.should_quit = true;
                return;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
                self.cycle_theme();
                return;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                match self.buffer.save() {
                    Ok(()) => {
                        let name = self.buffer.file_path.as_ref()
                            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                            .unwrap_or_else(|| "untitled".into());
                        self.status_msg = Some(format!("Saved {name}"));
                    }
                    Err(e) => {
                        self.status_msg = Some(format!("Save error: {e}"));
                    }
                }
                return;
            }
            _ => {}
        }

        // Ctrl+Tab / Shift+Tab cycles focus from any pane
        if key.code == KeyCode::BackTab || (key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::CONTROL)) {
            self.cycle_focus();
            return;
        }

        // Shell: forward everything except the above to PTY
        if self.focus == Focus::Shell {
            self.forward_key_to_shell(key);
            return;
        }

        // --- Editor-specific selection keybinds ---
        if self.focus == Focus::Editor {
            match (key.modifiers, key.code) {
                // Ctrl+A: toggle mark (start/stop selection)
                (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                    self.buffer.toggle_mark();
                    if self.buffer.anchor.is_some() {
                        self.status_msg = Some("Mark set — move cursor to select".to_string());
                    } else {
                        self.status_msg = Some("Mark cleared".to_string());
                    }
                    return;
                }
                // Ctrl+C: copy selection (or no-op if no selection)
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                    if self.buffer.anchor.is_some() {
                        self.copy_selection();
                    }
                    return;
                }
                // Ctrl+X: cut selection
                (KeyModifiers::CONTROL, KeyCode::Char('x')) => {
                    if self.buffer.anchor.is_some() {
                        self.cut_selection();
                    }
                    return;
                }
                // Ctrl+V: paste
                (KeyModifiers::CONTROL, KeyCode::Char('v')) => {
                    self.paste();
                    return;
                }
                // Escape: clear selection
                (_, KeyCode::Esc) => {
                    if self.buffer.anchor.is_some() {
                        self.buffer.clear_selection();
                        self.status_msg = Some("Selection cleared".to_string());
                    }
                    return;
                }
                _ => {}
            }
        }

        // Ctrl+C from filetree: quit
        if self.focus == Focus::FileTree && key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        // Clear status message on any non-global key
        self.status_msg = None;

        // Pane-specific keybinds
        match self.focus {
            Focus::FileTree => match key.code {
                KeyCode::Up => self.file_tree.move_up(),
                KeyCode::Down => self.file_tree.move_down(),
                KeyCode::Enter => self.open_selected_file(),
                _ => {}
            },
            Focus::Editor => match key.code {
                KeyCode::Up => self.buffer.move_up(),
                KeyCode::Down => self.buffer.move_down(),
                KeyCode::Left => self.buffer.move_left(),
                KeyCode::Right => self.buffer.move_right(),
                KeyCode::Home => self.buffer.move_home(),
                KeyCode::End => self.buffer.move_end(),
                KeyCode::Enter => { self.buffer.insert_newline(); self.highlight_dirty = true; }
                KeyCode::Backspace => { self.buffer.backspace(); self.highlight_dirty = true; }
                KeyCode::Delete => { self.buffer.delete(); self.highlight_dirty = true; }
                KeyCode::Char(c) => { self.buffer.insert_char(c); self.highlight_dirty = true; }
                _ => {}
            },
            Focus::Shell => unreachable!(),
        }
    }
}
