use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use ratatui::style::Color;
use vte::{Params, Perform};

/// A single cell in the terminal screen buffer.
#[derive(Clone, Copy)]
pub struct Cell {
    pub c: char,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            c: ' ',
            fg: None,
            bg: None,
            bold: false,
        }
    }
}

/// Terminal screen buffer that the VTE parser writes into.
pub struct TerminalScreen {
    pub cells: Vec<Vec<Cell>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub rows: usize,
    pub cols: usize,
    // Current SGR attributes
    cur_fg: Option<Color>,
    cur_bg: Option<Color>,
    cur_bold: bool,
    // Scroll region
    scroll_top: usize,
    scroll_bottom: usize,
    // CWD reported by shell via OSC 7
    pub reported_cwd: Option<std::path::PathBuf>,
}

impl TerminalScreen {
    pub fn new(rows: usize, cols: usize) -> Self {
        let cells = vec![vec![Cell::default(); cols]; rows];
        TerminalScreen {
            cells,
            cursor_row: 0,
            cursor_col: 0,
            rows,
            cols,
            cur_fg: None,
            cur_bg: None,
            cur_bold: false,
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
            reported_cwd: None,
        }
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.rows = rows;
        self.cols = cols;
        self.cells.resize(rows, vec![Cell::default(); cols]);
        for row in &mut self.cells {
            row.resize(cols, Cell::default());
        }
        if self.cursor_row >= rows {
            self.cursor_row = rows.saturating_sub(1);
        }
        if self.cursor_col >= cols {
            self.cursor_col = cols.saturating_sub(1);
        }
        self.scroll_bottom = rows.saturating_sub(1);
    }

    fn scroll_up(&mut self) {
        if self.scroll_top < self.scroll_bottom && self.scroll_bottom < self.rows {
            self.cells.remove(self.scroll_top);
            self.cells
                .insert(self.scroll_bottom, vec![Cell::default(); self.cols]);
        }
    }

    fn put_char(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            // Line wrap
            self.cursor_col = 0;
            self.cursor_row += 1;
            if self.cursor_row > self.scroll_bottom {
                self.cursor_row = self.scroll_bottom;
                self.scroll_up();
            }
        }
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.cells[self.cursor_row][self.cursor_col] = Cell {
                c,
                fg: self.cur_fg,
                bg: self.cur_bg,
                bold: self.cur_bold,
            };
            self.cursor_col += 1;
        }
    }

    fn clear_line_from_cursor(&mut self) {
        if self.cursor_row < self.rows {
            for col in self.cursor_col..self.cols {
                self.cells[self.cursor_row][col] = Cell::default();
            }
        }
    }

    fn clear_line_to_cursor(&mut self) {
        if self.cursor_row < self.rows {
            for col in 0..=self.cursor_col.min(self.cols.saturating_sub(1)) {
                self.cells[self.cursor_row][col] = Cell::default();
            }
        }
    }

    fn clear_entire_line(&mut self) {
        if self.cursor_row < self.rows {
            for col in 0..self.cols {
                self.cells[self.cursor_row][col] = Cell::default();
            }
        }
    }

    fn clear_screen_from_cursor(&mut self) {
        self.clear_line_from_cursor();
        for row in (self.cursor_row + 1)..self.rows {
            for col in 0..self.cols {
                self.cells[row][col] = Cell::default();
            }
        }
    }

    fn clear_screen_to_cursor(&mut self) {
        self.clear_line_to_cursor();
        for row in 0..self.cursor_row {
            for col in 0..self.cols {
                self.cells[row][col] = Cell::default();
            }
        }
    }

    fn clear_entire_screen(&mut self) {
        for row in 0..self.rows {
            for col in 0..self.cols {
                self.cells[row][col] = Cell::default();
            }
        }
    }
}

