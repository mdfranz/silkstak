use crate::ui::pickers::{draw_picker_list, fuzzy_score};

pub struct ModelsPicker {
    pub active: bool,
    pub query: String,
    pub cursor: usize,
    pub matches: Vec<String>,
    pub selected: usize,
    quick: Vec<String>,
    provider: Vec<String>,
    pub group: usize,
    monochrome: bool,
}

impl ModelsPicker {
    pub fn new() -> Self {
        ModelsPicker {
            active: false,
            query: String::new(),
            cursor: 0,
            matches: Vec::new(),
            selected: 0,
            quick: Vec::new(),
            provider: Vec::new(),
            group: 0,
            monochrome: false,
        }
    }

    pub fn set_monochrome(&mut self, monochrome: bool) {
        self.monochrome = monochrome;
    }

    pub fn set_groups(&mut self, quick: Vec<String>, provider: Vec<String>) {
        self.quick = quick;
        self.provider = provider;
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.query.clear();
        self.cursor = 0;
        self.matches.clear();
        self.selected = 0;
        self.group = if self.quick.is_empty() && !self.provider.is_empty() {
            1
        } else {
            0
        };
        self.filter();
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn toggle_group(&mut self) {
        self.group = 1 - self.group;
        self.selected = 0;
        self.filter();
    }

    pub fn char_input(&mut self, c: char) {
        let byte_pos = self
            .query
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.query.len());
        self.query.insert(byte_pos, c);
        self.cursor += 1;
        self.filter();
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 && !self.query.is_empty() {
            self.cursor -= 1;
            let byte_pos = self
                .query
                .char_indices()
                .nth(self.cursor)
                .map(|(i, _)| i)
                .unwrap_or(self.query.len());
            self.query.remove(byte_pos);
            self.filter();
        }
    }

    fn filter(&mut self) {
        let src = if self.group == 0 {
            &self.quick
        } else {
            &self.provider
        };
        let mut scored: Vec<(i32, &String)> = src
            .iter()
            .filter_map(|n| fuzzy_score(n, &self.query).map(|s| (s, n)))
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        self.matches = scored
            .into_iter()
            .take(50)
            .map(|(_, n)| n.clone())
            .collect();
        self.selected = 0;
    }

    pub fn select_next(&mut self) {
        if !self.matches.is_empty() {
            self.selected = (self.selected + 1) % self.matches.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.matches.is_empty() {
            self.selected = if self.selected == 0 {
                self.matches.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn selected_name(&self) -> Option<&str> {
        self.matches.get(self.selected).map(|s| s.as_str())
    }

    pub fn header_text(&self) -> String {
        let tab = |label: &str, count: usize, active: bool| {
            if active {
                format!("[{} {}]", label, count)
            } else {
                format!(" {} {} ", label, count)
            }
        };
        format!(
            "{}  {}   (Tab to switch · /models refresh for the latest)",
            tab("Quick", self.quick.len(), self.group == 0),
            tab("Provider", self.provider.len(), self.group == 1)
        )
    }

    pub fn draw(&self) -> std::io::Result<()> {
        if !self.active {
            return Ok(());
        }
        draw_picker_list(&self.matches, self.selected, self.monochrome, None, 5, &[])
    }
}
