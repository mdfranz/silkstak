use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Pattern {
    regex: OnceLock<Regex>,
    pub original: String,
    is_regex: bool,
}

impl Pattern {
    pub fn new(pattern: &str) -> Self {
        Pattern {
            regex: OnceLock::new(),
            original: pattern.to_string(),
            is_regex: false,
        }
    }

    pub fn new_regex(pattern: &str) -> Self {
        Pattern {
            regex: OnceLock::new(),
            original: pattern.to_string(),
            is_regex: true,
        }
    }

    pub fn matches(&self, input: &str) -> bool {
        let regex = self.regex.get_or_init(|| {
            let expanded = expand_home(&self.original);
            let regex_str = if self.is_regex {
                expanded
            } else {
                glob_to_regex(&expanded)
            };
            Regex::new(&regex_str).unwrap_or_else(|_| Regex::new("^$").unwrap())
        });
        regex.is_match(input)
    }
}

fn expand_home(pattern: &str) -> String {
    if pattern == "~" || pattern == "$HOME" {
        if let Some(home) = dirs::home_dir() {
            return home.to_string_lossy().to_string();
        }
        return pattern.to_string();
    }
    if let Some(rest) = pattern.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}/{}", home.to_string_lossy(), rest);
        }
        return pattern.to_string();
    }
    if let Some(rest) = pattern.strip_prefix("$HOME/")
        && let Some(home) = dirs::home_dir()
    {
        return format!("{}/{}", home.to_string_lossy(), rest);
    }
    pattern.to_string()
}

fn glob_to_regex(pattern: &str) -> String {
    let mut re = String::with_capacity(pattern.len() * 2);
    re.push('^');
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    if chars.peek() == Some(&'/') {
                        chars.next();
                        re.push_str("(?:.*/)?");
                    } else {
                        re.push_str(".*");
                    }
                } else {
                    re.push_str("[^/]*");
                }
            }
            '?' => re.push('.'),
            '.' => re.push_str("\\."),
            '\\' => re.push_str("\\\\"),
            '(' | ')' | '[' | ']' | '{' | '}' | '+' | '^' | '$' | '|' => {
                re.push('\\');
                re.push(c);
            }
            _ => re.push(c),
        }
    }
    re.push('$');
    re
}
