use crate::tui::{app::AppState, utils::{colors::*, formatting::*}};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct LogsScreen;

impl LogsScreen {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([
                Constraint::Length(20),
                Constraint::Min(40),
            ])
            .split(area);

        Self::render_log_filters(frame, chunks[0], state);
        Self::render_log_entries(frame, chunks[1], state);
    }

    fn render_log_filters(frame: &mut Frame, area: Rect, _state: &AppState) {
        let content = vec![
            Line::from(Span::styled("🏷️  Level:", secondary_style())),
            Line::from(vec![
                Span::styled("🟥 ", error_style()),
                Span::styled("ERROR", error_style()),
            ]),
            Line::from(vec![
                Span::styled("🟨 ", warning_style()),
                Span::styled("WARN", warning_style()),
            ]),
            Line::from(vec![
                Span::styled("🟦 ", info_style()),
                Span::styled("INFO", info_style()),
            ]),
            Line::from(vec![
                Span::styled("⚪ ", muted_style()),
                Span::styled("DEBUG", muted_style()),
            ]),
            Line::from(""),
            Line::from(Span::styled("📁 Source:", secondary_style())),
            Line::from(vec![
                Span::styled("⚡ ", primary_style()),
                Span::styled("System", text_primary_style()),
            ]),
            Line::from(vec![
                Span::styled("🤖 ", accent_style()),
                Span::styled("Agents", text_primary_style()),
            ]),
            Line::from(vec![
                Span::styled("📋 ", secondary_style()),
                Span::styled("Tasks", text_primary_style()),
            ]),
            Line::from(vec![
                Span::styled("🌐 ", info_style()),
                Span::styled("API", text_primary_style()),
            ]),
            Line::from(vec![
                Span::styled("⚪ ", muted_style()),
                Span::styled("WebSocket", muted_style()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("[R] ", highlight_style()),
                Span::styled("Refresh", text_primary_style()),
            ]),
            Line::from(vec![
                Span::styled("[A] ", highlight_style()),
                Span::styled("Auto-scroll", text_primary_style()),
            ]),
        ];

        let block = Block::default()
            .title("🗓️  Log Filters")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(primary_style());

        let paragraph = Paragraph::new(content).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_log_entries(frame: &mut Frame, area: Rect, _state: &AppState) {
        // Mock log entries - in a real implementation, these would come from the log system
        let log_entries = vec![
            ("[14:23:15]", "INFO", "Agent 'NewsBot' started task"),
            ("[14:23:10]", "INFO", "Task 'Generate report' queued"),
            ("[14:23:05]", "WARN", "High memory usage detected (85%)"),
            ("[14:22:58]", "INFO", "Agent 'CodeReviewer' went active"),
            ("[14:22:45]", "ERROR", "Task 'Update docs' failed"),
            ("[14:22:30]", "INFO", "System started successfully"),
            ("[14:22:15]", "INFO", "Registry initialized"),
            ("[14:22:10]", "INFO", "Git repository ready"),
            ("[14:22:05]", "INFO", "Claude CLI integration verified"),
            ("[14:22:00]", "INFO", "Configuration loaded"),
            ("[14:21:55]", "INFO", "Starting Nox v0.1.0"),
        ];

        let items: Vec<ListItem> = log_entries
            .iter()
            .map(|(time, level, message)| {
                let (level_icon, level_style) = match *level {
                    "ERROR" => ("🟥", error_style()),
                    "WARN" => ("🟨", warning_style()),
                    "INFO" => ("🟦", info_style()),
                    "DEBUG" => ("⚪", muted_style()),
                    _ => ("🟦", info_style()),
                };

                ListItem::new(Line::from(vec![
                    Span::styled(*time, text_secondary_style()),
                    Span::styled(" ", text_secondary_style()),
                    Span::styled(level_icon, level_style),
                    Span::styled(*level, level_style.add_modifier(Modifier::BOLD)),
                    Span::styled("  ", text_secondary_style()),
                    Span::styled(*message, text_primary_style()),
                ]))
            })
            .collect();

        let footer_text = format!(
            "[F] Filter  [C] Clear  [S] Save  [/] Search\n\
             Showing: 1,247 entries\n\
             Filtered: 247 entries"
        );

        let log_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(area);

        let block = Block::default()
            .title("📜 Log Entries")
            .borders(Borders::ALL)
            .border_style(border_active_style())
            .title_style(accent_style());

        let list = List::new(items).block(block);
        frame.render_widget(list, log_area[0]);

        let footer = Paragraph::new(footer_text).style(text_secondary_style());
        frame.render_widget(footer, log_area[1]);
    }
}