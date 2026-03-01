use std::collections::{HashMap, HashSet};

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub struct SyntaxHighlighter {
    ss: SyntaxSet,
    theme: Theme,
}

impl SyntaxHighlighter {
    pub fn new(theme_name: &str) -> Self {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let theme = ts
            .themes
            .get(theme_name)
            .cloned()
            .unwrap_or_else(|| ts.themes.values().next().cloned().unwrap());
        Self { ss, theme }
    }

    pub fn highlight_visible_lines(
        &self,
        buffer: &crate::domain::Buffer,
        file_path: Option<&str>,
        visible_logical_lines: &[usize],
    ) -> HashMap<usize, Vec<[f32; 4]>> {
        if visible_logical_lines.is_empty() || buffer.len_lines() == 0 {
            return HashMap::new();
        }

        let syntax = self.pick_syntax(file_path);

        let visible_set: HashSet<usize> = visible_logical_lines.iter().copied().collect();
        let max_line = *visible_logical_lines.iter().max().unwrap_or(&0);

        let mut hl = HighlightLines::new(syntax, &self.theme);
        let mut out: HashMap<usize, Vec<[f32; 4]>> = HashMap::new();

        for line_idx in 0..=max_line.min(buffer.len_lines().saturating_sub(1)) {
            let line = buffer
                .line_slice(line_idx)
                .map(|l| l.to_string())
                .unwrap_or_default();

            let ranges = hl
                .highlight_line(&line, &self.ss)
                .unwrap_or_else(|_| vec![(Style::default(), line.as_str())]);

            if visible_set.contains(&line_idx) {
                let mut colors: Vec<[f32; 4]> = Vec::with_capacity(line.chars().count());
                for (style, seg) in ranges {
                    let c = style_to_rgba(style);
                    colors.extend(seg.chars().map(|_| c));
                }
                out.insert(line_idx, colors);
            }
        }

        out
    }

    fn pick_syntax(&self, file_path: Option<&str>) -> &SyntaxReference {
        if let Some(path) = file_path {
            if let Some(ext) = path.rsplit('.').next() {
                if let Some(s) = self.ss.find_syntax_by_extension(ext) {
                    return s;
                }
            }
        }
        self.ss.find_syntax_plain_text()
    }
}

fn style_to_rgba(style: Style) -> [f32; 4] {
    let fg = style.foreground;
    [
        (fg.r as f32) / 255.0,
        (fg.g as f32) / 255.0,
        (fg.b as f32) / 255.0,
        (fg.a as f32) / 255.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::SyntaxHighlighter;
    use crate::domain::Buffer;

    #[test]
    fn test_syntax_highlighter_new_valid_theme() {
        let _highlighter = SyntaxHighlighter::new("base16-ocean.dark");
        assert!(true);
    }

    #[test]
    fn test_syntax_highlighter_new_invalid_theme_fallback() {
        let _highlighter = SyntaxHighlighter::new("nonexistent-theme-xyz");
        assert!(true);
    }

    #[test]
    fn test_highlight_visible_lines_empty_buffer() {
        let highlighter = SyntaxHighlighter::new("base16-ocean.dark");
        let buffer = Buffer::from_str("");
        let result = highlighter.highlight_visible_lines(&buffer, None, &[0]);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&0));
        assert!(result.get(&0).unwrap().is_empty());
    }

    #[test]
    fn test_highlight_visible_lines_empty_visibility() {
        let highlighter = SyntaxHighlighter::new("base16-ocean.dark");
        let buffer = Buffer::from_str("hello\nworld");
        let result = highlighter.highlight_visible_lines(&buffer, None, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_highlight_visible_lines_with_visible_lines() {
        let highlighter = SyntaxHighlighter::new("base16-ocean.dark");
        let buffer = Buffer::from_str("fn main() {}\nlet x = 1;");
        let result = highlighter.highlight_visible_lines(&buffer, None, &[0, 1]);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&0));
        assert!(result.contains_key(&1));
    }

    #[test]
    fn test_highlight_visible_lines_with_file_extension() {
        let highlighter = SyntaxHighlighter::new("base16-ocean.dark");
        let buffer = Buffer::from_str("fn main() {}");
        let result = highlighter.highlight_visible_lines(&buffer, Some("test.rs"), &[0]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_highlight_visible_lines_unknown_extension() {
        let highlighter = SyntaxHighlighter::new("base16-ocean.dark");
        let buffer = Buffer::from_str("some content");
        let result = highlighter.highlight_visible_lines(&buffer, Some("test.xyz"), &[0]);
        assert_eq!(result.len(), 1);
    }
}
