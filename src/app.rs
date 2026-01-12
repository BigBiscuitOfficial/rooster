use crate::{editor::Popup, tui::Tui};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use serde_json::{Value, json};
use std::{collections::HashMap, io};

#[allow(unused)]
pub struct AppState {
    pub selected_section: MainScreenElement,
    pub current_url: String,
    pub last_response: String,
    pub current_headers: HashMap<String, String>,
    pub current_body: Value,
    pub mode: EditorMode,
}

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

impl AppState {
    pub fn new() -> Self {
        Self {
            selected_section: MainScreenElement::Url,
            current_url: String::new(),
            last_response: String::new(),
            current_headers: HashMap::new(),
            current_body: json!(""),
            mode: EditorMode::View,
        }
    }

    pub fn left(&mut self) {
        if self.selected_section == MainScreenElement::Response {
            self.selected_section = MainScreenElement::Headers;
        }
    }

    pub fn right(&mut self) {
        if self.selected_section != MainScreenElement::Response
            && self.selected_section != MainScreenElement::Url
        {
            self.selected_section = MainScreenElement::Response
        }
    }

    pub fn up(&mut self) {
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
    fn get_editor(&self) -> Option<Popup<'_>> {
        match self.selected_section {
            MainScreenElement::Url => Some(self.get_url_popup()),
            MainScreenElement::Headers => Some(self.get_headers_popup()),
            MainScreenElement::Body => Some(self.get_body_popup()),
            _ => None,
        }
    }

    pub async fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        terminal.clear()?;
        loop {
            let mut popup = None;
            if self.mode == EditorMode::Edit {
                popup = self.get_editor();
            }

            terminal.draw(|f| {
                // Split the screen into two chunks (Top for Input, Bottom for Output)
                // For reference, these are considered nested layouts.
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // 3 lines high for input
                        Constraint::Min(1),    // The rest for output
                    ])
                    .split(f.area());

                let body_h_res = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
                    .split(chunks[1]);
                let h_body = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(body_h_res[0]);

                // Render a simple box for the URL input

                let url_input = Paragraph::new("https://jsonplaceholder.typicode.com/todos/1")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" URL (Press 'q' to quit) ")
                            .border_style(Self::style_for(
                                self.selected_section,
                                MainScreenElement::Url,
                            )),
                    );

                // Render a simple box for the Body/Response

                let response_body = Block::default()
                    .borders(Borders::ALL)
                    .title(" Response ")
                    .border_style(Self::style_for(
                        self.selected_section,
                        MainScreenElement::Response,
                    ));
                let headers = Block::default()
                    .borders(Borders::ALL)
                    .title(" Headers ")
                    .border_style(Self::style_for(
                        self.selected_section,
                        MainScreenElement::Headers,
                    ));

                let payload = Block::default()
                    .borders(Borders::ALL)
                    .title(" Body ")
                    .border_style(Self::style_for(
                        self.selected_section,
                        MainScreenElement::Body,
                    ));

                f.render_widget(url_input, chunks[0]);
                f.render_widget(response_body, body_h_res[1]);
                f.render_widget(headers, h_body[0]);
                f.render_widget(payload, h_body[1]);

                if let Some(p) = popup {
                    let area = centered_rect(60, 20, f.area());

                    f.render_widget(p, area);
                }
            })?;

            if event::poll(std::time::Duration::from_millis(16))?
                && let Event::Key(key) = event::read()?
            {
                // This is so bad. Like bad.
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => self.up(),
                    KeyCode::Down => {
                        self.down();
                    }
                    KeyCode::Left => {
                        self.left();
                    }
                    KeyCode::Right => {
                        self.right();
                    }
                    KeyCode::Enter => {
                        self.toggle_edit();
                    }
                    _ => continue,
                }
            }
        }
    }
    fn style_for(selected: MainScreenElement, element: MainScreenElement) -> Style {
        let selected_style = Style::default().fg(Color::Yellow);
        let normal_style = Style::default().fg(Color::DarkGray);

        if selected == element {
            selected_style // break my computer
        } else {
            normal_style
        }
    }

    fn get_url_popup(&self) -> Popup<'_> {
        Popup::default()
            .content(self.current_url.clone())
            .style(Style::new().yellow())
            .title("Edit Your Url:")
            .title_style(Style::new().white().bold())
            .border_style(Style::new().red())
    }
    fn get_headers_popup(&self) -> Popup<'_> {
        let mut headerstring: String = String::new();
        for (k, v) in &self.current_headers {
            headerstring.push_str(&format!("{} : {}\n", &k, &v));
        }
        Popup::default()
            .content(headerstring)
            .style(Style::new().yellow())
            .title("Edit Headers:")
            .title_style(Style::new().white().bold())
            .border_style(Style::new().red())
    }
    fn get_body_popup(&self) -> Popup<'_> {
        Popup::default()
            .content("")
            .style(Style::new().yellow())
            .title("Edit JSON Body:")
            .title_style(Style::new().white().bold())
            .border_style(Style::new().red())
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
