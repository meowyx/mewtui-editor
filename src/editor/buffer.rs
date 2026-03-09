use std::fs;
use std::path::PathBuf;

use ropey::Rope;

pub struct Buffer {
    pub rope: Rope,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub file_path: Option<PathBuf>,
    pub modified: bool,
    pub anchor: Option<(usize, usize)>, // (row, col) — selection start
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            rope: Rope::new(),
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            file_path: None,
            modified: false,
            anchor: None,
        }
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<(), std::io::Error> {
        let contents = fs::read_to_string(&path)?;
        self.rope = Rope::from_str(&contents);
        self.file_path = Some(path);
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
        self.modified = false;
        self.anchor = None;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if let Some(path) = &self.file_path {
            fs::write(path, self.rope.to_string())?;
            self.modified = false;
        }
        Ok(())
    }

    // --- Selection ---

    /// Toggle mark: set anchor at cursor, or clear if already set.
    pub fn toggle_mark(&mut self) {
        if self.anchor.is_some() {
            self.anchor = None;
        } else {
            self.anchor = Some((self.cursor_row, self.cursor_col));
        }
    }

    /// Clear selection.
    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    /// Returns (start, end) positions ordered, where each is (row, col).
    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.anchor?;
        let cursor = (self.cursor_row, self.cursor_col);
        if anchor <= cursor {
            Some((anchor, cursor))
        } else {
            Some((cursor, anchor))
        }
    }

    /// Returns true if (row, col) is inside the current selection.
    pub fn is_selected(&self, row: usize, col: usize) -> bool {
        let Some((start, end)) = self.selection_range() else {
            return false;
        };
        let pos = (row, col);
        pos >= start && pos < end
    }

    /// Extract selected text as a String.
    pub fn selected_text(&self) -> Option<String> {
        let ((sr, sc), (er, ec)) = self.selection_range()?;
        let start_char = self.rope.line_to_char(sr) + sc;
        let end_char = self.rope.line_to_char(er) + ec;
        if start_char >= end_char || end_char > self.rope.len_chars() {
            return None;
        }
        Some(self.rope.slice(start_char..end_char).to_string())
    }

    /// Delete the selected range and return the deleted text.
    pub fn delete_selection(&mut self) -> Option<String> {
        let text = self.selected_text()?;
        let ((sr, sc), (er, ec)) = self.selection_range()?;
        let start_char = self.rope.line_to_char(sr) + sc;
        let end_char = self.rope.line_to_char(er) + ec;
        if start_char < end_char && end_char <= self.rope.len_chars() {
            self.rope.remove(start_char..end_char);
            self.cursor_row = sr;
            self.cursor_col = sc;
            self.modified = true;
        }
        self.anchor = None;
        Some(text)
    }

    /// Insert a string at cursor position.
    pub fn insert_str(&mut self, s: &str) {
        let line_idx = self.cursor_row.min(self.rope.len_lines().saturating_sub(1));
        let line_start = self.rope.line_to_char(line_idx);
        let line_len = self.line_len(line_idx);
        let col = self.cursor_col.min(line_len);
        let char_idx = line_start + col;

        self.rope.insert(char_idx, s);

        // Move cursor to end of inserted text
        let inserted_chars = s.chars().count();
        let new_pos = char_idx + inserted_chars;
        if new_pos <= self.rope.len_chars() {
            let new_line = self.rope.char_to_line(new_pos);
            let new_col = new_pos - self.rope.line_to_char(new_line);
            self.cursor_row = new_line;
            self.cursor_col = new_col;
        }
        self.modified = true;
    }

    // --- Basic editing ---

    pub fn insert_char(&mut self, c: char) {
        let line_idx = self.cursor_row.min(self.rope.len_lines().saturating_sub(1));
        let line_start = self.rope.line_to_char(line_idx);
        let line_len = self.line_len(line_idx);
        let col = self.cursor_col.min(line_len);
        let char_idx = line_start + col;

        self.rope.insert_char(char_idx, c);
        self.cursor_col = col + 1;
        self.modified = true;
    }

    pub fn insert_newline(&mut self) {
        let line_idx = self.cursor_row.min(self.rope.len_lines().saturating_sub(1));
        let line_start = self.rope.line_to_char(line_idx);
        let line_len = self.line_len(line_idx);
        let col = self.cursor_col.min(line_len);
        let char_idx = line_start + col;

        self.rope.insert_char(char_idx, '\n');
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.modified = true;
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line_start = self.rope.line_to_char(self.cursor_row);
            let char_idx = line_start + self.cursor_col - 1;
            self.rope.remove(char_idx..char_idx + 1);
            self.cursor_col -= 1;
            self.modified = true;
        } else if self.cursor_row > 0 {
            let prev_line_len = self.line_len(self.cursor_row - 1);
            let line_start = self.rope.line_to_char(self.cursor_row);
            self.rope.remove(line_start - 1..line_start);
            self.cursor_row -= 1;
            self.cursor_col = prev_line_len;
            self.modified = true;
        }
    }

    pub fn delete(&mut self) {
        let line_idx = self.cursor_row;
        let line_start = self.rope.line_to_char(line_idx);
        let line_len = self.line_len(line_idx);
        let col = self.cursor_col.min(line_len);
        let char_idx = line_start + col;

        if char_idx < self.rope.len_chars() {
            self.rope.remove(char_idx..char_idx + 1);
            self.modified = true;
        }
    }

    // --- Cursor movement ---

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_cursor_col();
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_row < self.rope.len_lines().saturating_sub(1) {
            self.cursor_row += 1;
            self.clamp_cursor_col();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.line_len(self.cursor_row);
        }
    }

    pub fn move_right(&mut self) {
        let line_len = self.line_len(self.cursor_row);
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_row < self.rope.len_lines().saturating_sub(1) {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor_col = self.line_len(self.cursor_row);
    }

    /// Returns the length of a line excluding the trailing newline.
    pub fn line_len(&self, line_idx: usize) -> usize {
        if line_idx >= self.rope.len_lines() {
            return 0;
        }
        let line = self.rope.line(line_idx);
        let len = line.len_chars();
        if len > 0 && line.char(len - 1) == '\n' {
            len - 1
        } else {
            len
        }
    }

    fn clamp_cursor_col(&mut self) {
        let line_len = self.line_len(self.cursor_row);
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    /// Adjusts scroll_offset so the cursor is visible within pane_height lines.
    pub fn ensure_cursor_visible(&mut self, pane_height: usize) {
        if pane_height == 0 {
            return;
        }
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        } else if self.cursor_row >= self.scroll_offset + pane_height {
            self.scroll_offset = self.cursor_row - pane_height + 1;
        }
    }
}
