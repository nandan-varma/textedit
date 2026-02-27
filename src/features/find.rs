use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindReplace {
    pub search_text: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
    pub wrap_around: bool,
    pub show_replace: bool,
    #[serde(skip)]
    last_match: Option<usize>,
    #[serde(skip)]
    regex_cache: Option<Regex>,
}

impl Default for FindReplace {
    fn default() -> Self {
        Self {
            search_text: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            whole_word: false,
            regex: false,
            wrap_around: true,
            show_replace: false,
            last_match: None,
            regex_cache: None,
        }
    }
}

impl FindReplace {
    pub fn build_pattern(&mut self) -> Option<Regex> {
        if self.search_text.is_empty() {
            self.regex_cache = None;
            return None;
        }

        let pattern = if self.regex {
            self.search_text.clone()
        } else {
            regex::escape(&self.search_text)
        };

        let pattern = if self.whole_word {
            format!("\\b{}\\b", pattern)
        } else {
            pattern
        };

        let pattern = if self.case_sensitive {
            pattern
        } else {
            format!("(?i){}", pattern)
        };

        match Regex::new(&pattern) {
            Ok(re) => {
                self.regex_cache = Some(re);
                self.regex_cache.clone()
            }
            Err(e) => {
                log::warn!("Invalid regex: {}", e);
                self.regex_cache = None;
                None
            }
        }
    }

    pub fn find_next(&mut self, text: &str, start_from: usize) -> Option<(usize, usize)> {
        if self.search_text.is_empty() {
            return None;
        }

        let pattern = self.build_pattern()?;
        let search_text = &text[start_from..];

        if let Some(m) = pattern.find(search_text) {
            let start = start_from + m.start();
            let end = start_from + m.end();
            self.last_match = Some(start);
            return Some((start, end));
        }

        if self.wrap_around && start_from > 0 {
            if let Some(m) = pattern.find(text) {
                self.last_match = Some(m.start());
                return Some((m.start(), m.end()));
            }
        }

        None
    }

    pub fn find_all(&mut self, text: &str) -> Vec<(usize, usize)> {
        if self.search_text.is_empty() {
            return Vec::new();
        }

        let pattern = match self.build_pattern() {
            Some(p) => p,
            None => return Vec::new(),
        };

        pattern
            .find_iter(text)
            .map(|m| (m.start(), m.end()))
            .collect()
    }

    pub fn replace(&self, text: &str) -> String {
        if self.search_text.is_empty() {
            return text.to_string();
        }

        if let Some(ref pattern) = self.regex_cache {
            if self.regex {
                pattern
                    .replace_all(text, self.replace_text.as_str())
                    .to_string()
            } else {
                text.replace(&self.search_text, &self.replace_text)
            }
        } else {
            text.replace(&self.search_text, &self.replace_text)
        }
    }

    pub fn replace_match(&self, text: &str, range: (usize, usize)) -> String {
        let (start, end) = range;
        let mut result = String::new();
        result.push_str(&text[..start]);
        result.push_str(&self.replace_text);
        result.push_str(&text[end..]);
        result
    }

    pub fn count_matches(&mut self, text: &str) -> usize {
        self.find_all(text).len()
    }

    pub fn clear(&mut self) {
        self.search_text.clear();
        self.replace_text.clear();
        self.last_match = None;
        self.regex_cache = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_simple() {
        let mut fr = FindReplace::default();
        fr.search_text = "hello".to_string();

        let text = "hello world hello";
        let matches = fr.find_all(text);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (0, 5));
        assert_eq!(matches[1], (12, 17));
    }

    #[test]
    fn test_find_case_insensitive() {
        let mut fr = FindReplace::default();
        fr.search_text = "hello".to_string();
        fr.case_sensitive = false;

        let text = "Hello HELLO hello";
        let matches = fr.find_all(text);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_find_whole_word() {
        let mut fr = FindReplace::default();
        fr.search_text = "hello".to_string();
        fr.whole_word = true;

        let text = "hello world hellos";
        let matches = fr.find_all(text);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_replace_simple() {
        let mut fr = FindReplace::default();
        fr.search_text = "hello".to_string();
        fr.replace_text = "hi".to_string();

        let result = fr.replace("hello world");
        assert_eq!(result, "hi world");
    }

    #[test]
    fn test_replace_all() {
        let mut fr = FindReplace::default();
        fr.search_text = "o".to_string();
        fr.replace_text = "x".to_string();

        let result = fr.replace("hello world");
        assert_eq!(result, "hellx wxrld");
    }

    #[test]
    fn test_regex_find() {
        let mut fr = FindReplace::default();
        fr.search_text = r"\d+".to_string();
        fr.regex = true;

        let text = "abc 123 def 456";
        let matches = fr.find_all(text);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_regex_replace() {
        let mut fr = FindReplace::default();
        fr.search_text = "hello".to_string();
        fr.replace_text = "hi".to_string();

        let result = fr.replace("hello world");
        assert_eq!(result, "hi world");
    }

    #[test]
    fn test_wrap_around() {
        let mut fr = FindReplace::default();
        fr.search_text = "hello".to_string();
        fr.wrap_around = true;

        let text = "hello world";
        let result = fr.find_next(text, 6);
        assert!(result.is_some());
    }
}
