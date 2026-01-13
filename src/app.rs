use crate::editor::Popup;
use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::BorderType,
};
use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq)]
pub enum EditorMode {
    View,
    Edit,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum MainScreenElement {
    Url,
    Headers,
    Response,
    Body,
}

pub struct TextBuffer {
    pub content: String,
    pub cursor_idx: usize, // Character Index, not byte index!
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor_idx: 0,
        }
    }

    pub fn from_string(s: String) -> Self {
        let len = s.chars().count();
        Self {
            content: s,
            cursor_idx: len,
        }
    }

    pub fn insert(&mut self, c: char) {
        let len_chars = self.content.chars().count();
        if self.cursor_idx >= len_chars {
            self.content.push(c);
            self.cursor_idx = len_chars + 1;
        } else {
            let byte_idx = self
                .content
                .chars()
                .take(self.cursor_idx)
                .map(|c| c.len_utf8())
                .sum();
            self.content.insert(byte_idx, c);
            self.cursor_idx += 1;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor_idx > 0 {
            let prev_idx = self.cursor_idx - 1;
            let byte_idx: usize = self
                .content
                .chars()
                .take(prev_idx)
                .map(|c| c.len_utf8())
                .sum();

            // Should be safe if logic holds, but remove panics if boundary wrong
            if byte_idx < self.content.len() {
                self.content.remove(byte_idx);
                self.cursor_idx -= 1;
            }
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_idx > 0 {
            self.cursor_idx -= 1;
        }
    }

    pub fn move_right(&mut self) {
        let len = self.content.chars().count();
        if self.cursor_idx < len {
            self.cursor_idx += 1;
        }
    }

    pub fn move_up(&mut self) {
        todo!()
    }

    // Helper to render with cursor
    pub fn render_with_cursor<'a>(&self) -> Text<'a> {
        let mut lines = Vec::new();
        let mut current_line = Vec::new();

        let char_count = self.content.chars().count();

        for (i, c) in self.content.chars().enumerate() {
            let is_cursor = i == self.cursor_idx;
            let style = if is_cursor {
                Style::default().bg(Color::White).fg(Color::Black)
            } else {
                Style::default()
            };

            // editor cursor fix
            if c == '\n' {
                if is_cursor {
                    current_line.push(Span::styled(" ", style));
                }
                lines.push(Line::from(current_line));
                current_line = Vec::new();
                continue;
            }

            current_line.push(Span::styled(c.to_string(), style));
        }

        // Handle cursor at end of the buffer
        if self.cursor_idx == char_count {
            current_line.push(Span::styled(
                " ",
                Style::default().bg(Color::White).fg(Color::Black),
            ));
        }

        lines.push(Line::from(current_line));

        Text::from(lines)
    }
}

pub struct BodyManager {
    pub buffer: TextBuffer,
    pub is_valid: bool,
}

impl BodyManager {
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            is_valid: true,
        }
    }

    pub fn update(&mut self, c: char) {
        self.buffer.insert(c);
        self.validate();
    }

    pub fn pop(&mut self) {
        self.buffer.delete();
        self.validate();
    }

    pub fn validate(&mut self) {
        if self.buffer.content.trim().is_empty() {
            self.is_valid = true;
            return;
        }
        let v: Result<Value, _> = serde_json::from_str(&self.buffer.content);
        self.is_valid = v.is_ok();
    }
}

pub struct HeaderManager {
    pub headers: Vec<(TextBuffer, TextBuffer)>, // Key Buffer, Value Buffer
    pub selected_index: usize,
    pub editing_value: bool, // false = key, true = value
}

impl HeaderManager {
    pub fn new() -> Self {
        Self {
            headers: vec![(
                TextBuffer::from_string("Content-Type".to_string()),
                TextBuffer::from_string("application/json".to_string()),
            )],
            selected_index: 0,
            editing_value: false,
        }
    }

    pub fn add_new(&mut self) {
        self.headers.push((TextBuffer::new(), TextBuffer::new()));
        self.selected_index = self.headers.len() - 1;
        self.editing_value = false;
    }

    pub fn remove_current(&mut self) {
        if self.headers.len() > 0 {
            self.headers.remove(self.selected_index);
            if self.selected_index >= self.headers.len() && self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }
    }