impl Perform for TerminalScreen {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            // BEL
            0x07 => {}
            // Backspace
            0x08 => {
                self.cursor_col = self.cursor_col.saturating_sub(1);
            }
            // Tab
            0x09 => {
                let next_tab = (self.cursor_col / 8 + 1) * 8;
                self.cursor_col = next_tab.min(self.cols.saturating_sub(1));
            }
            // Newline / Line Feed
            0x0A => {
                self.cursor_row += 1;
                if self.cursor_row > self.scroll_bottom {
                    self.cursor_row = self.scroll_bottom;
                    self.scroll_up();
                }
            }
            // Carriage Return
            0x0D => {
                self.cursor_col = 0;
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        let params_vec: Vec<u16> = params.iter().flat_map(|s| s.iter().copied()).collect();

        match c {
            // CUU - Cursor Up
            'A' => {
                let n = params_vec.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = self.cursor_row.saturating_sub(n);
            }
            // CUB - Cursor Down
            'B' => {
                let n = params_vec.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = (self.cursor_row + n).min(self.rows.saturating_sub(1));
            }
            // CUF - Cursor Forward
            'C' => {
                let n = params_vec.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_col = (self.cursor_col + n).min(self.cols.saturating_sub(1));
            }
            // CUB - Cursor Backward
            'D' => {
                let n = params_vec.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            // CUP / HVP - Cursor Position
            'H' | 'f' => {
                let row = params_vec.first().copied().unwrap_or(1).max(1) as usize - 1;
                let col = params_vec.get(1).copied().unwrap_or(1).max(1) as usize - 1;
                self.cursor_row = row.min(self.rows.saturating_sub(1));
                self.cursor_col = col.min(self.cols.saturating_sub(1));
            }
            // ED - Erase in Display
            'J' => {
                let mode = params_vec.first().copied().unwrap_or(0);
                match mode {
                    0 => self.clear_screen_from_cursor(),
                    1 => self.clear_screen_to_cursor(),
                    2 | 3 => self.clear_entire_screen(),
                    _ => {}
                }
            }
            // EL - Erase in Line
            'K' => {
                let mode = params_vec.first().copied().unwrap_or(0);
                match mode {
                    0 => self.clear_line_from_cursor(),
                    1 => self.clear_line_to_cursor(),
                    2 => self.clear_entire_line(),
                    _ => {}
                }
            }
            // SGR - Select Graphic Rendition
            'm' => {
                if params_vec.is_empty() {
                    // Reset
                    self.cur_fg = None;
                    self.cur_bg = None;
                    self.cur_bold = false;
                    return;
                }
                let mut i = 0;
                while i < params_vec.len() {
                    match params_vec[i] {
                        0 => {
                            self.cur_fg = None;
                            self.cur_bg = None;
                            self.cur_bold = false;
                        }
                        1 => self.cur_bold = true,
                        22 => self.cur_bold = false,
                        // Standard foreground colors
                        30 => self.cur_fg = Some(Color::Black),
                        31 => self.cur_fg = Some(Color::Red),
                        32 => self.cur_fg = Some(Color::Green),
                        33 => self.cur_fg = Some(Color::Yellow),
                        34 => self.cur_fg = Some(Color::Blue),
                        35 => self.cur_fg = Some(Color::Magenta),
                        36 => self.cur_fg = Some(Color::Cyan),
                        37 => self.cur_fg = Some(Color::White),
                        39 => self.cur_fg = None, // Default fg
                        // Standard background colors
                        40 => self.cur_bg = Some(Color::Black),
                        41 => self.cur_bg = Some(Color::Red),
                        42 => self.cur_bg = Some(Color::Green),
                        43 => self.cur_bg = Some(Color::Yellow),
                        44 => self.cur_bg = Some(Color::Blue),
                        45 => self.cur_bg = Some(Color::Magenta),
                        46 => self.cur_bg = Some(Color::Cyan),
                        47 => self.cur_bg = Some(Color::White),
                        49 => self.cur_bg = None, // Default bg
                        // Bright foreground
                        90 => self.cur_fg = Some(Color::DarkGray),
                        91 => self.cur_fg = Some(Color::LightRed),
                        92 => self.cur_fg = Some(Color::LightGreen),
                        93 => self.cur_fg = Some(Color::LightYellow),
                        94 => self.cur_fg = Some(Color::LightBlue),
                        95 => self.cur_fg = Some(Color::LightMagenta),
                        96 => self.cur_fg = Some(Color::LightCyan),
                        97 => self.cur_fg = Some(Color::White),
                        // Bright background
                        100 => self.cur_bg = Some(Color::DarkGray),
                        101 => self.cur_bg = Some(Color::LightRed),
                        102 => self.cur_bg = Some(Color::LightGreen),
                        103 => self.cur_bg = Some(Color::LightYellow),
                        104 => self.cur_bg = Some(Color::LightBlue),
                        105 => self.cur_bg = Some(Color::LightMagenta),
                        106 => self.cur_bg = Some(Color::LightCyan),
                        107 => self.cur_bg = Some(Color::White),
                        // 256-color: ESC[38;5;{n}m (fg) / ESC[48;5;{n}m (bg)
                        38 => {
                            if params_vec.get(i + 1) == Some(&5) {
                                if let Some(&n) = params_vec.get(i + 2) {
                                    self.cur_fg = Some(Color::Indexed(n as u8));
                                    i += 2;
                                }
                            } else if params_vec.get(i + 1) == Some(&2) {
                                // 24-bit: ESC[38;2;r;g;b m
                                if let (Some(&r), Some(&g), Some(&b)) = (
                                    params_vec.get(i + 2),
                                    params_vec.get(i + 3),
                                    params_vec.get(i + 4),
                                ) {
                                    self.cur_fg =
                                        Some(Color::Rgb(r as u8, g as u8, b as u8));
                                    i += 4;
                                }
                            }
                        }
                        48 => {
                            if params_vec.get(i + 1) == Some(&5) {
                                if let Some(&n) = params_vec.get(i + 2) {
                                    self.cur_bg = Some(Color::Indexed(n as u8));
                                    i += 2;
                                }
                            } else if params_vec.get(i + 1) == Some(&2) {
                                if let (Some(&r), Some(&g), Some(&b)) = (
                                    params_vec.get(i + 2),
                                    params_vec.get(i + 3),
                                    params_vec.get(i + 4),
                                ) {
                                    self.cur_bg =
                                        Some(Color::Rgb(r as u8, g as u8, b as u8));
                                    i += 4;
                                }
                            }
                        }
                        _ => {} // Ignore unsupported SGR codes
                    }
                    i += 1;
                }
            }
            // Scroll region
            'r' => {
                let top = params_vec.first().copied().unwrap_or(1).max(1) as usize - 1;
                let bottom = params_vec
                    .get(1)
                    .copied()
                    .unwrap_or(self.rows as u16)
                    .max(1) as usize
                    - 1;
                self.scroll_top = top.min(self.rows.saturating_sub(1));
                self.scroll_bottom = bottom.min(self.rows.saturating_sub(1));
            }
            // IL - Insert Lines
            'L' => {
                let n = params_vec.first().copied().unwrap_or(1).max(1) as usize;
                for _ in 0..n {
                    if self.cursor_row <= self.scroll_bottom {
                        if self.scroll_bottom < self.rows {
                            self.cells.remove(self.scroll_bottom);
                        }
                        self.cells
                            .insert(self.cursor_row, vec![Cell::default(); self.cols]);
                    }
                }
            }
            // DL - Delete Lines
            'M' => {
                let n = params_vec.first().copied().unwrap_or(1).max(1) as usize;
                for _ in 0..n {
                    if self.cursor_row <= self.scroll_bottom {
                        self.cells.remove(self.cursor_row);
                        if self.scroll_bottom < self.rows {
                            self.cells
                                .insert(self.scroll_bottom, vec![Cell::default(); self.cols]);
                        }
                    }
                }
            }
            // CHA - Cursor Horizontal Absolute
            'G' => {
                let col = params_vec.first().copied().unwrap_or(1).max(1) as usize - 1;
                self.cursor_col = col.min(self.cols.saturating_sub(1));
            }
            // VPA - Vertical Position Absolute
            'd' => {
                let row = params_vec.first().copied().unwrap_or(1).max(1) as usize - 1;
                self.cursor_row = row.min(self.rows.saturating_sub(1));
            }
            // DCH - Delete Characters
            'P' => {
                let n = params_vec.first().copied().unwrap_or(1).max(1) as usize;
                if self.cursor_row < self.rows {
                    let row = &mut self.cells[self.cursor_row];
                    for _ in 0..n {
                        if self.cursor_col < row.len() {
                            row.remove(self.cursor_col);
                            row.push(Cell::default());
                        }
                    }
                }
            }
            // ICH - Insert Characters
            '@' => {
                let n = params_vec.first().copied().unwrap_or(1).max(1) as usize;
                if self.cursor_row < self.rows {
                    let row = &mut self.cells[self.cursor_row];
                    for _ in 0..n {
                        if self.cursor_col < row.len() {
                            row.insert(self.cursor_col, Cell::default());
                            row.truncate(self.cols);
                        }
                    }
                }
            }
            // ECH - Erase Characters
            'X' => {
                let n = params_vec.first().copied().unwrap_or(1).max(1) as usize;
                if self.cursor_row < self.rows {
                    for col in self.cursor_col..(self.cursor_col + n).min(self.cols) {
                        self.cells[self.cursor_row][col] = Cell::default();
                    }
                }
            }
            _ => {} // Ignore other CSI sequences
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // OSC 7: report working directory
        // VTE splits on ';', so we may get:
        //   params[0] = b"7", params[1] = b"file://hostname/path"
        // OR all in one param if the shell sends it differently.
        if params.is_empty() {
            return;
        }

        let first = String::from_utf8_lossy(params[0]);

        // Check if first param is "7" and URL is in second param
        let url_str = if first == "7" {
            params.get(1).map(|u| String::from_utf8_lossy(u).to_string())
        } else if first.starts_with("7;") {
            // Single param format: "7;file://host/path"
            Some(first[2..].to_string())
        } else {
            None
        };

        if let Some(url) = url_str {
            // Extract path from file://hostname/path
            if let Some(pos) = url.find("//") {
                let after_scheme = &url[pos + 2..];
                if let Some(slash_pos) = after_scheme.find('/') {
                    let path = &after_scheme[slash_pos..];
                    let decoded = percent_decode(path);
                    self.reported_cwd = Some(std::path::PathBuf::from(decoded));
                }
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // Ignore ESC sequences for now
    }
}

/// Decode percent-encoded URL path (e.g. %20 -> space).
fn percent_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let hi = chars.next().and_then(|c| (c as char).to_digit(16));
            let lo = chars.next().and_then(|c| (c as char).to_digit(16));
            if let (Some(h), Some(l)) = (hi, lo) {
                result.push((h * 16 + l) as u8 as char);
            }
        } else {
            result.push(b as char);
        }
    }
    result
}

/// The shell pane — manages a PTY child process and its screen buffer.
pub struct ShellPane {
    pub screen: TerminalScreen,
    writer: Box<dyn Write + Send>,
    output_rx: Receiver<Vec<u8>>,
    vte_parser: vte::Parser,
    _master: Box<dyn MasterPty + Send>,
    child_pid: Option<u32>,
    last_cwd_check: std::time::Instant,
    last_known_cwd: Option<std::path::PathBuf>,
}

impl ShellPane {
    pub fn new(rows: u16, cols: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let pty_system = NativePtySystem::default();

        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Spawn shell
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let mut cmd = CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");

        let child = pair.slave.spawn_command(cmd)?;
        let child_pid = child.process_id();
        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader()?;
        let mut writer = pair.master.take_writer()?;

        // Inject init: minimal prompt + precmd hook for OSC 7 cwd reporting.
        let is_zsh = shell.contains("zsh");
        let is_bash = shell.contains("bash");
        if is_zsh {
            // %1~ = last dir component (~ for home), %# = % for user, # for root
            let init = b" PROMPT='%1~ %# '; __mewtui_precmd(){ printf '\\e]7;file://%s%s\\a' \"$(hostname)\" \"$PWD\" }; precmd_functions+=(__mewtui_precmd); clear\n";
            let _ = writer.write_all(init);
            let _ = writer.flush();
        } else if is_bash {
            // \\W = basename of cwd (~ for home), \\$ = $ for user, # for root
            let init = b" PS1='\\W \\$ '; PROMPT_COMMAND=\"${PROMPT_COMMAND:+$PROMPT_COMMAND;}printf '\\e]7;file://%s%s\\a' \\\"$(hostname)\\\" \\\"$PWD\\\"\"; clear\n";
            let _ = writer.write_all(init);
            let _ = writer.flush();
        }

        // Channel for PTY output
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();

        // Reader thread
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(ShellPane {
            screen: TerminalScreen::new(rows as usize, cols as usize),
            writer,
            output_rx: rx,
            vte_parser: vte::Parser::new(),
            _master: pair.master,
            child_pid,
            last_cwd_check: std::time::Instant::now(),
            last_known_cwd: None,
        })
    }

