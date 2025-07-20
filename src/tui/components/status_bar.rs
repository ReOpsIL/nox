use crate::tui::{app::AppState, utils::colors::*};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct StatusBar;

impl StatusBar {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState, message: &str) {
        let content = if message.is_empty() {
            Line::from(vec![
                Span::styled("📍 Screen: ", text_secondary_style()),
                Span::styled(format!("{:?}", state.current_screen), highlight_style()),
                Span::styled(" | 🤖 Agents: ", text_secondary_style()),
                Span::styled(
                    format!("{}/{}", 
                        state.system_status.active_agents,
                        state.system_status.active_agents + state.system_status.inactive_agents
                    ),
                    primary_style()
                ),
                Span::styled(" | 📋 Tasks: ", text_secondary_style()),
                Span::styled(state.tasks.len().to_string(), accent_style()),
            ])
        } else {
            Line::from(Span::styled(message, warning_style()))
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style());

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }
}