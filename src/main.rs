mod requests;
mod tui;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use serde_json::{Value, json};
use std::{collections::HashMap, io};

// Yuck
#[derive(Clone, Debug, PartialEq)]
struct AppState {
    selected_section: MainScreenElement,
    current_url: String,
    last_response: String,
    current_headers: HashMap<String, String>,
    current_body: Value,
    mode: EditorMode,
}

#[derive(Clone, Debug, PartialEq)]
enum MainScreenElement {
    Url,
    Headers,
    Response,
    Body,
}

#[derive(Clone, Debug, PartialEq)]
enum EditorMode {
    View,
    Edit,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = tui::init()?;

    // Constructor later
    let mut app = AppState {
        selected_section: MainScreenElement::Url,
        current_url: "https://example.com/api/v1".to_string(),
        last_response: "".to_string(),
        current_headers: HashMap::new(),
        current_body: json!(""),
        mode: EditorMode::View,
    };

    let res = run_app(&mut terminal, &mut app).await;

    tui::restore()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

// It would be simpler to handle this with a virtualized grid or something. Just going quick for
// now.
fn get_next_selection(selected: MainScreenElement, direction: KeyCode) -> MainScreenElement {
    if selected == MainScreenElement::Url {
        match direction {
            KeyCode::Down => MainScreenElement::Headers,
            _ => MainScreenElement::Url,
        }
    } else if selected == MainScreenElement::Headers {
        match direction {
            KeyCode::Down => MainScreenElement::Body,
            KeyCode::Up => MainScreenElement::Url,
            KeyCode::Right => MainScreenElement::Response,
            _ => MainScreenElement::Headers,
        }
    } else if selected == MainScreenElement::Body {
        match direction {
            KeyCode::Right => MainScreenElement::Response,
            KeyCode::Up => MainScreenElement::Headers,
            _ => MainScreenElement::Body,
        }
    } else {
        match direction {
            KeyCode::Up => MainScreenElement::Url,
            KeyCode::Left => MainScreenElement::Headers,
            _ => MainScreenElement::Response,
        }
    }
}

// Change to explicit lifetimes to reduce clone overhead.
fn style_for(selected: MainScreenElement, element: MainScreenElement) -> Style {
    let selected_style = Style::default().fg(Color::Yellow);
    let normal_style = Style::default().fg(Color::DarkGray);

    if selected == element {
        selected_style // break my computer
    } else {
        normal_style
    }
}

// TO DO: Keep track of state
// Task 1 - Upon open, have the URL Auto hovered. This spell checker is ticking me off.
async fn run_app(terminal: &mut tui::Tui, app: &mut AppState) -> io::Result<()> {
    loop {
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
            let url_input = Paragraph::new("https://jsonplaceholder.typicode.com/todos/1").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" URL (Press 'q' to quit) ")
                    .border_style(style_for(
                        app.selected_section.clone(),
                        MainScreenElement::Url,
                    )),
            );

            // Render a simple box for the Body/Response

            let response_body = Block::default()
                .borders(Borders::ALL)
                .title(" Response ")
                .border_style(style_for(
                    app.selected_section.clone(),
                    MainScreenElement::Response,
                ));
            let headers = Block::default()
                .borders(Borders::ALL)
                .title(" Headers ")
                .border_style(style_for(
                    app.selected_section.clone(),
                    MainScreenElement::Headers,
                ));
            let payload = Block::default()
                .borders(Borders::ALL)
                .title(" Body ")
                .border_style(style_for(
                    app.selected_section.clone(),
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
                KeyCode::Up => {
                    app.selected_section =
                        get_next_selection(app.selected_section.clone(), key.code)
                }
                KeyCode::Down => {
                    app.selected_section =
                        get_next_selection(app.selected_section.clone(), key.code)
                }
                KeyCode::Left => {
                    app.selected_section =
                        get_next_selection(app.selected_section.clone(), key.code)
                }
                KeyCode::Right => {
                    app.selected_section =
                        get_next_selection(app.selected_section.clone(), key.code)
                }
                KeyCode::Enter => {
                    if app.selected_section != MainScreenElement::Response
                        && app.mode == EditorMode::View
                    {
                        app.mode = EditorMode::Edit;
                    } else {
                        app.mode = EditorMode::View
                    }
                }
                _ => continue,
            }
        }
        // Handle Exit
        //if event::poll(std::time::Duration::from_millis(16))?
        //    && let Event::Key(key) = event::read()?
        //    && key.code == KeyCode::Char('q')
        //{
        //    return OK(());
        //}
    }
}
