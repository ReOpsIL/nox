use crate::tui::{app::AppState, utils::{colors::*, formatting::*}};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct TasksScreen;

impl TasksScreen {
    pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(50),
            ])
            .split(area);

        Self::render_task_list(frame, chunks[0], state);
        Self::render_task_details(frame, chunks[1], state);
    }

    fn render_task_list(frame: &mut Frame, area: Rect, state: &mut AppState) {
        // Get filtered and sorted task indices
        let filtered_indices = Self::get_filtered_task_indices(state);
        let sorted_indices = Self::sort_task_indices(filtered_indices, state);

        let items: Vec<ListItem> = sorted_indices
            .iter()
            .enumerate()
            .map(|(_display_i, &original_i)| {
                let task = &state.tasks[original_i];
                let (status_icon, _) = format_task_status(&task.status);
                let (priority_icon, _) = format_task_priority(&task.priority);
                let is_selected = Some(original_i) == state.selected_task;
                
                let status_style = match task.status {
                    crate::types::TaskStatus::Done => task_done_style(),
                    crate::types::TaskStatus::InProgress => task_running_style(),
                    crate::types::TaskStatus::Cancelled => task_cancelled_style(),
                    _ => task_todo_style(),
                };
                
                let priority_style = match task.priority {
                    crate::types::TaskPriority::High => priority_high_style(),
                    crate::types::TaskPriority::Medium => priority_medium_style(),
                    crate::types::TaskPriority::Low => priority_low_style(),
                };

                let agent_name = state.agents
                    .iter()
                    .find(|a| a.id == task.agent_id)
                    .map(|a| a.name.as_str())
                    .unwrap_or("Unknown");

                let content = if is_selected {
                    Line::from(vec![
                        Span::styled("👉 ", highlight_style()),
                        Span::styled(format!("[{}]: ", agent_name), text_secondary_style()),
                        Span::styled(&task.title, selected_style()),
                        Span::styled(" ", text_primary_style()),
                        Span::styled(status_icon, status_style),
                        Span::styled(" ", text_primary_style()),
                        Span::styled(priority_icon, priority_style),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled("   ", text_primary_style()),
                        Span::styled(format!("[{}]: ", agent_name), text_secondary_style()),
                        Span::styled(&task.title, text_primary_style()),
                        Span::styled(" ", text_primary_style()),
                        Span::styled(status_icon, status_style),
                        Span::styled(" ", text_primary_style()),
                        Span::styled(priority_icon, priority_style),
                    ])
                };

                ListItem::new(content)
            })
            .collect();

        let running_count = state.system_status.tasks_by_status
            .get(&crate::types::TaskStatus::InProgress)
            .unwrap_or(&0);
        let pending_count = state.system_status.tasks_by_status
            .get(&crate::types::TaskStatus::Todo)
            .unwrap_or(&0);
        let completed_count = state.system_status.tasks_by_status
            .get(&crate::types::TaskStatus::Done)
            .unwrap_or(&0);

        let filter_status = if let Some(ref filter) = state.filters.task_status_filter {
            format!("(Filter: {})", match filter.as_str() {
                "Todo" => "Pending",
                "InProgress" => "Running", 
                "Done" => "Complete",
                "Cancelled" => "Cancelled",
                _ => "Unknown"
            })
        } else {
            String::new()
        };

        let footer_text = format!(
            "Filter: [A] All [R] Running       \n\
             [P] Pending [C] Complete  \n\
             \n\
             [N] New  [E] Execute  [X] Cancel\n\
             [U] Update  [D] Delete            \n\
             \n\
             Total: {} tasks {}\n\
             {} Running, {} Pending, {} Done",
            sorted_indices.len(),
            filter_status,
            running_count,
            pending_count,
            completed_count
        );

        let footer = Paragraph::new(footer_text)
            .style(text_secondary_style());

        let list_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(8),
            ])
            .split(area);

        let block = Block::default()
            .title("📋 Task List")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(accent_style());

        let list = List::new(items)
            .block(block)
            .highlight_style(ratatui::style::Style::default().bg(ratatui::style::Color::DarkGray));
        frame.render_stateful_widget(list, list_area[0], &mut state.tasks_list_state);
        frame.render_widget(footer, list_area[1]);
    }

    fn render_task_details(frame: &mut Frame, area: Rect, state: &AppState) {
        let content = if let Some(selected_index) = state.selected_task {
            if let Some(task) = state.tasks.get(selected_index) {
                let (status_icon, status_text) = format_task_status(&task.status);
                let (priority_icon, priority_text) = format_task_priority(&task.priority);
                let created_at = format_datetime(&task.created_at);
                let progress_bar = format_progress_bar(task.progress, 20);

                // Calculate available width for text wrapping (accounting for borders and padding)
                let text_width = area.width.saturating_sub(4) as usize; // 2 for borders + 2 for padding

                let agent_name = state.agents
                    .iter()
                    .find(|a| a.id == task.agent_id)
                    .map(|a| a.name.as_str())
                    .unwrap_or("Unknown");

                let mut lines = vec![
                    Line::from(vec![
                        Span::styled("🏷️  Title: ", text_secondary_style()),
                        Span::styled(&task.title, highlight_style()),
                    ]),
                    Line::from(vec![
                        Span::styled("🤖 Agent: ", text_secondary_style()),
                        Span::styled(agent_name, primary_style()),
                    ]),
                    Line::from(vec![
                        Span::styled("🟡 Status: ", text_secondary_style()),
                        Span::styled(format!("{} {}", status_icon, status_text), 
                                   match task.status {
                                       crate::types::TaskStatus::Done => task_done_style(),
                                       crate::types::TaskStatus::InProgress => task_running_style(),
                                       crate::types::TaskStatus::Cancelled => task_cancelled_style(),
                                       _ => task_todo_style(),
                                   }),
                    ]),
                    Line::from(vec![
                        Span::styled("⚡ Priority: ", text_secondary_style()),
                        Span::styled(format!("{} {}", priority_icon, priority_text), 
                                   match task.priority {
                                       crate::types::TaskPriority::High => priority_high_style(),
                                       crate::types::TaskPriority::Medium => priority_medium_style(),
                                       crate::types::TaskPriority::Low => priority_low_style(),
                                   }),
                    ]),
                    Line::from(vec![
                        Span::styled("📅 Created: ", text_secondary_style()),
                        Span::styled(created_at, info_style()),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled("📜 Description:", secondary_style())),
                ];

                // Add wrapped description text
                let wrapped_description = crate::tui::utils::formatting::wrap_text(
                    &task.description, 
                    text_width, 
                    text_primary_style()
                );
                lines.extend(wrapped_description);
                
                lines.extend(vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("📈 Progress: ", text_secondary_style()),
                        Span::styled(progress_bar, progress_style()),
                        Span::styled(format!(" {}%", task.progress), accent_style()),
                    ]),
                ]);

                if let Some(started_at) = &task.started_at {
                    lines.push(Line::from(vec![
                        Span::styled("🚀 Started: ", text_secondary_style()),
                        Span::styled(format_datetime(started_at), task_running_style()),
                    ]));
                }

                if let Some(completed_at) = &task.completed_at {
                    lines.push(Line::from(vec![
                        Span::styled("🏁 Completed: ", text_secondary_style()),
                        Span::styled(format_datetime(completed_at), task_done_style()),
                    ]));
                }

                // Add execution output if available
                if let Some(claude_response) = task.metadata.get("claude_response") {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("⚡ Execution Output:", accent_style())));
                    
                    let wrapped_output = crate::tui::utils::formatting::wrap_text(
                        claude_response, 
                        text_width, 
                        text_primary_style()
                    );
                    lines.extend(wrapped_output);
                }

                lines
            } else {
                vec![Line::from(Span::styled("👀 No task selected", muted_style()))]
            }
        } else {
            vec![Line::from(Span::styled("👀 No task selected", muted_style()))]
        };

        let block = Block::default()
            .title("🔍 Task Details")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(info_style());

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }

    // Get tasks filtered by current filter state
    fn get_filtered_task_indices(state: &AppState) -> Vec<usize> {
        let mut task_indices: Vec<usize> = (0..state.tasks.len()).collect();
        
        // Apply status filter if set
        if let Some(ref status_filter) = state.filters.task_status_filter {
            task_indices.retain(|&i| {
                let task = &state.tasks[i];
                match status_filter.as_str() {
                    "Todo" => matches!(task.status, crate::types::TaskStatus::Todo),
                    "InProgress" => matches!(task.status, crate::types::TaskStatus::InProgress),
                    "Done" => matches!(task.status, crate::types::TaskStatus::Done),
                    "Cancelled" => matches!(task.status, crate::types::TaskStatus::Cancelled),
                    _ => true, // Show all for unknown filters
                }
            });
        }
        
        // Apply search query filter if set
        if !state.filters.search_query.is_empty() {
            let search_query = state.filters.search_query.to_lowercase();
            task_indices.retain(|&i| {
                let task = &state.tasks[i];
                task.title.to_lowercase().contains(&search_query) ||
                task.description.to_lowercase().contains(&search_query)
            });
        }
        
        task_indices
    }

    // Sort task indices by agent name
    fn sort_task_indices(task_indices: Vec<usize>, state: &AppState) -> Vec<usize> {
        let mut sorted_indices = task_indices;
        sorted_indices.sort_by(|&a, &b| {
            let agent_a_name = state.agents
                .iter()
                .find(|agent| agent.id == state.tasks[a].agent_id)
                .map(|agent| agent.name.as_str())
                .unwrap_or("Unknown");
            let agent_b_name = state.agents
                .iter()
                .find(|agent| agent.id == state.tasks[b].agent_id)
                .map(|agent| agent.name.as_str())
                .unwrap_or("Unknown");
            agent_a_name.cmp(agent_b_name)
        });
        sorted_indices
    }

}