use crate::tui::{app::AppState, utils::{colors::*, formatting::*}};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub struct ExecutionScreen;

impl ExecutionScreen {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(area);

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[0]);

        Self::render_running_tasks(frame, top_chunks[0], state);
        Self::render_execution_output(frame, top_chunks[1], state);
        Self::render_system_resources(frame, chunks[1], state);
    }

    fn render_running_tasks(frame: &mut Frame, area: Rect, state: &AppState) {
        let running_tasks: Vec<&crate::types::Task> = state
            .tasks
            .iter()
            .filter(|task| task.status == crate::types::TaskStatus::InProgress)
            .collect();

        let queued_tasks: Vec<&crate::types::Task> = state
            .tasks
            .iter()
            .filter(|task| task.status == crate::types::TaskStatus::Todo)
            .take(3)
            .collect();

        let mut items = Vec::new();

        for task in &running_tasks {
            let agent_name = state.agents
                .iter()
                .find(|a| a.id == task.agent_id)
                .map(|a| a.name.as_str())
                .unwrap_or("Unknown");

            let duration = if let Some(started_at) = &task.started_at {
                let elapsed = chrono::Utc::now().signed_duration_since(*started_at);
                format!("{}s", elapsed.num_seconds())
            } else {
                "0s".to_string()
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled("🟢 ", task_running_style()),
                Span::styled(&task.title, highlight_style()),
                Span::styled(" - ", text_secondary_style()),
                Span::styled(agent_name, primary_style()),
                Span::styled(" (", text_secondary_style()),
                Span::styled(duration, accent_style()),
                Span::styled(")", text_secondary_style()),
            ])));
        }

        if !queued_tasks.is_empty() {
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from(Span::styled("📋 Queue:", secondary_style()))));

            for task in &queued_tasks {
                let agent_name = state.agents
                    .iter()
                    .find(|a| a.id == task.agent_id)
                    .map(|a| a.name.as_str())
                    .unwrap_or("Unknown");

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  ⏳ ", task_todo_style()),
                    Span::styled(&task.title, text_secondary_style()),
                    Span::styled(" - ", muted_style()),
                    Span::styled(agent_name, muted_style()),
                ])));
            }
        }

        let footer_text = format!(
            "[C] Cancel  [P] Pause  [R] Resume\n\
             \n\
             Active: {}, Queued: {}, Slots: 5",
            running_tasks.len(),
            queued_tasks.len()
        );

        let list_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(4),
            ])
            .split(area);

        let block = Block::default()
            .title("⚡ Running Tasks")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(primary_style());

        let list = List::new(items).block(block);
        frame.render_widget(list, list_area[0]);

        let footer = Paragraph::new(footer_text).style(text_secondary_style());
        frame.render_widget(footer, list_area[1]);
    }

    fn render_execution_output(frame: &mut Frame, area: Rect, state: &AppState) {
        let content = if let Some(selected_task_index) = state.selected_task {
            if let Some(selected_task) = state.tasks.get(selected_task_index) {
                if selected_task.status == crate::types::TaskStatus::InProgress {
                    let agent_name = state.agents
                        .iter()
                        .find(|a| a.id == selected_task.agent_id)
                        .map(|a| a.name.as_str())
                        .unwrap_or("Unknown");

                    let started_time = if let Some(started_at) = &selected_task.started_at {
                        format_datetime(started_at)
                    } else {
                        "Unknown".to_string()
                    };

                    vec![
                        Line::from(vec![
                            Span::styled("Task: ", muted_style()),
                            Span::styled(&selected_task.title, info_style()),
                        ]),
                        Line::from(vec![
                            Span::styled("Agent: ", muted_style()),
                            Span::styled(agent_name, info_style()),
                        ]),
                        Line::from(vec![
                            Span::styled("Started: ", muted_style()),
                            Span::styled(started_time, info_style()),
                        ]),
                        Line::from("──────────────────────────────────"),
                        Line::from("[14:20:15] Task started"),
                        Line::from("[14:20:16] Initializing environment"),
                        Line::from("[14:20:18] Processing data..."),
                        Line::from("[14:20:22] Analyzing content..."),
                        Line::from("[14:20:35] Generating output..."),
                        Line::from(vec![
                            Span::styled("[14:20:45] ", muted_style()),
                            Span::styled(
                                format_progress_bar(selected_task.progress, 15),
                                primary_style()
                            ),
                            Span::styled(format!(" {}%", selected_task.progress), info_style()),
                        ]),
                    ]
                } else {
                    vec![Line::from("Select a running task to view output")]
                }
            } else {
                vec![Line::from("No task selected")]
            }
        } else {
            vec![Line::from("No task selected")]
        };

        let block = Block::default()
            .title("📺 Execution Output")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(accent_style());

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_system_resources(frame: &mut Frame, area: Rect, state: &AppState) {
        let cpu_usage = state.system_status.cpu_usage_percent;
        let memory_usage = (state.system_status.memory_usage_mb as f64 / 1024.0) / 8.0 * 100.0; // Assume 8GB total

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(area);

        let cpu_gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(
                if cpu_usage > 80.0 { error_style() }
                else if cpu_usage > 60.0 { warning_style() }
                else { success_style() }
            )
            .ratio(cpu_usage as f64 / 100.0)
            .label(format!("CPU: {:.0}%", cpu_usage));

        let memory_gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(
                if memory_usage > 80.0 { error_style() }
                else if memory_usage > 60.0 { warning_style() }
                else { success_style() }
            )
            .ratio(memory_usage / 100.0)
            .label(format!("Memory: {:.0}%", memory_usage));

        let status_text = format!(
            "Active Processes: 2/5    Claude CLI Sessions: 2"
        );
        let status = Paragraph::new(status_text).style(text_primary_style());

        let block = Block::default()
            .title("📊 System Resources")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(secondary_style());

        frame.render_widget(block, area);
        frame.render_widget(cpu_gauge, chunks[0]);
        frame.render_widget(memory_gauge, chunks[1]);
        frame.render_widget(status, chunks[3]);
    }
}