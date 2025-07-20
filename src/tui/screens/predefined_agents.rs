use crate::tui::app::AppState;
use crate::core::predefined_agents::PredefinedAgent;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub struct PredefinedAgentsScreen;

impl PredefinedAgentsScreen {
    pub fn render(f: &mut Frame, app_state: &mut AppState) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(4),
            ])
            .split(f.size());

        // Title
        Self::render_title(f, main_layout[0]);

        // Main content
        Self::render_main_content(f, main_layout[1], app_state);

        // Help/Status
        Self::render_help(f, main_layout[2]);
    }

    fn render_title(f: &mut Frame, area: ratatui::layout::Rect) {
        let title = Paragraph::new("🤖 Predefined Agents - Load Specialized AI Agents")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, area);
    }

    fn render_main_content(f: &mut Frame, area: ratatui::layout::Rect, app_state: &AppState) {
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Categories
                Constraint::Percentage(70), // Agents
            ])
            .split(area);

        // Render categories panel
        Self::render_categories(f, horizontal_layout[0], app_state);

        // Render agents panel
        Self::render_agents(f, horizontal_layout[1], app_state);
    }

    fn render_categories(f: &mut Frame, area: ratatui::layout::Rect, app_state: &AppState) {
        if app_state.predefined_categories.is_empty() {
            let empty_msg = Paragraph::new("No categories loaded.\nPress 'R' to refresh.")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default()
                    .title("📂 Categories")
                    .borders(Borders::ALL));
            f.render_widget(empty_msg, area);
            return;
        }

        let items: Vec<ListItem> = app_state.predefined_categories
            .iter()
            .enumerate()
            .map(|(i, category)| {
                let style = if Some(i) == app_state.selected_category {
                    Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(vec![
                    Line::from(Span::styled(
                        format!("📂 {}", category.name),
                        style
                    )),
                    Line::from(Span::styled(
                        category.description.clone(),
                        Style::default().fg(Color::Gray)
                    )),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title("📂 Categories (↑↓ to navigate)")
                .borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("▶ ");

        f.render_widget(list, area);
    }

    fn render_agents(f: &mut Frame, area: ratatui::layout::Rect, app_state: &AppState) {
        let agents_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),     // Agent list
                Constraint::Length(8),  // Agent details
            ])
            .split(area);

        // Render agent list
        Self::render_agent_list(f, agents_layout[0], app_state);

        // Render agent details
        Self::render_agent_details(f, agents_layout[1], app_state);
    }

    fn render_agent_list(f: &mut Frame, area: ratatui::layout::Rect, app_state: &AppState) {
        if app_state.predefined_agents.is_empty() {
            let empty_msg = Paragraph::new("No agents in this category.\nSelect a different category.")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default()
                    .title("🤖 Agents")
                    .borders(Borders::ALL));
            f.render_widget(empty_msg, area);
            return;
        }

        let items: Vec<ListItem> = app_state.predefined_agents
            .iter()
            .enumerate()
            .map(|(i, agent)| {
                let is_selected = Some(i) == app_state.selected_predefined_agent;
                let is_checked = i < app_state.predefined_agent_selection.len() 
                    && app_state.predefined_agent_selection[i];

                let checkbox = if is_checked { "☑" } else { "☐" };
                
                let name_style = if is_selected {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else if is_checked {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(format!("{} ", checkbox), 
                            if is_checked { Style::default().fg(Color::Green) } else { Style::default() }),
                        Span::styled(agent.name.clone(), name_style),
                    ]),
                    Line::from(Span::styled(
                        agent.description.clone(),
                        Style::default().fg(Color::Gray)
                    )),
                    Line::from(Span::styled(
                        format!("🎯 {} capabilities", agent.capabilities.len()),
                        Style::default().fg(Color::Magenta)
                    )),
                ])
            })
            .collect();

        let selected_count = app_state.predefined_agent_selection.iter().filter(|&&x| x).count();
        let title = format!("🤖 Agents (←→ navigate, Space=select, {} selected)", selected_count);

        let list = List::new(items)
            .block(Block::default()
                .title(title)
                .borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("▶ ");

        f.render_widget(list, area);
    }

    fn render_agent_details(f: &mut Frame, area: ratatui::layout::Rect, app_state: &AppState) {
        if let Some(selected_idx) = app_state.selected_predefined_agent {
            if let Some(agent) = app_state.predefined_agents.get(selected_idx) {
                let details_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(60), // Description and capabilities
                        Constraint::Percentage(40), // Default tasks
                    ])
                    .split(area);

                // Render agent description and capabilities
                Self::render_agent_info(f, details_layout[0], agent);

                // Render default tasks
                Self::render_agent_tasks(f, details_layout[1], agent);
            }
        } else {
            let placeholder = Paragraph::new("Select an agent to view details")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default()
                    .title("📋 Agent Details")
                    .borders(Borders::ALL));
            f.render_widget(placeholder, area);
        }
    }

    fn render_agent_info(f: &mut Frame, area: ratatui::layout::Rect, agent: &PredefinedAgent) {
        let mut content = vec![
            Line::from(Span::styled(
                &agent.name,
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            )),
            Line::from(""),
            Line::from(Span::styled("Description:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(Span::raw(&agent.description)),
            Line::from(""),
            Line::from(Span::styled("Capabilities:", Style::default().add_modifier(Modifier::BOLD))),
        ];

        for capability in &agent.capabilities {
            content.push(Line::from(format!("• {}", capability)));
        }

        let paragraph = Paragraph::new(content)
            .block(Block::default()
                .title("📋 Agent Details")
                .borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_agent_tasks(f: &mut Frame, area: ratatui::layout::Rect, agent: &PredefinedAgent) {
        let items: Vec<ListItem> = agent.default_tasks
            .iter()
            .map(|task| {
                ListItem::new(vec![
                    Line::from(Span::styled(
                        &task.title,
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    )),
                    Line::from(Span::styled(
                        &task.description,
                        Style::default().fg(Color::Gray)
                    )),
                    Line::from(format!("⏱️ {} | 🎯 {}", task.estimated_duration, task.priority)),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title("📋 Default Tasks")
                .borders(Borders::ALL));

        f.render_widget(list, area);
    }

    fn render_help(f: &mut Frame, area: ratatui::layout::Rect) {
        let help_text = vec![
            Line::from(vec![
                Span::styled("Navigation: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("↑↓ Categories | ←→ Agents | Space Select | A Select All | N Clear All"),
            ]),
            Line::from(vec![
                Span::styled("Actions: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("Enter Load Selected | R Refresh | Tab Switch Screen | Q Quit"),
            ]),
        ];

        let help = Paragraph::new(help_text)
            .block(Block::default()
                .title("🔧 Help")
                .borders(Borders::ALL))
            .alignment(Alignment::Left);

        f.render_widget(help, area);
    }
}