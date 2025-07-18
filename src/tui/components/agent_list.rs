use crate::tui::utils::{colors::*, formatting::*};
use crate::types::Agent;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct AgentList;

impl AgentList {
    pub fn render(
        frame: &mut Frame, 
        area: Rect, 
        agents: &[Agent], 
        selected: Option<usize>
    ) {
        let items: Vec<ListItem> = agents
            .iter()
            .enumerate()
            .map(|(i, agent)| {
                let (status_icon, _) = format_agent_status(&agent.status);
                let style = if Some(i) == selected {
                    selected_style()
                } else {
                    info_style()
                };

                let content = format!("{} {} {}", 
                    if Some(i) == selected { ">" } else { " " },
                    agent.name,
                    status_icon
                );

                ListItem::new(Line::from(Span::styled(content, style)))
            })
            .collect();

        let block = Block::default()
            .title("Agents")
            .borders(Borders::ALL)
            .border_style(border_style());

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}