use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};

use crate::app::{App, Focus};

fn make_block<'a>(title: &'a str, focus: Focus, current: Focus, app: &'a App) -> Block<'a> {
    let theme = app.current_theme();
    let is_active = focus == current;
    Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_type(if is_active { BorderType::Double } else { BorderType::Plain })
        .border_style(Style::default().fg(if is_active { theme.border_active } else { theme.border_inactive }))
        .style(Style::default().bg(theme.bg).fg(theme.fg))
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let theme = app.current_theme();
    let area = frame.area();

    // Fill background
    frame.render_widget(
        Block::default().style(Style::default().bg(theme.bg)),
        area,
    );

    // Reserve bottom row for status bar
    let main_and_status = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);

    let main_area = main_and_status[0];
    let status_area = main_and_status[1];

    // Three-pane horizontal layout
    let chunks = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(45),
        Constraint::Percentage(15),
    ])
    .split(main_area);

    draw_shell(frame, app, chunks[0]);
    draw_editor(frame, app, chunks[1]);
    draw_file_tree(frame, app, chunks[2]);
    draw_status_bar(frame, app, status_area);
}

fn draw_shell(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.current_theme();
    let block = make_block("Shell", Focus::Shell, app.focus, app);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(shell) = &mut app.shell else {
        let msg = Paragraph::new("Shell not available")
            .style(Style::default().fg(theme.border_inactive));
        frame.render_widget(msg, inner);
        return;
    };

    // Poll for new PTY output
    shell.poll_output();

    // Resize shell screen to match pane if needed
    let pane_rows = inner.height as usize;
    let pane_cols = inner.width as usize;
    if shell.screen.rows != pane_rows || shell.screen.cols != pane_cols {
        shell.resize(pane_rows as u16, pane_cols as u16);
    }

    // Render the terminal screen buffer
    let mut lines: Vec<Line> = Vec::with_capacity(pane_rows);
    for row_idx in 0..pane_rows.min(shell.screen.rows) {
        let row = &shell.screen.cells[row_idx];
        let mut spans: Vec<Span> = Vec::new();
        let mut run_start = 0;

        while run_start < pane_cols.min(shell.screen.cols) {
            let cell = &row[run_start];
            let fg = cell.fg.unwrap_or(theme.fg);
            let bg = cell.bg.unwrap_or(theme.bg);
            let mut style = Style::default().fg(fg).bg(bg);
            if cell.bold {
                style = style.bold();
            }

            // Collect run of same style
            let mut run_end = run_start + 1;
            while run_end < pane_cols.min(shell.screen.cols) {
                let next = &row[run_end];
                let nfg = next.fg.unwrap_or(theme.fg);
                let nbg = next.bg.unwrap_or(theme.bg);
                if nfg != fg || nbg != bg || next.bold != cell.bold {
                    break;
                }
                run_end += 1;
            }

            let text: String = row[run_start..run_end].iter().map(|c| c.c).collect();
            spans.push(Span::styled(text, style));
            run_start = run_end;
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn draw_editor(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.current_theme();
    let editor_title = if let Some(path) = &app.buffer.file_path {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let modified = if app.buffer.modified { " [+]" } else { "" };
        format!("{name}{modified}")
    } else {
        "Editor".to_string()
    };
    let block = make_block(&editor_title, Focus::Editor, app.focus, app);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.buffer.file_path.is_none() && app.buffer.rope.len_chars() == 0 {
        let hint = Paragraph::new("Open a file from the file tree (Tab → Enter)")
            .style(Style::default().fg(theme.border_inactive));
        frame.render_widget(hint, inner);
        return;
    }

    // Recompute highlights if dirty
    app.update_highlights();

    // Calculate visible area (inner area minus line number gutter)
    let total_lines = app.buffer.rope.len_lines();
    let gutter_width = line_number_width(total_lines) + 1; // +1 for space separator
    let text_width = inner.width.saturating_sub(gutter_width as u16) as usize;
    let pane_height = inner.height as usize;

    // Ensure cursor is visible
    app.buffer.ensure_cursor_visible(pane_height);

    let scroll = app.buffer.scroll_offset;
    let mut lines: Vec<Line> = Vec::with_capacity(pane_height);

    for i in 0..pane_height {
        let line_idx = scroll + i;
        if line_idx >= total_lines {
            let gutter = format!("{:>width$} ", "~", width = gutter_width - 1);
            lines.push(Line::from(vec![
                Span::styled(gutter, Style::default().fg(theme.border_inactive)),
            ]));
            continue;
        }

        // Line number gutter
        let line_num = format!("{:>width$} ", line_idx + 1, width = gutter_width - 1);
        let gutter_style = if line_idx == app.buffer.cursor_row {
            Style::default().fg(theme.fg).bold()
        } else {
            Style::default().fg(theme.border_inactive)
        };

        // Get line content
        let rope_line = app.buffer.rope.line(line_idx);
        let line_text: String = rope_line.chars()
            .take(text_width)
            .filter(|&c| c != '\n')
            .collect();

        // Compute byte offset of this line in the full source for highlight lookup
        let line_byte_start = app.buffer.rope.line_to_byte(line_idx);
        let line_char_start = app.buffer.rope.line_to_char(line_idx);

        // Build per-character colors from highlight spans
        let chars: Vec<char> = line_text.chars().collect();
        let mut char_colors: Vec<Color> = vec![theme.fg; chars.len()];

        // Map highlight spans (byte offsets) to character positions in this line
        if !app.highlight_spans.is_empty() {
            // Get byte offsets for each char in this line
            let mut byte_offsets: Vec<usize> = Vec::with_capacity(chars.len() + 1);
            for ci in 0..chars.len() {
                let char_idx = line_char_start + ci;
                if char_idx < app.buffer.rope.len_chars() {
                    byte_offsets.push(app.buffer.rope.char_to_byte(char_idx));
                }
            }
            // End byte
            let end_char = line_char_start + chars.len();
            if end_char <= app.buffer.rope.len_chars() {
                byte_offsets.push(app.buffer.rope.char_to_byte(end_char.min(app.buffer.rope.len_chars())));
            } else if let Some(&last) = byte_offsets.last() {
                byte_offsets.push(last + 1);
            }

            let line_byte_end = byte_offsets.last().copied().unwrap_or(line_byte_start);

            for span in &app.highlight_spans {
                // Skip spans that don't overlap this line
                if span.end <= line_byte_start || span.start >= line_byte_end {
                    continue;
                }
                // Apply color to overlapping characters
                for (ci, _) in chars.iter().enumerate() {
                    if ci < byte_offsets.len() - 1 {
                        let char_byte = byte_offsets[ci];
                        if char_byte >= span.start && char_byte < span.end {
                            char_colors[ci] = span.color;
                        }
                    }
                }
            }
        }

        // Build spans with syntax colors + cursor + selection overlay
        let mut spans = vec![Span::styled(line_num, gutter_style)];
        let is_cursor_line = line_idx == app.buffer.cursor_row && app.focus == Focus::Editor;
        let cursor_col = app.buffer.cursor_col;
        let has_selection = app.buffer.anchor.is_some();

        if chars.is_empty() {
            if is_cursor_line {
                spans.push(Span::styled(" ", Style::default().fg(theme.bg).bg(theme.fg)));
            }
        } else {
            for (ci, ch) in chars.iter().enumerate() {
                let is_cursor = is_cursor_line && ci == cursor_col;
                let is_sel = has_selection && app.buffer.is_selected(line_idx, ci);
                let fg_color = char_colors[ci];

                let style = if is_cursor {
                    // Cursor: always inverted
                    Style::default().fg(theme.bg).bg(fg_color)
                } else if is_sel {
                    // Selection: swap fg/bg
                    Style::default().fg(theme.bg).bg(fg_color)
                } else {
                    Style::default().fg(fg_color)
                };

                spans.push(Span::styled(ch.to_string(), style));
            }
            // Cursor past end of line
            if is_cursor_line && cursor_col >= chars.len() {
                spans.push(Span::styled(" ", Style::default().fg(theme.bg).bg(theme.fg)));
            }
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn draw_file_tree(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.current_theme();
    let dir_name = app.file_tree.root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());
    let block = make_block(&dir_name, Focus::FileTree, app.focus, app);

    let items: Vec<ListItem> = app
        .file_tree
        .entries
        .iter()
        .map(|entry| {
            let indent = "  ".repeat(entry.depth);
            let icon = if entry.is_dir {
                if entry.expanded { "v " } else { "> " }
            } else {
                "  "
            };
            let line = Line::from(vec![
                Span::raw(format!("{indent}{icon}")),
                Span::styled(
                    entry.name.clone(),
                    if entry.is_dir {
                        Style::default().fg(theme.accent).bold()
                    } else {
                        Style::default().fg(theme.fg)
                    },
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(theme.accent).fg(theme.bg).bold());

    let mut list_state = ListState::default().with_selected(Some(app.file_tree.selected));
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let theme = app.current_theme();

    let file_info = if let Some(path) = &app.buffer.file_path {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let modified = if app.buffer.modified { " [+]" } else { "" };
        let pos = format!("{}:{}", app.buffer.cursor_row + 1, app.buffer.cursor_col + 1);
        format!(" {name}{modified}  {pos}")
    } else {
        " No file open".to_string()
    };

    let theme_name = theme.name;
    let focus_label = match app.focus {
        Focus::Shell => "Shell",
        Focus::Editor => "Editor",
        Focus::FileTree => "Files",
    };
    let sel_hint = if app.buffer.anchor.is_some() { " SEL " } else { "" };
    let right = format!("{sel_hint} [{focus_label}] {theme_name} ");

    // Pad middle
    let total_width = area.width as usize;
    let left_len = file_info.len();
    let right_len = right.len();
    let padding = total_width.saturating_sub(left_len + right_len);

    let status_text = format!("{file_info}{}{right}", " ".repeat(padding));

    let mut status_line = Line::from(vec![
        Span::styled(status_text, Style::default().fg(theme.bg).bg(theme.accent)),
    ]);

    // Override with status message if present
    if let Some(msg) = &app.status_msg {
        status_line = Line::from(vec![
            Span::styled(format!(" {msg}"), Style::default().fg(theme.bg).bg(theme.fg).bold()),
        ]);
    }

    let bar = Paragraph::new(status_line);
    frame.render_widget(bar, area);
}

fn line_number_width(total_lines: usize) -> usize {
    if total_lines == 0 { 1 } else { (total_lines as f64).log10().floor() as usize + 1 }
}
