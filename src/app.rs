use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use serde_json::Value;
use std::{collections::HashMap, io};

use crate::tui::{self};
// Yuck

#[allow(unused)]
pub struct AppState {
    pub selected_section: MainScreenElement,
    pub current_url: String,
    pub last_response: String,
    pub current_headers: Option<HashMap<String, String>>,
    pub current_body: Option<Value>,
    pub mode: EditorMode,
    terminal: tui::Tui,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EditorMode {
    View,
    Edit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MainScreenElement {
    Url,
    Headers,
    Response,
    Body,
}

impl AppState {
    pub fn new(terminal: tui::Tui) -> Self {
        Self {
            selected_section: MainScreenElement::Url,
            current_url: String::new(),
            last_response: String::new(),
            current_headers: None,
            current_body: None,
            mode: EditorMode::View,
            terminal, //intentional unwrap
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

    pub async fn run(&mut self) -> io::Result<()> {
        self.terminal.clear()?;
        loop {
            self.terminal.draw(|f| {
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
                                self.selected_section.clone(),
                                MainScreenElement::Url,
                            )),
                    );

                // Render a simple box for the Body/Response

                let response_body = Block::default()
                    .borders(Borders::ALL)
                    .title(" Response ")
                    .border_style(Self::style_for(
                        self.selected_section.clone(),
                        MainScreenElement::Response,
                    ));
                let headers = Block::default()
                    .borders(Borders::ALL)
                    .title(" Headers ")
                    .border_style(Self::style_for(
                        self.selected_section.clone(),
                        MainScreenElement::Headers,
                    ));

                let payload = Block::default()
                    .borders(Borders::ALL)
                    .title(" Body ")
                    .border_style(Self::style_for(
                        self.selected_section.clone(),
                        MainScreenElement::Body,
                    ));

                f.render_widget(url_input, chunks[0]);
                f.render_widget(response_body, body_h_res[1]);
                f.render_widget(headers, h_body[0]);
                f.render_widget(payload, h_body[1]);
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
}
