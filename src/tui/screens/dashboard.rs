use crate::tui::{app::AppState, utils::{colors::*, formatting::*}};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub struct Dashboard;

impl Dashboard {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(7),  // Status cards
                Constraint::Min(5),     // Activity feed
            ])
            .split(area);

        Self::render_status_cards(frame, chunks[0], state);
        Self::render_activity_feed(frame, chunks[1], state);
    }

    fn render_status_cards(frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(area);

        Self::render_system_status(frame, chunks[0], state);
        Self::render_agent_summary(frame, chunks[1], state);
        Self::render_task_summary(frame, chunks[2], state);
    }

    fn render_system_status(frame: &mut Frame, area: Rect, state: &AppState) {
        let uptime = format_duration(state.system_status.uptime_seconds);
        let memory = format_memory(state.system_status.memory_usage_mb);
        let cpu = format_percentage(state.system_status.cpu_usage_percent);

        let content = vec![
            Line::from(vec![
                Span::styled("Status: ", text_secondary_style()),
                Span::styled("🟢 Running", success_style()),
            ]),
            Line::from(vec![
                Span::styled("⏱️  Uptime: ", text_secondary_style()),
                Span::styled(uptime, highlight_style()),
            ]),
            Line::from(vec![
                Span::styled("🖥️  CPU: ", text_secondary_style()),
                Span::styled(cpu, primary_style()),
            ]),
            Line::from(vec![
                Span::styled("💾 Memory: ", text_secondary_style()),
                Span::styled(memory, accent_style()),
            ]),
        ];

        let block = Block::default()
            .title("⚡ System Status")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(primary_style());

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_agent_summary(frame: &mut Frame, area: Rect, state: &AppState) {
        let total_agents = state.system_status.active_agents + state.system_status.inactive_agents;
        
        let content = vec![
            Line::from(vec![
                Span::styled("🟢 Active: ", text_secondary_style()),
                Span::styled(
                    format!("{}/{}", state.system_status.active_agents, total_agents),
                    agent_active_style(),
                ),
            ]),
            Line::from(vec![
                Span::styled("⚪ Inactive: ", text_secondary_style()),
                Span::styled(
                    format!("{}/{}", state.system_status.inactive_agents, total_agents),
                    agent_inactive_style(),
                ),
            ]),
            Line::from(vec![
                Span::styled("🔴 Error: ", text_secondary_style()),
                Span::styled(format!("{}/{}", state.system_status.error_agents, total_agents), muted_style()),
            ]),
        ];

        let block = Block::default()
            .title("🤖 Agent Summary")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(secondary_style());

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_task_summary(frame: &mut Frame, area: Rect, state: &AppState) {
        let running = state.system_status.tasks_by_status
            .get(&crate::types::TaskStatus::InProgress)
            .unwrap_or(&0);
        let pending = state.system_status.tasks_by_status
            .get(&crate::types::TaskStatus::Todo)
            .unwrap_or(&0);
        let completed = state.system_status.tasks_by_status
            .get(&crate::types::TaskStatus::Done)
            .unwrap_or(&0);
        let cancelled = state.system_status.tasks_by_status
            .get(&crate::types::TaskStatus::Cancelled)
            .unwrap_or(&0);

        let content = vec![
            Line::from(vec![
                Span::styled("🔄 Running: ", text_secondary_style()),
                Span::styled(running.to_string(), task_running_style()),
            ]),
            Line::from(vec![
                Span::styled("⏳ Pending: ", text_secondary_style()),
                Span::styled(pending.to_string(), task_todo_style()),
            ]),
            Line::from(vec![
                Span::styled("✅ Completed: ", text_secondary_style()),
                Span::styled(completed.to_string(), task_done_style()),
            ]),
            Line::from(vec![
                Span::styled("❌ Failed: ", text_secondary_style()),
                Span::styled(cancelled.to_string(), task_cancelled_style()),
            ]),
        ];

        let block = Block::default()
            .title("📋 Task Summary")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(accent_style());

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_activity_feed(frame: &mut Frame, area: Rect, _state: &AppState) {
        // TODO: Implement actual activity feed with recent events
        let activities = vec![
            ("🚀", "[14:23] Agent 'NewsBot' started task 'Analyze papers'", task_running_style()),
            ("✅", "[14:20] Task 'Generate report' completed successfully", task_done_style()),
            ("🟢", "[14:15] Agent 'CodeReviewer' status changed to Active", agent_active_style()),
            ("📝", "[14:10] New task 'Review logs' created for agent 'DataAnalyst'", task_todo_style()),
            ("🏁", "[14:05] Agent 'TestRunner' completed task 'Run tests'", task_done_style()),
        ];

        let items: Vec<ListItem> = activities
            .iter()
            .map(|(icon, activity, style)| {
                ListItem::new(Line::from(vec![
                    Span::styled(*icon, highlight_style()),
                    Span::styled(" ", text_primary_style()),
                    Span::styled(*activity, *style),
                ]))
            })
            .collect();

        let block = Block::default()
            .title("📈 Recent Activity")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(info_style());

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}