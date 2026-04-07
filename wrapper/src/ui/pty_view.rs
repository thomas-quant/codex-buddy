use ratatui::{
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub struct PtyView;

impl Default for PtyView {
    fn default() -> Self {
        Self::new()
    }
}

impl PtyView {
    pub fn new() -> Self {
        Self
    }

    pub fn render<'a>(&self, text: String, focused: bool) -> Paragraph<'a> {
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(if focused { " Codex * " } else { " Codex " })
                    .borders(Borders::ALL)
                    .border_style(if focused {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }),
            )
            .wrap(Wrap { trim: false })
    }
}
