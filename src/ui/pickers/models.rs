use crate::ui::pickers::{draw_picker_list, fuzzy_score};

pub struct ModelsPicker {
    pub active: bool,
    pub query: String,
    pub cursor: usize,
    pub matches: Vec<String>,
    pub selected: usize,
    quick: Vec<(String, String)>,
    provider: Vec<String>,
    provider_name: String,
    match_descriptions: Vec<Option<String>>,
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
            provider_name: String::new(),
            match_descriptions: Vec::new(),
            group: 0,
            monochrome: false,
        }
    }

    pub fn set_monochrome(&mut self, monochrome: bool) {
        self.monochrome = monochrome;
    }

    pub fn set_groups(
        &mut self,
        quick: Vec<(String, String)>,
        provider: Vec<String>,
        provider_name: String,
    ) {
        self.quick = quick;
        self.provider = provider;
        self.provider_name = provider_name;
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.query.clear();
        self.cursor = 0;
        self.matches.clear();
        self.selected = 0;
        self.group = if self.provider.is_empty() && !self.quick.is_empty() {
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
        if self.group == 0 {
            let mut scored: Vec<(i32, &String)> = self
                .provider
                .iter()
                .filter_map(|n| fuzzy_score(n, &self.query).map(|s| (s, n)))
                .collect();
            scored.sort_by_key(|b| std::cmp::Reverse(b.0));
            self.matches = scored.iter().take(50).map(|(_, n)| (*n).clone()).collect();
            self.match_descriptions = vec![None; self.matches.len()];
        } else {
            let mut scored: Vec<(i32, &String, &String)> = self
                .quick
                .iter()
                .filter_map(|(name, model)| {
                    fuzzy_score(name, &self.query).map(|s| (s, name, model))
                })
                .collect();
            scored.sort_by_key(|b| std::cmp::Reverse(b.0));
            self.matches = scored
                .iter()
                .take(50)
                .map(|(_, n, _)| (*n).clone())
                .collect();
            self.match_descriptions = scored
                .iter()
                .take(50)
                .map(|(_, _, m)| Some((*m).clone()))
                .collect();
        }
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
        let label_provider = {
            let name = if self.provider_name.is_empty() {
                "Provider".to_string()
            } else {
                format!("{} Models", self.provider_name)
            };
            if self.group == 0 {
                format!("▶ All {} ({})", name, self.provider.len())
            } else {
                format!("  All {} ({})", name, self.provider.len())
            }
        };

        let label_quick = if self.group == 1 {
            format!("▶ Aliases ({})", self.quick.len())
        } else {
            format!("  Aliases ({})", self.quick.len())
        };

        format!(
            "{}   {}    (Left/Right to switch · Tab to cycle)",
            label_provider, label_quick
        )
    }

    pub fn draw(&self) -> std::io::Result<()> {
        if !self.active {
            return Ok(());
        }
        draw_picker_list(
            &self.matches,
            self.selected,
            self.monochrome,
            None,
            5,
            &self.match_descriptions,
        )
    }
}
