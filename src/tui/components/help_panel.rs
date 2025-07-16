use crate::tui::{app::Screen, utils::colors::*};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct HelpPanel;

impl HelpPanel {
    pub fn render(frame: &mut Frame, area: Rect, current_screen: &Screen) {
        let help_items = Self::get_help_items(current_screen);

        let items: Vec<ListItem> = help_items
            .iter()
            .map(|(key, description)| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("[{}] ", key), primary_style()),
                    Span::styled(*description, info_style()),
                ]))
            })
            .collect();

        let block = Block::default()
            .title("Help")
            .borders(Borders::ALL)
            .border_style(border_style());

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    fn get_help_items(screen: &Screen) -> Vec<(&'static str, &'static str)> {
        let mut items = vec![
            ("Q", "Quit application"),
            ("Tab", "Next screen"),
            ("1-5", "Jump to screen"),
            ("↑/↓", "Navigate lists"),
        ];

        match screen {
            Screen::Dashboard => {
                items.extend(vec![
                    ("R", "Refresh data"),
                    ("H", "Show/hide help"),
                ]);
            }
            Screen::Agents => {
                items.extend(vec![
                    ("N", "New agent"),
                    ("E", "Edit agent"),
                    ("D", "Delete agent"),
                    ("S", "Start agent"),
                    ("T", "Stop agent"),
                    ("R", "Restart agent"),
                    ("Enter", "View details"),
                ]);
            }
            Screen::Tasks => {
                items.extend(vec![
                    ("N", "New task"),
                    ("E", "Execute task"),
                    ("C", "Cancel task"),
                    ("U", "Update task"),
                    ("D", "Delete task"),
                    ("F", "Filter tasks"),
                    ("Enter", "View details"),
                ]);
            }
            Screen::Execution => {
                items.extend(vec![
                    ("C", "Cancel task"),
                    ("P", "Pause task"),
                    ("R", "Resume task"),
                    ("Space", "Pause/Resume"),
                    ("Del", "Cancel selected"),
                    ("Enter", "View output"),
                ]);
            }
            Screen::Logs => {
                items.extend(vec![
                    ("F", "Filter logs"),
                    ("C", "Clear logs"),
                    ("S", "Save logs"),
                    ("/", "Search logs"),
                    ("Space", "Toggle filter"),
                    ("Home/End", "Jump to top/bottom"),
                ]);
            }
        }

        items
    }
}