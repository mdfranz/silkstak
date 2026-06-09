use crossterm::event::KeyEvent;
use crossterm::style::Color;

use crate::ui::pickers::file::FilePicker;
use crate::ui::pickers::handlers;
use crate::ui::pickers::list::ListPicker;
use crate::ui::pickers::models::ModelsPicker;
use crate::ui::renderer::LineEntry;
use compact_str::CompactString;

pub enum Picker {
    File(FilePicker),
    Command(ListPicker),
    Prefixed(ListPicker, &'static str),
    Models(ModelsPicker),
}

impl Picker {
    pub fn active(&self) -> bool {
        match self {
            Picker::File(p) => p.active,
            Picker::Command(p) => p.active,
            Picker::Prefixed(p, _) => p.active,
            Picker::Models(p) => p.active,
        }
    }

    pub fn set_monochrome(&mut self, monochrome: bool) {
        match self {
            Picker::File(p) => p.set_monochrome(monochrome),
            Picker::Command(p) => p.set_monochrome(monochrome),
            Picker::Prefixed(p, _) => p.set_monochrome(monochrome),
            Picker::Models(p) => p.set_monochrome(monochrome),
        }
    }

    pub fn header(&self) -> Option<LineEntry> {
        let text = match self {
            Picker::File(p) => p.header_text(),
            Picker::Command(p) => p.header_text(None),
            Picker::Prefixed(p, prefix) => p.header_text(Some(prefix)),
            Picker::Models(p) => p.header_text(),
        };
        Some(LineEntry {
            text: CompactString::from(text),
            color: Color::White,
        })
    }

    pub fn draw(&mut self) -> std::io::Result<()> {
        match self {
            Picker::File(p) => p.draw(),
            Picker::Command(p) => p.draw(None),
            Picker::Prefixed(p, prefix) => {
                let msg = if *prefix == "/provider " {
                    Some("no matches  (type a registered custom gateway name)")
                } else {
                    None
                };
                p.draw(msg)
            }
            Picker::Models(p) => p.draw(),
        }
    }
}

use super::InputEditor;

impl InputEditor {
    pub fn handle_picker_key(&mut self, key: KeyEvent) -> bool {
        let handled = match self.picker.as_mut() {
            Some(Picker::File(p)) => {
                handlers::handle_file_key(&mut self.buffer, &mut self.cursor, p, key)
            }
            Some(Picker::Command(p)) => {
                let ctx = handlers::CommandPickerCtx {
                    prompt_names: &self.prompt_names,
                    theme_names: &self.theme_names,
                    quick_model_names: &self.quick_model_names,
                    live_model_names: &self.live_model_names,
                    provider_names: &self.provider_names,
                };
                let (handled, replacement) =
                    handlers::handle_command_key(&mut self.buffer, &mut self.cursor, &ctx, p, key);
                if let Some(new) = replacement {
                    self.picker = Some(new);
                }
                handled
            }
            Some(Picker::Prefixed(p, prefix)) => {
                handlers::handle_prefixed_key(&mut self.buffer, &mut self.cursor, p, prefix, key)
            }
            Some(Picker::Models(p)) => {
                handlers::handle_models_key(&mut self.buffer, &mut self.cursor, p, key)
            }
            None => false,
        };
        if handled {
            self.yank_pos = None;
        }
        handled
    }
}
