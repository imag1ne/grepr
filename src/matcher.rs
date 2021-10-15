use regex::Regex;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

pub trait Matcher<'a> {
    fn match_pattern(&'a self, pattern: &'a str) -> Option<Vec<MatchLine<'a>>>;
    fn match_pattern_line(content: &'a str, pattern: &'a str) -> Option<Vec<MatchWord<'a>>>;
}

#[derive(Debug)]
pub struct MatchLine<'a> {
    row: usize,
    content: &'a str,
    match_words: Vec<MatchWord<'a>>,
}

impl<'a> MatchLine<'a> {
    pub fn new(row: usize, content: &'a str, match_words: Vec<MatchWord<'a>>) -> Self {
        Self {
            row,
            content,
            match_words,
        }
    }
}

#[derive(Debug)]
pub struct MatchWord<'a> {
    col: usize,
    word: &'a str,
}

impl<'a> From<regex::Match<'a>> for MatchWord<'a> {
    fn from(m: regex::Match<'a>) -> Self {
        Self {
            col: m.start(),
            word: m.as_str(),
        }
    }
}

impl<'a, T> Matcher<'a> for T
where
    T: AsRef<str>,
{
    fn match_pattern(&'a self, pattern: &'a str) -> Option<Vec<MatchLine<'a>>> {
        let content = self.as_ref();

        let match_lines = content
            .lines()
            .enumerate()
            .filter_map(|(row, line)| {
                Self::match_pattern_line(line, pattern).map(|w| MatchLine::new(row, line, w))
            })
            .collect::<Vec<_>>();

        if match_lines.is_empty() {
            None
        } else {
            Some(match_lines)
        }
    }

    fn match_pattern_line(content: &'a str, pattern: &'a str) -> Option<Vec<MatchWord<'a>>> {
        let re = Regex::new(pattern).unwrap();

        let match_words: Vec<_> = re
            .captures_iter(content)
            .filter_map(|cap| cap.get(0).map(|m| m.into()))
            .collect();

        if match_words.is_empty() {
            None
        } else {
            Some(match_words)
        }
    }
}

impl Display for MatchLine<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use colored::Colorize;
        let row = (self.row + 1).to_string().blue();

        let (cols, words): (Vec<_>, HashSet<_>) = self
            .match_words
            .iter()
            .map(|mw| ((mw.col + 1).to_string(), mw.word.to_string()))
            .unzip();

        let cols = cols.join(" ").blue();

        let mut content = String::new();

        for word in words {
            content = self.content.replace(&word, &format!("{}", word.red()))
        }

        let output = format!("{}:{} {}", row, cols, content);
        write!(f, "{}", output)
    }
}
