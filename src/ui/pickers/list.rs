use std::collections::HashMap;

use super::draw_picker_list;

const COMMANDS: &[(&str, &str)] = &[
    ("/add", "attach a file or URL to context"),
    ("/drop", "remove a context file"),
    ("/drop-all", "clear all context files"),
    ("/init", "generate AGENTS.md for this project"),
    ("/memory", "view or edit memory files"),
    ("/model", "switch the active model"),
    ("/models", "search and switch model"),
    ("/models-add", "save a quick-model alias"),
    ("/provider", "switch provider"),
    ("/sessions", "list and resume past sessions"),
    ("/reasoning", "toggle extended reasoning"),
    ("/thinking", "alias for /reasoning"),
    ("/mode", "change permission mode"),
    ("/mcp", "list connected MCP servers"),
    ("/toggle", "toggle a boolean config flag"),
    ("/compress", "summarise context to free token budget"),
    ("/compact", "alias for /compress"),
    ("/loop", "repeat a prompt on an interval"),
    ("/prompt", "list and activate system prompts"),
    ("/theme", "list and apply colour themes"),
    ("/history", "show recent input history"),
    ("/regen-prompts", "re-extract built-in prompts to disk"),
    ("/regen-themes", "re-extract built-in themes to disk"),
    ("/editsys", "open system prompt in $EDITOR"),
    ("/quit", "exit zerostack"),
    ("/exit", "alias for /quit"),
    ("/clear", "clear the current session"),
    ("/new", "start a new session"),
    ("/undo", "undo the last exchange"),
    ("/retry", "re-run the last user message"),
    ("/help", "show all commands"),
    ("/welcome", "show the quickstart screen"),
    ("/tutorial", "run the interactive tutorial"),
    ("/worktree", "create a git worktree for isolation"),
    ("/wt-merge", "merge the current worktree back"),
    ("/wt-exit", "exit the current worktree"),
    ("/btw", "send a side note while agent is running"),
    ("/queue", "queue a message while agent is running"),
];

pub struct ListPicker {
    pub active: bool,
    pub query: String,
    pub cursor: usize,
    pub matches: Vec<String>,
    pub selected: usize,
    items: Vec<String>,
    descriptions: HashMap<String, String>,
    match_descriptions: Vec<Option<String>>,
    monochrome: bool,
}

impl ListPicker {
    pub fn new() -> Self {
        ListPicker {
            active: false,
            query: String::new(),
            cursor: 0,
            matches: Vec::new(),
            selected: 0,
            items: Vec::new(),
            descriptions: HashMap::new(),
            match_descriptions: Vec::new(),
            monochrome: false,
        }
    }

    pub fn with_static_commands() -> Self {
        let mut picker = ListPicker::new();
        picker.items = COMMANDS.iter().map(|(name, _)| name.to_string()).collect();
        picker.descriptions = COMMANDS
            .iter()
            .map(|(name, desc)| (name.to_string(), desc.to_string()))
            .collect();
        picker
    }

    pub fn set_monochrome(&mut self, monochrome: bool) {
        self.monochrome = monochrome;
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.query.clear();
        self.cursor = 0;
        self.matches.clear();
        self.selected = 0;
        self.filter();
    }

    pub fn deactivate(&mut self) {
        self.active = false;
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
        let query_lower = self.query.to_lowercase();
        self.matches = self
            .items
            .iter()
            .filter(|name| name.to_lowercase().contains(&query_lower))
            .take(50)
            .cloned()
            .collect();
        self.match_descriptions = self
            .matches
            .iter()
            .map(|m| self.descriptions.get(m).cloned())
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

    pub fn draw(&self, empty_message: Option<&str>) -> std::io::Result<()> {
        if !self.active {
            return Ok(());
        }
        draw_picker_list(
            &self.matches,
            self.selected,
            self.monochrome,
            empty_message,
            4,
            &self.match_descriptions,
        )
    }
}