    /// Process any pending PTY output. Call this every frame.
    pub fn poll_output(&mut self) {
        loop {
            match self.output_rx.try_recv() {
                Ok(data) => {
                    self.vte_parser.advance(&mut self.screen, &data);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Take the reported cwd if the shell has changed directories.
    /// Falls back to querying the child process's cwd via lsof on macOS (~1s interval).
    pub fn take_cwd(&mut self) -> Option<std::path::PathBuf> {
        // Prefer OSC 7 if available
        if let Some(cwd) = self.screen.reported_cwd.take() {
            self.last_known_cwd = Some(cwd.clone());
            return Some(cwd);
        }

        // Fallback: query child process cwd via lsof (throttled to ~1s)
        if self.last_cwd_check.elapsed() < std::time::Duration::from_secs(1) {
            return None;
        }
        self.last_cwd_check = std::time::Instant::now();

        if let Some(pid) = self.child_pid {
            if let Ok(output) = std::process::Command::new("lsof")
                .args(["-p", &pid.to_string(), "-Fn", "-a", "-d", "cwd"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if let Some(path) = line.strip_prefix('n') {
                        let new_cwd = std::path::PathBuf::from(path);
                        if self.last_known_cwd.as_ref() != Some(&new_cwd) {
                            self.last_known_cwd = Some(new_cwd.clone());
                            return Some(new_cwd);
                        }
                        return None;
                    }
                }
            }
        }

        None
    }

    /// Send raw bytes to the shell (keystrokes).
    pub fn write(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
        let _ = self.writer.flush();
    }

    /// Resize the PTY and screen buffer.
    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.screen.resize(rows as usize, cols as usize);
        let _ = self._master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
    }
}
