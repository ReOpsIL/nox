use crate::tui::{app::AppState, utils::{colors::*, formatting::*}};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct AgentsScreen;

impl AgentsScreen {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ])
            .split(area);

        Self::render_agent_list(frame, chunks[0], state);
        Self::render_agent_details(frame, chunks[1], state);
    }

    fn render_agent_list(frame: &mut Frame, area: Rect, state: &AppState) {
        let items: Vec<ListItem> = state
            .agents
            .iter()
            .enumerate()
            .map(|(i, agent)| {
                let (status_icon, _) = format_agent_status(&agent.status);
                let is_selected = Some(i) == state.selected_agent;
                
                let agent_style = match agent.status {
                    crate::types::AgentStatus::Active => agent_active_style(),
                    crate::types::AgentStatus::Inactive => agent_inactive_style(),
                    crate::types::AgentStatus::Error => agent_error_style(),
                    crate::types::AgentStatus::Starting => agent_starting_style(),
                    crate::types::AgentStatus::Stopping => agent_stopping_style(),
                };

                let content = if is_selected {
                    Line::from(vec![
                        Span::styled("👉 ", highlight_style()),
                        Span::styled(&agent.name, selected_style()),
                        Span::styled(" ", text_primary_style()),
                        Span::styled(status_icon, agent_style),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled("   ", text_primary_style()),
                        Span::styled(&agent.name, text_primary_style()),
                        Span::styled(" ", text_primary_style()),
                        Span::styled(status_icon, agent_style),
                    ])
                };

                ListItem::new(content)
            })
            .collect();

        let footer_text = format!(
            "[N] New  [E] Edit  [D] Delete     \n\
             [S] Start  [T] Stop  [R] Restart  \n\
             \n\
             Total: {} agents\n\
             Active: {}, Inactive: {}",
            state.agents.len(),
            state.system_status.active_agents,
            state.system_status.inactive_agents
        );

        let footer = Paragraph::new(footer_text)
            .style(text_secondary_style());

        let list_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(6),
            ])
            .split(area);

        let block = Block::default()
            .title("🤖 Agent List")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(secondary_style());

        let list = List::new(items).block(block);
        frame.render_widget(list, list_area[0]);
        frame.render_widget(footer, list_area[1]);
    }

    fn render_agent_details(frame: &mut Frame, area: Rect, state: &AppState) {
        let content = if let Some(selected_index) = state.selected_agent {
            if let Some(agent) = state.agents.get(selected_index) {
                let (status_icon, status_text) = format_agent_status(&agent.status);
                let created_at = format_datetime(&agent.created_at);

                // Calculate available width for text wrapping (accounting for borders and padding)
                let text_width = area.width.saturating_sub(4) as usize; // 2 for borders + 2 for padding

                let mut content = vec![
                    Line::from(vec![
                        Span::styled("🏷️  Name: ", text_secondary_style()),
                        Span::styled(&agent.name, highlight_style()),
                    ]),
                    Line::from(vec![
                        Span::styled("🆔 ID: ", text_secondary_style()),
                        Span::styled(&agent.id, muted_style()),
                    ]),
                    Line::from(vec![
                        Span::styled("🟡 Status: ", text_secondary_style()),
                        Span::styled(format!("{} {}", status_icon, status_text), 
                                   match agent.status {
                                       crate::types::AgentStatus::Active => agent_active_style(),
                                       crate::types::AgentStatus::Error => agent_error_style(),
                                       crate::types::AgentStatus::Starting => agent_starting_style(),
                                       crate::types::AgentStatus::Stopping => agent_stopping_style(),
                                       _ => agent_inactive_style(),
                                   }),
                    ]),
                    Line::from(vec![
                        Span::styled("📅 Created: ", text_secondary_style()),
                        Span::styled(created_at, info_style()),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled("📜 System Prompt:", primary_style())),
                ];

                // Add wrapped system prompt text
                let wrapped_prompt = crate::tui::utils::formatting::wrap_text(
                    &agent.system_prompt, 
                    text_width, 
                    text_primary_style()
                );
                content.extend(wrapped_prompt);
                
                content.extend(vec![
                    Line::from(""),
                    Line::from(Span::styled("⚙️  Resource Limits:", accent_style())),
                    Line::from(vec![
                        Span::styled("💾 Memory: ", text_secondary_style()),
                        Span::styled(
                            format!("{}MB", agent.resource_limits.max_memory_mb),
                            warning_style()
                        ),
                        Span::styled(", 🖥️  CPU: ", text_secondary_style()),
                        Span::styled(
                            format!("{}%", agent.resource_limits.max_cpu_percent),
                            warning_style()
                        ),
                    ]),
                ]);

                content
            } else {
                vec![Line::from(Span::styled("👀 No agent selected", muted_style()))]
            }
        } else {
            vec![Line::from(Span::styled("👀 No agent selected", muted_style()))]
        };

        let block = Block::default()
            .title("🔍 Agent Details")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(info_style());

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }
}