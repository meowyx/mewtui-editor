use ratatui::style::Color;
use tree_sitter::Language;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

use crate::theme::Theme;

/// Recognized highlight capture names — order matters, maps to Highlight(index).
const HIGHLIGHT_NAMES: &[&str] = &[
    "keyword",           // 0
    "function",          // 1
    "function.builtin",  // 2
    "string",            // 3
    "string.escape",     // 4
    "comment",           // 5
    "type",              // 6
    "type.builtin",      // 7
    "constant",          // 8
    "constant.builtin",  // 9
    "number",            // 10
    "operator",          // 11
    "punctuation",       // 12
    "punctuation.bracket", // 13
    "punctuation.delimiter", // 14
    "variable",          // 15
    "variable.builtin",  // 16
    "variable.parameter", // 17
    "property",          // 18
    "constructor",       // 19
    "module",            // 20
    "attribute",         // 21
    "boolean",           // 22
    "escape",            // 23
];

/// A colored span in the source text (byte offsets).
pub struct ColorSpan {
    pub start: usize,
    pub end: usize,
    pub color: Color,
}

pub struct SyntaxHighlighter {
    highlighter: Highlighter,
    rust_config: HighlightConfiguration,
    javascript_config: HighlightConfiguration,
    python_config: HighlightConfiguration,
    json_config: HighlightConfiguration,
    toml_config: HighlightConfiguration,
    go_config: HighlightConfiguration,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let highlighter = Highlighter::new();

        let mut rust_config = HighlightConfiguration::new(
            Language::from(tree_sitter_rust::LANGUAGE),
            "rust",
            include_str!("queries/rust-highlights.scm"),
            "",
            "",
        )
        .expect("Failed to create Rust highlight config");
        rust_config.configure(HIGHLIGHT_NAMES);

        let mut javascript_config = HighlightConfiguration::new(
            Language::from(tree_sitter_javascript::LANGUAGE),
            "javascript",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            "",
            "",
        )
        .expect("Failed to create JavaScript highlight config");
        javascript_config.configure(HIGHLIGHT_NAMES);

        let mut python_config = HighlightConfiguration::new(
            Language::from(tree_sitter_python::LANGUAGE),
            "python",
            tree_sitter_python::HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .expect("Failed to create Python highlight config");
        python_config.configure(HIGHLIGHT_NAMES);

        let mut json_config = HighlightConfiguration::new(
            Language::from(tree_sitter_json::LANGUAGE),
            "json",
            tree_sitter_json::HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .expect("Failed to create JSON highlight config");
        json_config.configure(HIGHLIGHT_NAMES);

        let mut toml_config = HighlightConfiguration::new(
            Language::from(tree_sitter_toml_ng::LANGUAGE),
            "toml",
            tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .expect("Failed to create TOML highlight config");
        toml_config.configure(HIGHLIGHT_NAMES);

        let mut go_config = HighlightConfiguration::new(
            Language::from(tree_sitter_go::LANGUAGE),
            "go",
            tree_sitter_go::HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .expect("Failed to create Go highlight config");
        go_config.configure(HIGHLIGHT_NAMES);

        SyntaxHighlighter {
            highlighter,
            rust_config,
            javascript_config,
            python_config,
            json_config,
            toml_config,
            go_config,
        }
    }

    /// Compute highlight spans for the given source code.
    /// Returns byte-offset ranges with associated colors from the theme.
    pub fn highlight(&mut self, source: &str, extension: &str, theme: &Theme) -> Vec<ColorSpan> {
        let config = match extension {
            "rs" => &self.rust_config,
            "js" | "jsx" | "mjs" | "cjs" => &self.javascript_config,
            "ts" | "tsx" => &self.javascript_config,
            "py" => &self.python_config,
            "json" => &self.json_config,
            "toml" => &self.toml_config,
            "go" => &self.go_config,
            _ => return Vec::new(),
        };

        let Ok(events) = self.highlighter.highlight(config, source.as_bytes(), None, |_| None)
        else {
            return Vec::new();
        };

        let mut spans = Vec::new();
        let mut highlight_stack: Vec<Color> = Vec::new();

        for event in events {
            let Ok(event) = event else {
                break;
            };
            match event {
                HighlightEvent::Source { start, end } => {
                    if let Some(&color) = highlight_stack.last() {
                        spans.push(ColorSpan {
                            start,
                            end,
                            color,
                        });
                    }
                }
                HighlightEvent::HighlightStart(Highlight(idx)) => {
                    let color = highlight_color(idx, theme);
                    highlight_stack.push(color);
                }
                HighlightEvent::HighlightEnd => {
                    highlight_stack.pop();
                }
            }
        }

        spans
    }
}

/// Map highlight index to a theme color.
fn highlight_color(idx: usize, theme: &Theme) -> Color {
    match idx {
        0 => theme.syntax_keyword,       // keyword
        1 | 2 => theme.syntax_function,  // function, function.builtin
        3 | 4 => theme.syntax_string,    // string, string.escape
        5 => theme.syntax_comment,       // comment
        6 | 7 => theme.syntax_keyword,   // type, type.builtin (use keyword color)
        8 | 9 => theme.accent,           // constant, constant.builtin
        10 => theme.accent,              // number
        11 => theme.fg,                  // operator
        12 | 13 | 14 => theme.fg,        // punctuation
        15 | 16 | 17 => theme.fg,        // variable
        18 => theme.syntax_function,     // property
        19 => theme.syntax_function,     // constructor
        20 => theme.syntax_string,       // module
        21 => theme.syntax_comment,      // attribute
        22 => theme.accent,              // boolean
        23 => theme.syntax_string,       // escape
        _ => theme.fg,
    }
}
