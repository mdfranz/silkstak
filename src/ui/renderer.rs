use std::io::{self, Write};

use compact_str::CompactString;
use crossterm::ExecutableCommand;
use crossterm::cursor::{Hide, MoveTo, SetCursorStyle, Show};
use crossterm::style::{
    Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{Clear, ClearType, ScrollUp};
use smallvec::{SmallVec, smallvec};

use super::markdown::word_wrap;
use super::utils::{char_display_width, display_width, resolve_color};

#[derive(Clone)]
pub struct LineEntry {
    pub text: CompactString,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionPoint {
    pub buf_idx: usize,
    pub chunk_idx: usize,
    pub byte: usize,
}

fn col_to_byte(text: &str, col: usize) -> usize {
    let mut w = 0;
    for (pos, c) in text.char_indices() {
        if w >= col {
            return pos;
        }
        w += char_display_width(c);
    }
    text.len()
}

pub struct Renderer {
    lines: u16,
    col: u16,
    spinner_tick: bool,
    buffer: Vec<LineEntry>,
    partial: CompactString,
    partial_color: Color,
    scroll_offset: usize,
    input_scroll_offset: usize,
    monochrome: bool,
    chat_bg: Option<Color>,
    input_bg: Option<Color>,
    status_bg: Option<Color>,
    text_color: Color,
    user_color: Color,
    status_color: Color,
    cursor_style: Option<SetCursorStyle>,
    pub selection_active: bool,
    pub selection_start: Option<SelectionPoint>,
    pub selection_end: Option<SelectionPoint>,
    pub selection_anchor: Option<SelectionPoint>,
    prev_input_height: usize,
    pub show_cursor: bool,
    pub header: Option<LineEntry>,
    pub top_bar: Option<LineEntry>,
    pub picker_height: u16,
}

impl Renderer {
    pub fn new() -> io::Result<Self> {
        Ok(Renderer {
            lines: 0,
            col: 0,
            spinner_tick: false,
            buffer: Vec::new(),
            partial: CompactString::new(""),
            partial_color: Color::White,
            scroll_offset: 0,
            input_scroll_offset: 0,
            monochrome: false,
            chat_bg: None,
            input_bg: None,
            status_bg: None,
            text_color: Color::Reset,
            user_color: Color::Green,
            status_color: Color::Grey,
            cursor_style: Some(SetCursorStyle::SteadyBar),
            selection_active: false,
            selection_start: None,
            selection_end: None,
            selection_anchor: None,
            prev_input_height: 0,
            show_cursor: true,
            header: None,
            top_bar: None,
            picker_height: 0,
        })
    }

    pub fn set_monochrome(&mut self, monochrome: bool) {
        self.monochrome = monochrome;
    }

    pub fn set_background_colors(
        &mut self,
        chat_bg: Option<Color>,
        input_bg: Option<Color>,
        status_bg: Option<Color>,
    ) {
        self.chat_bg = chat_bg;
        self.input_bg = input_bg;
        self.status_bg = status_bg;
        if self.text_color == Color::Reset
            && let Some(bg) = input_bg.or(chat_bg).or(status_bg)
        {
            self.text_color = contrasting_text_color(bg);
        }
    }

    pub fn reset_colors(&mut self) {
        self.chat_bg = None;
        self.input_bg = None;
        self.status_bg = None;
        self.text_color = Color::Reset;
        self.user_color = Color::Green;
        self.status_color = Color::Grey;
    }

    pub fn set_text_color(&mut self, c: Color) {
        self.text_color = c;
    }
    pub fn set_user_color(&mut self, c: Color) {
        self.user_color = c;
    }
    pub fn set_status_color(&mut self, c: Color) {
        self.status_color = c;
    }
    pub fn set_cursor_style(&mut self, style: SetCursorStyle) {
        self.cursor_style = Some(style);
    }

    pub fn text_color(&self) -> Color {
        self.color(self.text_color)
    }
    pub fn user_color(&self) -> Color {
        self.color(self.user_color)
    }

    fn color(&self, color: Color) -> Color {
        let color = if color == Color::Reset && self.text_color != Color::Reset {
            self.text_color
        } else {
            color
        };
        resolve_color(color, self.monochrome)
    }

    fn terminal_size(&self) -> (u16, u16) {
        crossterm::terminal::size().unwrap_or((80, 24))
    }

    fn max_line_width(&self) -> usize {
        let (cols, _) = self.terminal_size();
        cols.saturating_sub(1) as usize
    }

    pub fn line_width(&self) -> usize {
        self.max_line_width()
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn replace_from(&mut self, start: usize, lines: Vec<LineEntry>) {
        self.commit_partial();
        self.buffer.truncate(start);
        self.buffer.extend(lines);
        self.lines = self.buffer.len() as u16;
        self.col = 0;
        self.partial.clear();
        let visible = self.visible_lines();
        let max_offset = self.buffer.len().saturating_sub(visible);
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }

    pub fn visible_lines(&self) -> usize {
        let (_, rows) = self.terminal_size();
        let mut base = rows.saturating_sub(2) as usize;
        if self.header.is_some() || self.top_bar.is_some() {
            base = base.saturating_sub(1);
        }
        base.saturating_sub(self.picker_height as usize)
    }

    fn viewport_start(&self, visible: usize, total: usize) -> usize {
        let start = if self.scroll_offset == 0 {
            total.saturating_sub(visible)
        } else {
            total.saturating_sub(self.scroll_offset + visible)
        };
        start.min(total.saturating_sub(visible))
    }

    pub fn buffer_pos_at_row_col(&self, row: u16, col: u16) -> Option<SelectionPoint> {
        let (cols, _) = self.terminal_size();
        let max_width = cols.saturating_sub(1) as usize;
        let visible = self.visible_lines();
        let total = self.buffer.len();
        if total == 0 {
            return None;
        }

        let start_row = if self.header.is_some() || self.top_bar.is_some() {
            1
        } else {
            0
        };
        if row < start_row {
            return None;
        }
        let adjusted_row = row - start_row;

        let start = self.viewport_start(visible, total);
        let mut visual_row: u16 = 0;
        let mut buf_idx = start;
        while buf_idx < total {
            let text = &self.buffer[buf_idx].text;
            let chunks: SmallVec<[CompactString; 4]> = if display_width(text) > max_width {
                word_wrap(text, max_width)
            } else {
                smallvec![text.clone()]
            };
            for (chunk_idx, chunk) in chunks.iter().enumerate() {
                if visual_row == adjusted_row {
                    let byte = col_to_byte(chunk, col as usize);
                    return Some(SelectionPoint {
                        buf_idx,
                        chunk_idx,
                        byte,
                    });
                }
                visual_row += 1;
                if visual_row as usize >= visible {
                    return None;
                }
            }
            buf_idx += 1;
        }
        None
    }

    fn normalized_selection(&self) -> Option<(&SelectionPoint, &SelectionPoint)> {
        let s = self.selection_start.as_ref()?;
        let e = self.selection_end.as_ref()?;
        if (s.buf_idx, s.chunk_idx, s.byte) <= (e.buf_idx, e.chunk_idx, e.byte) {
            Some((s, e))
        } else {
            Some((e, s))
        }
    }

    fn chunk_selection_range(
        &self,
        buf_idx: usize,
        chunk_idx: usize,
        chunk_len: usize,
    ) -> Option<(usize, usize)> {
        if !self.selection_active {
            return None;
        }
        let (start, end) = self.normalized_selection()?;
        let this = (buf_idx, chunk_idx);
        let sel_start = (start.buf_idx, start.chunk_idx);
        let sel_end = (end.buf_idx, end.chunk_idx);
        if this < sel_start || this > sel_end {
            return None;
        }
        let byte_start = if this == sel_start {
            start.byte.min(chunk_len)
        } else {
            0
        };
        let byte_end = if this == sel_end {
            end.byte.min(chunk_len)
        } else {
            chunk_len
        };
        if byte_start >= byte_end {
            return None;
        }
        Some((byte_start, byte_end))
    }

    pub fn clear_selection(&mut self) {
        self.selection_active = false;
        self.selection_start = None;
        self.selection_end = None;
        self.selection_anchor = None;
    }

    pub fn selected_text(&self) -> Option<String> {
        let (start, end) = self.normalized_selection()?;
        let (cols, _) = self.terminal_size();
        let max_width = cols.saturating_sub(1) as usize;
        let mut result = String::new();
        for buf_idx in start.buf_idx..=end.buf_idx {
            let Some(entry) = self.buffer.get(buf_idx) else {
                continue;
            };
            let text = &entry.text;
            let chunks: SmallVec<[CompactString; 4]> = if display_width(text) > max_width {
                word_wrap(text, max_width)
            } else {
                smallvec![text.clone()]
            };
            let chunk_lo = if buf_idx == start.buf_idx {
                start.chunk_idx
            } else {
                0
            };
            let chunk_hi = if buf_idx == end.buf_idx {
                end.chunk_idx
            } else {
                chunks.len().saturating_sub(1)
            };
            for (chunk_idx, chunk) in chunks.iter().enumerate() {
                if chunk_idx < chunk_lo || chunk_idx > chunk_hi {
                    continue;
                }
                let byte_start = if buf_idx == start.buf_idx && chunk_idx == start.chunk_idx {
                    start.byte.min(chunk.len())
                } else {
                    0
                };
                let byte_end = if buf_idx == end.buf_idx && chunk_idx == end.chunk_idx {
                    end.byte.min(chunk.len())
                } else {
                    chunk.len()
                };
                let slice = &chunk[byte_start..byte_end];
                if !result.is_empty() && !slice.is_empty() {
                    result.push('\n');
                }
                result.push_str(slice);
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn wrap_line(&self, line: &str, max_width: usize) -> SmallVec<[CompactString; 4]> {
        word_wrap(line, max_width)
    }

    fn commit_partial(&mut self) {
        if !self.partial.is_empty() {
            let max_width = self.max_line_width();
            let c = self.partial_color;
            for chunk in self.wrap_line(&self.partial, max_width) {
                self.buffer.push(LineEntry {
                    text: chunk,
                    color: c,
                });
            }
            self.partial.clear();
        }
    }

    pub fn is_scrolling(&self) -> bool {
        self.scroll_offset > 0
    }

    pub fn scroll_line_up(&mut self) {
        let visible = self.visible_lines();
        let max_offset = self.buffer.len().saturating_sub(visible);
        if self.scroll_offset < max_offset {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_line_down(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_page_up(&mut self) {
        let visible = self.visible_lines();
        let page = visible.saturating_sub(2).max(1);
        let max_offset = self.buffer.len().saturating_sub(visible);
        self.scroll_offset = (self.scroll_offset + page).min(max_offset);
    }

    pub fn scroll_page_down(&mut self) {
        let visible = self.visible_lines();
        let page = visible.saturating_sub(2).max(1);
        if self.scroll_offset <= page {
            self.scroll_offset = 0;
        } else {
            self.scroll_offset = self.scroll_offset.saturating_sub(page);
        }
    }

    pub fn scroll_to_top(&mut self) {
        let visible = self.visible_lines();
        self.scroll_offset = self.buffer.len().saturating_sub(visible);
    }

    pub fn scroll_to_bottom(&mut self) -> io::Result<()> {
        self.scroll_offset = 0;
        self.sync_to_buffer()
    }

    fn sync_to_buffer(&mut self) -> io::Result<()> {
        self.commit_partial();
        self.col = 0;
        self.lines = self.buffer.len() as u16;
        self.render_viewport()
    }

    pub fn render_viewport(&mut self) -> io::Result<()> {
        let (cols, _rows) = self.terminal_size();
        let max_width = cols.saturating_sub(1) as usize;
        let visible = self.visible_lines();
        let total = self.buffer.len();
        let mut stdout = io::stdout();
        write!(stdout, "{}", Hide)?;

        let mut visual_row: u16 = 0;

        if let Some(ref header) = self.header {
            stdout.execute(MoveTo(0, 0))?;
            if let Some(bg) = self.status_bg {
                write!(stdout, "{}", SetBackgroundColor(self.color(bg)))?;
            } else {
                write!(stdout, "{}", SetBackgroundColor(self.color(Color::Blue)))?;
            }
            write!(stdout, "{}", SetForegroundColor(self.color(header.color)))?;
            let truncated: String = header.text.chars().take(cols as usize).collect();
            write!(stdout, "{}", truncated)?;
            write!(stdout, "{}", Clear(ClearType::UntilNewLine))?;
            write!(stdout, "{}", ResetColor)?;
            visual_row = 1;
        } else if let Some(ref top) = self.top_bar {
            stdout.execute(MoveTo(0, 0))?;
            let bg = self.color(Color::DarkGrey);
            write!(stdout, "{}", SetBackgroundColor(bg))?;
            write!(stdout, "{}", SetForegroundColor(self.color(top.color)))?;
            let truncated: String = top.text.chars().take(cols as usize).collect();
            write!(stdout, "{}", truncated)?;
            write!(stdout, "{}", Clear(ClearType::UntilNewLine))?;
            write!(stdout, "{}", ResetColor)?;
            visual_row = 1;
        }

        let start = self.viewport_start(visible, total);
        let mut buf_idx = start;
        let limit = (visible
            + if self.header.is_some() || self.top_bar.is_some() {
                1
            } else {
                0
            }) as u16;

        while visual_row < limit && buf_idx < total {
            let entry = &self.buffer[buf_idx];
            let text = &entry.text;

            let wrapped = if display_width(text) > max_width {
                word_wrap(text, max_width)
            } else {
                smallvec![text.clone()]
            };

            for (chunk_idx, chunk) in wrapped.iter().enumerate() {
                if visual_row >= limit {
                    break;
                }

                stdout.execute(MoveTo(0, visual_row))?;

                if let Some(bg) = self.chat_bg {
                    write!(stdout, "{}", SetBackgroundColor(self.color(bg)))?;
                }
                write!(stdout, "{}", SetForegroundColor(self.color(entry.color)))?;

                match self.chunk_selection_range(buf_idx, chunk_idx, chunk.len()) {
                    None => {
                        write!(stdout, "{}", chunk)?;
                    }
                    Some((byte_start, byte_end)) => {
                        let before = &chunk[..byte_start];
                        let selected = &chunk[byte_start..byte_end];
                        let after = &chunk[byte_end..];
                        if !before.is_empty() {
                            write!(stdout, "{}", before)?;
                        }
                        write!(stdout, "{}", SetAttribute(Attribute::Reverse))?;
                        write!(stdout, "{}", selected)?;
                        write!(stdout, "{}", SetAttribute(Attribute::NoReverse))?;
                        if !after.is_empty() {
                            write!(stdout, "{}", after)?;
                        }
                    }
                }

                write!(stdout, "{}", Clear(ClearType::UntilNewLine))?;
                write!(stdout, "{}", ResetColor)?;

                visual_row += 1;
            }

            buf_idx += 1;
        }

        while visual_row < limit {
            stdout.execute(MoveTo(0, visual_row))?;
            write!(stdout, "{}", ResetColor)?;
            if let Some(bg) = self.chat_bg {
                write!(stdout, "{}", SetBackgroundColor(self.color(bg)))?;
            }
            write!(stdout, "{}", Clear(ClearType::UntilNewLine))?;
            write!(stdout, "{}", ResetColor)?;
            visual_row += 1;
        }

        if self.scroll_offset > 0 {
            let pct = if total > visible {
                ((total - self.scroll_offset - visible) * 100 / (total - visible)).min(100)
            } else {
                0
            };
            let indicator = format!(" SCROLL {}% ", pct);
            let x = cols.saturating_sub(indicator.len() as u16);
            let indicator_row = if self.header.is_some() { 1 } else { 0 };
            stdout.execute(MoveTo(x, indicator_row))?;
            if let Some(bg) = self.chat_bg {
                write!(stdout, "{}", SetBackgroundColor(self.color(bg)))?;
            }
            write!(
                stdout,
                "{}",
                SetForegroundColor(self.color(Color::DarkYellow))
            )?;
            write!(stdout, "{}", indicator)?;
            write!(stdout, "{}", ResetColor)?;
        }

        stdout.flush()?;
        Ok(())
    }

    fn ensure_room(&mut self) {
        if self.scroll_offset > 0 {
            return;
        }
        let (cols, rows) = self.terminal_size();
        if rows < 3 {
            return;
        }
        let max_content = rows.saturating_sub(2);
        if self.lines >= max_content {
            let mut stdout = io::stdout();
            let _ = stdout.execute(ScrollUp(1));
            self.lines = self.lines.saturating_sub(1);
            for &r in &[max_content.saturating_sub(1), max_content] {
                let _ = stdout.execute(MoveTo(0, r));
                if let Some(bg) = self.chat_bg {
                    let _ = write!(stdout, "{}", SetBackgroundColor(self.color(bg)));
                }
                let _ = write!(stdout, "{}", " ".repeat(cols as usize));
                let _ = write!(stdout, "{}", ResetColor);
            }
            let _ = stdout.flush();
        }
    }

    fn content_row(&self) -> u16 {
        let (_, rows) = self.terminal_size();
        self.lines.min(rows.saturating_sub(3))
    }

    pub fn resize(&mut self) {
        let visible = self.visible_lines();
        let max_offset = self.buffer.len().saturating_sub(visible);
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }

    pub fn write_line(&mut self, text: &str, color: Color) -> io::Result<()> {
        self.commit_partial();
        let max_width = self.max_line_width();
        for segment in text.split('\n') {
            let wrapped = self.wrap_line(segment, max_width);
            for chunk in &wrapped {
                self.buffer.push(LineEntry {
                    text: chunk.clone(),
                    color,
                });
                if self.scroll_offset == 0 {
                    self.ensure_room();
                    let mut stdout = io::stdout();
                    let r = self.content_row();
                    stdout.execute(MoveTo(0, r))?;
                    if let Some(bg) = self.chat_bg {
                        write!(stdout, "{}", SetBackgroundColor(self.color(bg)))?;
                    }
                    write!(stdout, "{}", Clear(ClearType::CurrentLine))?;
                    write!(stdout, "{}", SetForegroundColor(self.color(color)))?;
                    writeln!(stdout, "{}", chunk)?;
                    write!(stdout, "{}", ResetColor)?;
                    self.lines = self.lines.saturating_add(1);
                    self.col = 0;
                }
            }
        }
        if self.scroll_offset == 0 {
            io::stdout().flush()?;
        }
        Ok(())
    }

    pub fn write(&mut self, text: &str, color: Color) -> io::Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        let max_width = self.max_line_width();
        if max_width == 0 {
            return Ok(());
        }
        let parts: SmallVec<[&str; 4]> = text.split('\n').collect();
        let last = parts.len() - 1;
        for (i, segment) in parts.iter().enumerate() {
            if i < last {
                let len_before = self.buffer.len();
                self.commit_partial();
                let had_content = len_before < self.buffer.len();
                if !segment.is_empty() {
                    self.partial_color = color;
                    self.partial.push_str(segment);
                    self.commit_partial();
                } else if !had_content {
                    self.buffer.push(LineEntry {
                        text: CompactString::new(""),
                        color,
                    });
                }
                if self.scroll_offset == 0 {
                    self.ensure_room();
                    let mut stdout = io::stdout();
                    let r = self.content_row();
                    stdout.execute(MoveTo(self.col, r))?;
                    if let Some(bg) = self.chat_bg {
                        write!(stdout, "{}", SetBackgroundColor(self.color(bg)))?;
                    }
                    if !segment.is_empty() {
                        write!(stdout, "{}", SetForegroundColor(self.color(color)))?;
                        write!(stdout, "{}", segment)?;
                        write!(stdout, "{}", ResetColor)?;
                    }
                    writeln!(stdout)?;
                    self.lines = self.lines.saturating_add(1);
                    self.col = 0;
                }
            } else if !segment.is_empty() {
                let chars: SmallVec<[char; 64]> = segment.chars().collect();
                let mut idx = 0;
                while idx < chars.len() {
                    let avail = max_width.saturating_sub(self.col as usize);
                    if avail == 0 {
                        self.commit_partial();
                        if self.scroll_offset == 0 {
                            self.lines = self.lines.saturating_add(1);
                            self.col = 0;
                        }
                        continue;
                    }
                    // Collect chars that fit within avail display columns
                    let mut end = idx;
                    let mut w: usize = 0;
                    while end < chars.len() {
                        let cw = char_display_width(chars[end]);
                        if w + cw > avail {
                            break;
                        }
                        w += cw;
                        end += 1;
                    }
                    // Try to break at a word boundary
                    if end < chars.len() && end > idx {
                        let mut break_at = end;
                        for i in (idx..end).rev() {
                            if chars[i] == ' ' {
                                break_at = i + 1;
                                break;
                            }
                        }
                        if break_at != idx {
                            end = break_at;
                            // Recalculate width for the shortened chunk
                            w = chars[idx..end].iter().map(|&c| char_display_width(c)).sum();
                        }
                    }
                    let chunk: String = chars[idx..end].iter().collect();
                    self.partial_color = color;
                    self.partial.push_str(&chunk);
                    if self.scroll_offset == 0 {
                        self.ensure_room();
                        let mut stdout = io::stdout();
                        let r = self.content_row();
                        stdout.execute(MoveTo(self.col, r))?;
                        if let Some(bg) = self.chat_bg {
                            write!(stdout, "{}", SetBackgroundColor(self.color(bg)))?;
                        }
                        write!(stdout, "{}", SetForegroundColor(self.color(color)))?;
                        write!(stdout, "{}", chunk)?;
                        write!(stdout, "{}", ResetColor)?;

                        self.col = self.col.saturating_add(w as u16);
                    }
                    idx = end;
                    if idx < chars.len() {
                        self.commit_partial();
                        if self.scroll_offset == 0 {
                            self.lines = self.lines.saturating_add(1);
                            self.col = 0;
                        }
                    }
                }
            }
        }
        if self.scroll_offset == 0 {
            io::stdout().flush()?;
        }
        Ok(())
    }

    pub fn clear_content(&mut self) -> io::Result<()> {
        self.buffer.clear();
        self.partial.clear();
        self.scroll_offset = 0;
        self.clear_selection();
        let mut stdout = io::stdout();
        if let Some(bg) = self.chat_bg {
            write!(stdout, "{}", SetBackgroundColor(self.color(bg)))?;
        }
        stdout.execute(Clear(ClearType::All))?;
        write!(stdout, "{}", ResetColor)?;
        stdout.execute(MoveTo(0, 0))?;
        stdout.flush()?;
        self.lines = 0;
        self.col = 0;
        Ok(())
    }

    pub fn draw_bottom(
        &mut self,
        input_line: &str,
        cursor_pos: usize,
        status: &str,
        is_running: bool,
    ) -> io::Result<()> {
        let (cols, rows) = crossterm::terminal::size()?;
        let mut stdout = io::stdout();

        let status_row = rows.saturating_sub(1);

        let lines: SmallVec<[&str; 4]> = input_line.split('\n').collect();
        let line_count = lines.len();

        let last_line = rows.saturating_sub(2) as usize - 1;
        let available_rows = last_line + 1;
        let need_scroll = line_count > available_rows;
        let first_visible = if need_scroll {
            line_count - available_rows
        } else {
            0
        };

        let prompt = if is_running {
            self.spinner_tick = !self.spinner_tick;
            if self.spinner_tick { ". " } else { ": " }
        } else {
            "> "
        };
        let prompt_width = display_width(prompt);

        let (cursor_line, cursor_col) =
            crate::ui::input::cursor_to_line_col(input_line, cursor_pos);

        let visible_width = cols.saturating_sub(prompt_width as u16) as usize;
        let cursor_line_text = lines.get(cursor_line).unwrap_or(&"");

        // Convert cursor char-index to display column
        let cursor_byte = cursor_line_text
            .char_indices()
            .nth(cursor_col)
            .map(|(i, _)| i)
            .unwrap_or(cursor_line_text.len());
        let cursor_display_col = display_width(&cursor_line_text[..cursor_byte]);

        let cursor_line_len = display_width(cursor_line_text);
        let mut h_scroll = 0usize;
        if cursor_line_len > visible_width {
            if cursor_display_col < self.input_scroll_offset {
                self.input_scroll_offset = cursor_display_col;
            } else if cursor_display_col >= self.input_scroll_offset + visible_width {
                self.input_scroll_offset = cursor_display_col - visible_width + 1;
            }
            let max_h_scroll = cursor_line_len.saturating_sub(visible_width);
            h_scroll = self.input_scroll_offset.min(max_h_scroll);
        } else {
            self.input_scroll_offset = 0;
        }

        // Clear and draw input area
        let visible_line_count = if need_scroll {
            available_rows
        } else {
            line_count
        };

        if visible_line_count < self.prev_input_height {
            let old_start = rows.saturating_sub(2) - self.prev_input_height as u16 + 1;
            let new_start = rows.saturating_sub(2) - visible_line_count as u16 + 1;
            for row in old_start..new_start {
                stdout.execute(MoveTo(0, row))?;
                if let Some(bg) = self.input_bg {
                    write!(stdout, "{}", SetBackgroundColor(self.color(bg)))?;
                }
                write!(stdout, "{}", Clear(ClearType::UntilNewLine))?;
                write!(stdout, "{}", ResetColor)?;
            }
        }
        self.prev_input_height = visible_line_count;

        for (i, line) in lines
            .iter()
            .enumerate()
            .take(line_count)
            .skip(first_visible)
        {
            let render_row = (rows.saturating_sub(2) - visible_line_count as u16 + 1)
                + (i - first_visible) as u16;
            stdout.execute(MoveTo(0, render_row))?;

            if let Some(bg) = self.input_bg {
                write!(stdout, "{}", SetBackgroundColor(self.color(bg)))?;
            }

            if i == first_visible {
                write!(stdout, "{}", SetForegroundColor(self.color(Color::Cyan)))?;
                write!(stdout, "{}", prompt)?;
                write!(stdout, "{}", SetForegroundColor(self.color(Color::Reset)))?;
            } else {
                write!(stdout, "{}", " ".repeat(prompt_width))?;
            }

            let line_chars: SmallVec<[char; 64]> = line.chars().collect();
            // Skip chars to reach display column h_scroll, then take enough to fill visible_width
            let skip_chars: usize = if i == cursor_line {
                let mut w = 0usize;
                let mut skip = 0usize;
                for &ch in &line_chars {
                    let cw = char_display_width(ch);
                    if w + cw > h_scroll {
                        break;
                    }
                    w += cw;
                    skip += 1;
                }
                skip
            } else {
                0
            };
            let display: String = line_chars
                .iter()
                .skip(skip_chars)
                .take(visible_width)
                .collect();
            write!(stdout, "{}", SetForegroundColor(self.color(Color::Reset)))?;
            write!(stdout, "{}", display)?;
            write!(stdout, "{}", Clear(ClearType::UntilNewLine))?;
            write!(stdout, "{}", ResetColor)?;
        }

        // Status line
        stdout.execute(MoveTo(0, status_row))?;
        let bg_color = self.status_bg.unwrap_or(Color::DarkGrey);
        write!(stdout, "{}", SetBackgroundColor(self.color(bg_color)))?;
        write!(stdout, "{}", Clear(ClearType::CurrentLine))?;
        stdout.execute(MoveTo(0, status_row))?;
        write!(stdout, "{}", SetBackgroundColor(self.color(bg_color)))?;
        write!(
            stdout,
            "{}",
            SetForegroundColor(self.color(self.status_color))
        )?;
        let status_display = if self.scroll_offset > 0 {
            format!("-- SCROLL -- {}", status)
        } else {
            status.to_string()
        };
        let truncated: String = status_display.chars().take(cols as usize).collect();
        write!(stdout, "{}", truncated)?;
        write!(stdout, "{}", Clear(ClearType::UntilNewLine))?;
        write!(stdout, "{}", ResetColor)?;

        // Cursor
        if self.show_cursor && !is_running {
            let cursor_render_idx = cursor_line.saturating_sub(first_visible);
            let cursor_row =
                (rows.saturating_sub(2) - visible_line_count as u16 + 1) + cursor_render_idx as u16;
            let cursor_x = (prompt_width + cursor_display_col.saturating_sub(h_scroll)) as u16;
            stdout.execute(MoveTo(cursor_x, cursor_row))?;
            write!(stdout, "{}", Show)?;
            if let Some(style) = self.cursor_style {
                write!(stdout, "{}", style)?;
            }
        } else {
            write!(stdout, "{}", Hide)?;
        }
        stdout.flush()?;
        Ok(())
    }
}

pub(crate) fn contrasting_text_color(bg: Color) -> Color {
    let luminance = match bg {
        Color::Rgb { r, g, b } => {
            (u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114) / 1000
        }
        Color::Black => 0,
        Color::DarkGrey => 64,
        Color::DarkRed
        | Color::DarkGreen
        | Color::DarkYellow
        | Color::DarkBlue
        | Color::DarkMagenta
        | Color::DarkCyan => 96,
        Color::Red | Color::Green | Color::Yellow | Color::Blue | Color::Magenta | Color::Cyan => {
            160
        }
        Color::Grey => 192,
        Color::White => 255,
        Color::AnsiValue(n) if n < 16 => {
            const TABLE: [u32; 16] = [
                0, 128, 128, 128, 128, 128, 128, 192, 128, 255, 255, 255, 255, 255, 255, 255,
            ];
            TABLE[n as usize]
        }
        Color::AnsiValue(_) => 128,
        Color::Reset => return Color::Reset,
    };

    if luminance >= 128 {
        Color::Black
    } else {
        Color::White
    }
}

pub fn copy_to_clipboard(text: &str) {
    tracing::debug!(bytes = text.len(), text = text, "copy_to_clipboard");
    let on_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
    // xclip writes to the X11 clipboard which Wayland-native terminals don't read,
    // and child.wait() would block until another app claims the clipboard.
    // Skip it on Wayland and let OSC 52 handle it instead.
    let cmds: &[(&str, &[&str])] = if on_wayland {
        &[("wl-copy", &[]), ("pbcopy", &[]), ("clip.exe", &[])]
    } else {
        &[
            ("wl-copy", &[]),
            ("xclip", &["-selection", "clipboard"]),
            ("pbcopy", &[]),
            ("clip.exe", &[]),
        ]
    };
    for &(cmd, args) in cmds {
        match std::process::Command::new(cmd)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                tracing::debug!("copy_to_clipboard: using {}", cmd);
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                    let _ = stdin.flush();
                }
                // Don't wait — tools like xclip stay alive to serve clipboard requests.
                drop(child);
                tracing::debug!("copy_to_clipboard: {} done", cmd);
                return;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::debug!("copy_to_clipboard: {} failed: {}", cmd, e);
            }
        }
    }

    // OSC 52 escape sequence — clipboard access via terminal emulator.
    // Supported by Kitty, Alacritty, WezTerm, foot, iTerm2, Windows Terminal,
    // and most other modern terminals. No external tools needed.
    tracing::debug!("copy_to_clipboard: falling back to OSC 52");
    let encoded = base64_encode(text.as_bytes());
    let mut stdout = std::io::stdout().lock();
    let _ = write!(stdout, "\x1b]52;c;{encoded}\x07");
    let _ = stdout.flush();
    tracing::debug!("copy_to_clipboard: OSC 52 written");
}

/// Minimal base64 encoder — avoids pulling in a crate just for clipboard support.
pub(crate) fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(triple >> 18) & 63] as char);
        out.push(ALPHABET[(triple >> 12) & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(triple >> 6) & 63]
        } else {
            b'='
        } as char);
        out.push(if chunk.len() > 2 {
            ALPHABET[triple & 63]
        } else {
            b'='
        } as char);
    }
    out
}
