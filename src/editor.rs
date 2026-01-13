use derive_setters::Setters;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Widget, Wrap},
};

#[allow(unused)]
#[derive(Debug, Setters)]
pub struct Popup<'a> {
    #[setters(into)]
    title: Line<'a>,
    #[setters(into)]
    content: Text<'a>,
    border_style: Style,
    title_style: Style,
    style: Style,
    border_type: BorderType,
}

impl Default for Popup<'_> {
    fn default() -> Self {
        Self {
            title: Line::default(),
            content: Text::default(),
            border_style: Style::default(),
            title_style: Style::default(),
            style: Style::default(),
            border_type: BorderType::Rounded,
        }
    }
}

impl Widget for Popup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Ensure that all cells under the popup are cleared to avoid leaking content
        Clear.render(area, buf);
        let block = Block::new()
            .title(self.title)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style)
            .border_type(self.border_type)
            .padding(Padding::new(1, 1, 1, 1)); // Internal padding
            
        Paragraph::new(self.content)
            .wrap(Wrap { trim: false })
            .style(self.style)
            .block(block)
            .render(area, buf);
    }
}