    // Helper to get active buffer
    pub fn get_active_buffer_mut(&mut self) -> Option<&mut TextBuffer> {
        if self.headers.is_empty() {
            return None;
        }
        let pair = &mut self.headers[self.selected_index];
        if self.editing_value {
            Some(&mut pair.1)
        } else {
            Some(&mut pair.0)
        }
    }
}

pub struct AppState {
    pub selected_section: MainScreenElement,
    pub url_buffer: TextBuffer,
    pub last_response: String,

    // New Managers
    pub header_manager: HeaderManager,
    pub body_manager: BodyManager,

    pub current_body_json: Value,

    pub mode: EditorMode,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            selected_section: MainScreenElement::Url,
            url_buffer: TextBuffer::new(),
            last_response: String::new(),
            header_manager: HeaderManager::new(),
            body_manager: BodyManager::new(),
            current_body_json: json!(""),
            mode: EditorMode::View,
        }
    }

    // ... Navigation logic (left/right/up/down) remains largely same for View mode ...
    pub fn left(&mut self) {
        if self.selected_section == MainScreenElement::Response {
            self.selected_section = MainScreenElement::Headers;
        } else if self.mode == EditorMode::Edit
            && self.selected_section == MainScreenElement::Headers
        {
            if self.header_manager.editing_value {
                self.header_manager.editing_value = false;
            }
        }
    }

    pub fn right(&mut self) {
        if self.selected_section != MainScreenElement::Response
            && self.selected_section != MainScreenElement::Url
            && self.mode == EditorMode::View
        {
            self.selected_section = MainScreenElement::Response
        } else if self.mode == EditorMode::Edit
            && self.selected_section == MainScreenElement::Headers
        {
            if !self.header_manager.editing_value {
                self.header_manager.editing_value = true;
            }
        }
    }

    pub fn up(&mut self) {
        if self.mode == EditorMode::Edit && self.selected_section == MainScreenElement::Headers {
            if self.header_manager.selected_index > 0 {
                self.header_manager.selected_index -= 1;
            }
            return;
        }

        if self.selected_section == MainScreenElement::Url {
        } else if self.selected_section == MainScreenElement::Response
            || self.selected_section == MainScreenElement::Headers
        {
            self.selected_section = MainScreenElement::Url;
        } else {
            self.selected_section = MainScreenElement::Headers;
        }
    }

    pub fn down(&mut self) {
        if self.mode == EditorMode::Edit && self.selected_section == MainScreenElement::Headers {
            if self.header_manager.selected_index
                < self.header_manager.headers.len().saturating_sub(1)
            {
                self.header_manager.selected_index += 1;
            }
            return;
        }

        if self.selected_section == MainScreenElement::Response
            || self.selected_section == MainScreenElement::Body
        {
        } else if self.selected_section == MainScreenElement::Url {
            self.selected_section = MainScreenElement::Headers;
        } else {
            self.selected_section = MainScreenElement::Body;
        }
    }

    pub fn toggle_edit(&mut self) {
        if self.mode == EditorMode::View {
            self.mode = EditorMode::Edit
        } else {
            self.mode = EditorMode::View
        }
    }

    pub fn get_editor(&self) -> Option<Popup<'_>> {
        match self.selected_section {
            MainScreenElement::Url => Some(self.get_url_popup()),
            MainScreenElement::Headers => Some(self.get_headers_popup()),
            MainScreenElement::Body => Some(self.get_body_popup()),
            _ => None,
        }
    }

    pub fn delete_char(&mut self) {
        match self.selected_section {
            MainScreenElement::Url => {
                self.url_buffer.delete();
            }
            MainScreenElement::Body => {
                self.body_manager.pop();
            }
            MainScreenElement::Headers => {
                if let Some(buf) = self.header_manager.get_active_buffer_mut() {
                    buf.delete();
                }
            }
            _ => {}
        }
    }

    pub fn capture_char(&mut self, keycode: KeyCode) {
        let c_opt = match keycode {
            KeyCode::Char(c) => Some(c),
            KeyCode::Enter => Some('\n'),
            _ => None,
        };

        if let Some(c) = c_opt {
            match self.selected_section {
                MainScreenElement::Url => {
                    // Prevent newline in URL if preferred? Allow for now.
                    // Actually usually URLs are one line.
                    if c == '\n' {
                        return;
                    }
                    self.url_buffer.insert(c);
                }
                MainScreenElement::Body => {
                    self.body_manager.update(c);
                }
                MainScreenElement::Headers => {
                    // CRITICAL FIX: Do NOT allow newlines in headers
                    if c == '\n' {
                        return;
                    }

                    if self.header_manager.headers.is_empty() {
                        self.header_manager.add_new();
                    }
                    if let Some(buf) = self.header_manager.get_active_buffer_mut() {
                        buf.insert(c);
                    }
                }
                _ => {}
            }
        }
    }

    // Header Actions
    pub fn add_new_header(&mut self) {
        self.header_manager.add_new();
    }

    pub fn remove_current_header(&mut self) {
        self.header_manager.remove_current();
    }

    // Cursor Movement
    pub fn cursor_left(&mut self) {
        match self.selected_section {
            MainScreenElement::Url => self.url_buffer.move_left(),
            MainScreenElement::Body => self.body_manager.buffer.move_left(),
            MainScreenElement::Headers => {
                if let Some(buf) = self.header_manager.get_active_buffer_mut() {
                    buf.move_left();
                }
            }
            _ => {}
        }
    }

    pub fn cursor_right(&mut self) {
        match self.selected_section {
            MainScreenElement::Url => self.url_buffer.move_right(),
            MainScreenElement::Body => self.body_manager.buffer.move_right(),
            MainScreenElement::Headers => {
                if let Some(buf) = self.header_manager.get_active_buffer_mut() {
                    buf.move_right();
                }
            }
            _ => {}
        }
    }

    fn get_url_popup(&self) -> Popup<'_> {
        Popup::default()
            .content(self.url_buffer.render_with_cursor())
            .style(Style::default().fg(Color::White))
            .title(" Edit URL (Esc to Exit) ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Color::Cyan))
            .border_type(BorderType::Rounded)
    }

    fn get_headers_popup(&self) -> Popup<'_> {
        let mut text = Text::default();

        for (i, (key_buf, val_buf)) in self.header_manager.headers.iter().enumerate() {
            let mut line_spans = Vec::new();

            // Selection indicator
            if i == self.header_manager.selected_index {
                line_spans.push(Span::styled("> ", Style::default().fg(Color::Yellow)));
            } else {
                line_spans.push(Span::raw("  "));
            }

            if i == self.header_manager.selected_index && !self.header_manager.editing_value {
                let t = key_buf.render_with_cursor();
                if let Some(first_line) = t.lines.first() {
                    line_spans.extend(first_line.spans.clone());
                }
            } else {
                line_spans.push(Span::raw(&key_buf.content));
            }

            line_spans.push(Span::raw(" : "));

            // Value
            if i == self.header_manager.selected_index && self.header_manager.editing_value {
                let t = val_buf.render_with_cursor();
                if let Some(first_line) = t.lines.first() {
                    line_spans.extend(first_line.spans.clone());
                }
            } else {
                line_spans.push(Span::raw(&val_buf.content));
            }

            text.lines.push(Line::from(line_spans));
        }

        let instructions = "\n(Arrows: Nav, Tab: New, Ctrl+D: Delete)";
        text.lines.push(Line::from(instructions));

        Popup::default()
            .content(text)
            .style(Style::default().fg(Color::White))
            .title(" Edit Headers ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Color::Cyan))
            .border_type(BorderType::Rounded)
    }

    fn get_body_popup(&self) -> Popup<'_> {
        let border_color = if self.body_manager.is_valid {
            Color::Green
        } else {
            Color::Red
        };
        let title = if self.body_manager.is_valid {
            " Edit Body (Valid JSON) "
        } else {
            " Edit Body (INVALID JSON) "
        };

        Popup::default()
            .content(self.body_manager.buffer.render_with_cursor())
            .style(Style::default().fg(Color::White))
            .title(title)
            .title_style(
                Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(border_color))
            .border_type(BorderType::Rounded)
    }
}
