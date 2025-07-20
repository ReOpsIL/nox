use crate::tui::utils::{colors::*, formatting::*};
use crate::types::{Agent, Task};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct DetailPanel;

impl DetailPanel {
    pub fn render_agent(frame: &mut Frame, area: Rect, agent: &Agent) {
        let (status_icon, status_text) = format_agent_status(&agent.status);
        let created_at = format_datetime(&agent.created_at);

        // Calculate available width for text wrapping (accounting for borders and padding)
        let text_width = area.width.saturating_sub(4) as usize; // 2 for borders + 2 for padding

        let mut content = vec![
            Line::from(vec![
                Span::styled("Name: ", muted_style()),
                Span::styled(&agent.name, info_style()),
            ]),
            Line::from(vec![
                Span::styled("ID: ", muted_style()),
                Span::styled(&agent.id, muted_style()),
            ]),
            Line::from(vec![
                Span::styled("Status: ", muted_style()),
                Span::styled(format!("{} {}", status_icon, status_text), 
                           match agent.status {
                               crate::types::AgentStatus::Active => success_style(),
                               crate::types::AgentStatus::Error => error_style(),
                               _ => warning_style(),
                           }),
            ]),
            Line::from(vec![
                Span::styled("Created: ", muted_style()),
                Span::styled(created_at, info_style()),
            ]),
            Line::from(""),
            Line::from(Span::styled("System Prompt:", muted_style())),
        ];

        // Add wrapped system prompt text
        let wrapped_prompt = wrap_text(&agent.system_prompt, text_width, info_style());
        content.extend(wrapped_prompt);
        
        content.extend(vec![
            Line::from(""),
            Line::from(Span::styled("Resource Limits:", muted_style())),
            Line::from(vec![
                Span::styled("Memory: ", muted_style()),
                Span::styled(
                    format!("{}MB", agent.resource_limits.max_memory_mb),
                    info_style()
                ),
                Span::styled(", CPU: ", muted_style()),
                Span::styled(
                    format!("{}%", agent.resource_limits.max_cpu_percent),
                    info_style()
                ),
            ]),
        ]);

        let block = Block::default()
            .title("Agent Details")
            .borders(Borders::ALL)
            .border_style(border_style());

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }

    pub fn render_task(frame: &mut Frame, area: Rect, task: &Task, agent_name: &str) {
        let (status_icon, status_text) = format_task_status(&task.status);
        let (priority_icon, priority_text) = format_task_priority(&task.priority);
        let created_at = format_datetime(&task.created_at);
        let progress_bar = format_progress_bar(task.progress, 20);

        // Calculate available width for text wrapping (accounting for borders and padding)
        let text_width = area.width.saturating_sub(4) as usize; // 2 for borders + 2 for padding

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Title: ", muted_style()),
                Span::styled(&task.title, info_style()),
            ]),
            Line::from(vec![
                Span::styled("Agent: ", muted_style()),
                Span::styled(agent_name, info_style()),
            ]),
            Line::from(vec![
                Span::styled("Status: ", muted_style()),
                Span::styled(format!("{} {}", status_icon, status_text), 
                           match task.status {
                               crate::types::TaskStatus::Done => success_style(),
                               crate::types::TaskStatus::InProgress => primary_style(),
                               crate::types::TaskStatus::Cancelled => error_style(),
                               _ => warning_style(),
                           }),
            ]),
            Line::from(vec![
                Span::styled("Priority: ", muted_style()),
                Span::styled(format!("{} {}", priority_icon, priority_text), 
                           match task.priority {
                               crate::types::TaskPriority::High => error_style(),
                               crate::types::TaskPriority::Medium => warning_style(),
                               crate::types::TaskPriority::Low => success_style(),
                           }),
            ]),
            Line::from(vec![
                Span::styled("Created: ", muted_style()),
                Span::styled(created_at, info_style()),
            ]),
            Line::from(""),
            Line::from(Span::styled("Description:", muted_style())),
        ];

        // Add wrapped description text
        let wrapped_description = wrap_text(&task.description, text_width, info_style());
        lines.extend(wrapped_description);
        
        lines.extend(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Progress: ", muted_style()),
                Span::styled(progress_bar, primary_style()),
                Span::styled(format!(" {}%", task.progress), info_style()),
            ]),
        ]);

        if let Some(started_at) = &task.started_at {
            lines.push(Line::from(vec![
                Span::styled("Started: ", muted_style()),
                Span::styled(format_datetime(started_at), info_style()),
            ]));
        }

        if let Some(completed_at) = &task.completed_at {
            lines.push(Line::from(vec![
                Span::styled("Completed: ", muted_style()),
                Span::styled(format_datetime(completed_at), info_style()),
            ]));
        }

        // Add execution output if available
        if let Some(claude_response) = task.metadata.get("claude_response") {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Execution Output:", muted_style())));
            
            let wrapped_output = wrap_text(claude_response, text_width, info_style());
            lines.extend(wrapped_output);
        }

        let block = Block::default()
            .title("Task Details")
            .borders(Borders::ALL)
            .border_style(border_style());

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }
}