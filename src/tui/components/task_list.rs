use crate::tui::{app::AppState, utils::{colors::*, formatting::*}};
use crate::types::Task;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct TaskList;

impl TaskList {
    pub fn render(
        frame: &mut Frame, 
        area: Rect, 
        tasks: &[Task], 
        selected: Option<usize>,
        state: &AppState
    ) {
        let items: Vec<ListItem> = tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let (status_icon, _) = format_task_status(&task.status);
                let (priority_icon, _) = format_task_priority(&task.priority);
                let style = if Some(i) == selected {
                    selected_style()
                } else {
                    info_style()
                };

                let agent_name = state.agents
                    .iter()
                    .find(|a| a.id == task.agent_id)
                    .map(|a| a.name.as_str())
                    .unwrap_or("Unknown");

                let content = format!("{} [{}]: {} {} {}", 
                    if Some(i) == selected { ">" } else { " " },
                    agent_name,
                    task.title,
                    status_icon,
                    priority_icon
                );

                ListItem::new(Line::from(Span::styled(content, style)))
            })
            .collect();

        let block = Block::default()
            .title("Tasks")
            .borders(Borders::ALL)
            .border_style(border_style());

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}