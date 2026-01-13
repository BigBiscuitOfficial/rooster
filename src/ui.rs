use crate::app::{AppState, EditorMode, MainScreenElement};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

// Refactored UI separation below
pub fn ui(f: &mut Frame, app: &AppState) {
    // Split the screen into two chunks (Top for URL, bottom is headers, body, response)
    // This will change dramatically once multitasking is supported
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

    let url_input = Paragraph::new(app.url_buffer.content.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" URL (Press 'q' to quit) ")
            .border_style(style_for(app.selected_section, MainScreenElement::Url)),
    );

    // Render a simple box for the Body/Response
    let response_body = Block::default()
        .borders(Borders::ALL)
        .title(" Response ")
        .border_style(style_for(app.selected_section, MainScreenElement::Response));

    let headers = Block::default()
        .borders(Borders::ALL)
        .title(" Headers ")
        .border_style(style_for(app.selected_section, MainScreenElement::Headers));

    let payload = Block::default()
        .borders(Borders::ALL)
        .title(" Body ")
        .border_style(style_for(app.selected_section, MainScreenElement::Body));

    f.render_widget(url_input, chunks[0]);
    f.render_widget(response_body, body_h_res[1]);
    f.render_widget(headers, h_body[0]);
    f.render_widget(payload, h_body[1]);

    if app.mode == EditorMode::Edit
        && let Some(p) = app.get_editor()
    {
        let area = match app.selected_section {
            MainScreenElement::Url => centered_rect_fixed(80, 5, f.area()),

            // Keep percentage scaling
            MainScreenElement::Body | MainScreenElement::Headers | MainScreenElement::Response => {
                centered_rect(60, 80, f.area())
            }
        };

        f.render_widget(p, area);
    }
}

fn style_for(selected: MainScreenElement, element: MainScreenElement) -> Style {
    let selected_style = Style::default().fg(Color::Yellow);
    let normal_style = Style::default().fg(Color::DarkGray);

    if selected == element {
        selected_style
    } else {
        normal_style
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

// Just a helper for popup editor modal
fn centered_rect_fixed(percent_x: u16, height: u16, r: Rect) -> Rect {
    // Calculate vertical centering with fixed height
    let split_v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(r);

    // Calculate horizontal centering with percentage
    let split_h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(split_v[1]);

    split_h[1]
}
