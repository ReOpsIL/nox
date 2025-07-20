use crate::tui::{app::AppState, utils::{colors::*, formatting::*}};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub struct ExecutionScreen;

impl ExecutionScreen {
    pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
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
        let running_tasks: Vec<(usize, &crate::types::Task)> = state
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| task.status == crate::types::TaskStatus::InProgress)
            .collect();

        let queued_tasks: Vec<&crate::types::Task> = state
            .tasks
            .iter()
            .filter(|task| task.status == crate::types::TaskStatus::Todo)
            .take(3)
            .collect();

        let mut items = Vec::new();

        for (task_index, task) in &running_tasks {
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

            // Check if this task is selected
            let is_selected = state.selected_task == Some(*task_index);
            let selection_indicator = if is_selected { ">" } else { " " };
            
            let title_style = if is_selected { selected_style() } else { highlight_style() };
            let agent_style = if is_selected { selected_style() } else { primary_style() };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(selection_indicator, text_secondary_style()),
                Span::styled("🟢 ", task_running_style()),
                Span::styled(&task.title, title_style),
                Span::styled(" - ", text_secondary_style()),
                Span::styled(agent_name, agent_style),
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

        let footer_text = if running_tasks.len() > 0 {
            format!(
                "[C] Cancel  [P] Pause  [R] Resume  [A] Cancel All\n\
                 \n\
                 Active: {}, Queued: {}, Slots: 5",
                running_tasks.len(),
                queued_tasks.len()
            )
        } else {
            format!(
                "No running tasks\n\
                 \n\
                 Active: {}, Queued: {}, Slots: 5",
                running_tasks.len(),
                queued_tasks.len()
            )
        };

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

                let mut lines = vec![
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
                ];

                // Show real execution output based on task status
                match selected_task.status {
                    crate::types::TaskStatus::InProgress => {
                        lines.push(Line::from(vec![
                            Span::styled("🔄 ", task_running_style()),
                            Span::styled("Task is running...", info_style()),
                        ]));
                        
                        // Show progress bar if available
                        if selected_task.progress > 0 {
                            lines.push(Line::from(vec![
                                Span::styled(
                                    format_progress_bar(selected_task.progress, 20),
                                    primary_style()
                                ),
                                Span::styled(format!(" {}%", selected_task.progress), info_style()),
                            ]));
                        }
                        
                        lines.push(Line::from(""));
                        
                        // Show real-time streaming output if available
                        if let Some(output_lines) = state.task_outputs.get(&selected_task.id) {
                            lines.push(Line::from(vec![
                                Span::styled("📺 Live Output:", success_style()),
                            ]));
                            lines.push(Line::from(""));
                            
                            // Show the last 10 lines of output
                            let recent_lines = output_lines.iter().rev().take(10).rev();
                            for output_line in recent_lines {
                                let available_width = area.width.saturating_sub(4) as usize;
                                let wrapped_lines = wrap_text(output_line, available_width, text_primary_style());
                                for wrapped_line in wrapped_lines {
                                    lines.push(wrapped_line);
                                }
                            }
                            
                            if output_lines.len() > 10 {
                                lines.push(Line::from(vec![
                                    Span::styled(format!("... ({} more lines)", output_lines.len() - 10), muted_style()),
                                ]));
                            }
                        } else {
                            lines.push(Line::from("⏳ Waiting for output..."));
                        }
                    }
                    
                    crate::types::TaskStatus::Done => {
                        // Show completion info
                        if let Some(completed_at) = &selected_task.completed_at {
                            lines.push(Line::from(vec![
                                Span::styled("Completed: ", muted_style()),
                                Span::styled(format_datetime(completed_at), info_style()),
                            ]));
                        }
                        
                        lines.push(Line::from(""));
                        
                        // Show the final output or log file content
                        if let Some(output_lines) = state.task_outputs.get(&selected_task.id) {
                            lines.push(Line::from(vec![
                                Span::styled("✅ ", task_done_style()),
                                Span::styled("Final Output:", success_style()),
                            ]));
                            lines.push(Line::from(""));
                            
                            // Show all output lines for completed tasks
                            for output_line in output_lines {
                                let available_width = area.width.saturating_sub(4) as usize;
                                let wrapped_lines = wrap_text(output_line, available_width, text_primary_style());
                                for wrapped_line in wrapped_lines {
                                    lines.push(wrapped_line);
                                }
                            }
                        } else if let Some(claude_response) = selected_task.metadata.get("claude_response") {
                            lines.push(Line::from(vec![
                                Span::styled("✅ ", task_done_style()),
                                Span::styled("Claude Response:", success_style()),
                            ]));
                            lines.push(Line::from(""));
                            
                            // Wrap the response text to fit the panel width
                            let available_width = area.width.saturating_sub(4) as usize; // Account for borders
                            let wrapped_lines = wrap_text(claude_response, available_width, text_primary_style());
                            
                            for wrapped_line in wrapped_lines {
                                lines.push(wrapped_line);
                            }
                        } else {
                            lines.push(Line::from(vec![
                                Span::styled("✅ ", task_done_style()),
                                Span::styled("Task completed successfully", success_style()),
                            ]));
                        }
                    }
                    
                    crate::types::TaskStatus::Cancelled => {
                        if let Some(completed_at) = &selected_task.completed_at {
                            lines.push(Line::from(vec![
                                Span::styled("Cancelled: ", muted_style()),
                                Span::styled(format_datetime(completed_at), info_style()),
                            ]));
                        }
                        
                        lines.push(Line::from(""));
                        lines.push(Line::from(vec![
                            Span::styled("❌ ", task_cancelled_style()),
                            Span::styled("Task was cancelled", error_style()),
                        ]));
                    }

                    crate::types::TaskStatus::Failed => {
                        if let Some(completed_at) = &selected_task.completed_at {
                            lines.push(Line::from(vec![
                                Span::styled("Failed: ", muted_style()),
                                Span::styled(format_datetime(completed_at), info_style()),
                            ]));
                        }

                        lines.push(Line::from(""));
                        lines.push(Line::from(vec![
                            Span::styled("⚠️", task_cancelled_style()),
                            Span::styled("Task Failed", error_style()),
                        ]));
                    }
                    
                    crate::types::TaskStatus::Todo => {
                        lines.push(Line::from(vec![
                            Span::styled("⏳ ", task_todo_style()),
                            Span::styled("Task is pending execution", muted_style()),
                        ]));
                    }
                }

                lines
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
        let cpu_usage = state.system_status.cpu_usage_percent.clamp(0.0, 100.0);
        let memory_usage = state.system_status.memory_usage_percent.clamp(0.0, 100.0);

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
            .ratio((cpu_usage as f64 / 100.0).clamp(0.0, 1.0))
            .label(format!("CPU: {:.0}%", cpu_usage));

        let memory_gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(
                if memory_usage > 80.0 { error_style() }
                else if memory_usage > 60.0 { warning_style() }
                else { success_style() }
            )
            .ratio((memory_usage as f64 / 100.0).clamp(0.0, 1.0))
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