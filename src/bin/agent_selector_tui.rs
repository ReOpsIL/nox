use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AgentConfig {
    version: String,
    metadata: ConfigMetadata,
    categories: Vec<Category>,
    agents: Vec<Agent>,
    task_templates: HashMap<String, TaskTemplate>,
    filters: Filters,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ConfigMetadata {
    created_at: String,
    description: String,
    author: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Category {
    id: String,
    name: String,
    description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Agent {
    id: String,
    name: String,
    category: String,
    description: String,
    system_prompt: String,
    capabilities: Vec<String>,
    default_tasks: Vec<DefaultTask>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DefaultTask {
    template: String,
    title: String,
    description: String,
    priority: String,
    estimated_duration: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TaskTemplate {
    title: String,
    description: String,
    category: String,
    tags: Vec<String>,
    default_priority: String,
    estimated_duration: String,
    steps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Filters {
    by_category: Vec<String>,
    by_technology: Vec<String>,
    by_complexity: Vec<String>,
    by_duration: Vec<String>,
}

#[derive(Debug, Clone)]
enum AppState {
    CategorySelection,
    AgentSelection,
    AgentDetails,
    TaskSelection,
    TaskDetails,
    OutputGeneration,
}

#[derive(Debug, Clone)]
struct SelectedAgent {
    agent: Agent,
    selected_tasks: Vec<DefaultTask>,
}

struct App {
    config: AgentConfig,
    state: AppState,
    category_list_state: ListState,
    agent_list_state: ListState,
    task_list_state: ListState,
    filtered_categories: Vec<Category>,
    filtered_agents: Vec<Agent>,
    current_agent: Option<Agent>,
    selected_agents: Vec<SelectedAgent>,
    filter_category: Option<String>,
    should_quit: bool,
    show_help: bool,
}

impl App {
    fn new(config: AgentConfig) -> Self {
        let mut app = Self {
            filtered_categories: config.categories.clone(),
            filtered_agents: vec![],
            config,
            state: AppState::CategorySelection,
            category_list_state: ListState::default(),
            agent_list_state: ListState::default(),
            task_list_state: ListState::default(),
            current_agent: None,
            selected_agents: vec![],
            filter_category: None,
            should_quit: false,
            show_help: false,
        };
        app.category_list_state.select(Some(0));
        app
    }

    fn next_category(&mut self) {
        let i = match self.category_list_state.selected() {
            Some(i) => {
                if i >= self.filtered_categories.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.category_list_state.select(Some(i));
    }

    fn previous_category(&mut self) {
        let i = match self.category_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_categories.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.category_list_state.select(Some(i));
    }

    fn next_agent(&mut self) {
        let i = match self.agent_list_state.selected() {
            Some(i) => {
                if i >= self.filtered_agents.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.agent_list_state.select(Some(i));
    }

    fn previous_agent(&mut self) {
        let i = match self.agent_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_agents.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.agent_list_state.select(Some(i));
    }

    fn next_task(&mut self) {
        if let Some(agent) = &self.current_agent {
            let i = match self.task_list_state.selected() {
                Some(i) => {
                    if i >= agent.default_tasks.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.task_list_state.select(Some(i));
        }
    }

    fn previous_task(&mut self) {
        if let Some(agent) = &self.current_agent {
            let i = match self.task_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        agent.default_tasks.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.task_list_state.select(Some(i));
        }
    }

    fn select_category(&mut self) {
        if let Some(i) = self.category_list_state.selected() {
            if let Some(category) = self.filtered_categories.get(i) {
                self.filter_category = Some(category.id.clone());
                self.filtered_agents = self.config.agents
                    .iter()
                    .filter(|agent| agent.category == category.id)
                    .cloned()
                    .collect();
                self.agent_list_state.select(Some(0));
                self.state = AppState::AgentSelection;
            }
        }
    }

    fn select_agent(&mut self) {
        if let Some(i) = self.agent_list_state.selected() {
            if let Some(agent) = self.filtered_agents.get(i) {
                self.current_agent = Some(agent.clone());
                self.task_list_state.select(Some(0));
                self.state = AppState::AgentDetails;
            }
        }
    }

    fn view_tasks(&mut self) {
        if self.current_agent.is_some() {
            self.state = AppState::TaskSelection;
        }
    }

    fn select_task(&mut self) {
        if let Some(agent) = &self.current_agent {
            if let Some(i) = self.task_list_state.selected() {
                if let Some(task) = agent.default_tasks.get(i) {
                    // Add agent with selected task to selection
                    let selected_agent = SelectedAgent {
                        agent: agent.clone(),
                        selected_tasks: vec![task.clone()],
                    };
                    self.selected_agents.push(selected_agent);
                    self.state = AppState::OutputGeneration;
                }
            }
        }
    }

    fn generate_output(&self) -> Result<String> {
        let mut output = String::new();
        output.push_str("# Selected Agent Configuration\n\n");
        
        for selected_agent in &self.selected_agents {
            output.push_str(&format!("## Agent: {}\n", selected_agent.agent.name));
            output.push_str(&format!("**Category:** {}\n", selected_agent.agent.category));
            output.push_str(&format!("**Description:** {}\n\n", selected_agent.agent.description));
            
            output.push_str("### Selected Tasks:\n");
            for task in &selected_agent.selected_tasks {
                output.push_str(&format!("- **{}**: {}\n", task.title, task.description));
                output.push_str(&format!("  - Priority: {}\n", task.priority));
                output.push_str(&format!("  - Duration: {}\n", task.estimated_duration));
            }
            output.push_str("\n");
        }

        // Generate YAML output
        output.push_str("---\n\n");
        output.push_str("# YAML Configuration for Selected Agents\n");
        output.push_str("```yaml\n");
        output.push_str("selected_agents:\n");
        
        for selected_agent in &self.selected_agents {
            output.push_str(&format!("  - id: \"{}\"\n", selected_agent.agent.id));
            output.push_str(&format!("    name: \"{}\"\n", selected_agent.agent.name));
            output.push_str(&format!("    category: \"{}\"\n", selected_agent.agent.category));
            output.push_str("    tasks:\n");
            for task in &selected_agent.selected_tasks {
                output.push_str(&format!("      - template: \"{}\"\n", task.template));
                output.push_str(&format!("        title: \"{}\"\n", task.title));
                output.push_str(&format!("        priority: \"{}\"\n", task.priority));
            }
        }
        output.push_str("```\n");

        Ok(output)
    }

    fn back(&mut self) {
        match self.state {
            AppState::CategorySelection => {}
            AppState::AgentSelection => {
                self.state = AppState::CategorySelection;
                self.filter_category = None;
                self.filtered_agents.clear();
            }
            AppState::AgentDetails => {
                self.state = AppState::AgentSelection;
                self.current_agent = None;
            }
            AppState::TaskSelection => {
                self.state = AppState::AgentDetails;
            }
            AppState::TaskDetails => {
                self.state = AppState::TaskSelection;
            }
            AppState::OutputGeneration => {
                self.state = AppState::TaskSelection;
            }
        }
    }
}

fn main() -> Result<()> {
    // Load configuration
    let config_path = "config/predefined_agents.yaml";
    let config_content = fs::read_to_string(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", config_path, e))?;
    
    let config: AgentConfig = serde_yaml::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse YAML config: {}", e))?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = App::new(config);
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('h') => {
                        app.show_help = !app.show_help;
                    }
                    KeyCode::Down => {
                        match app.state {
                            AppState::CategorySelection => app.next_category(),
                            AppState::AgentSelection => app.next_agent(),
                            AppState::TaskSelection => app.next_task(),
                            _ => {}
                        }
                    }
                    KeyCode::Up => {
                        match app.state {
                            AppState::CategorySelection => app.previous_category(),
                            AppState::AgentSelection => app.previous_agent(),
                            AppState::TaskSelection => app.previous_task(),
                            _ => {}
                        }
                    }
                    KeyCode::Enter => {
                        match app.state {
                            AppState::CategorySelection => app.select_category(),
                            AppState::AgentSelection => app.select_agent(),
                            AppState::AgentDetails => app.view_tasks(),
                            AppState::TaskSelection => app.select_task(),
                            _ => {}
                        }
                    }
                    KeyCode::Char('s') => {
                        if matches!(app.state, AppState::OutputGeneration) {
                            // Save output to file
                            if let Ok(output) = app.generate_output() {
                                let filename = format!("selected_agents_{}.md", 
                                    chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                                if fs::write(&filename, output).is_ok() {
                                    app.should_quit = true;
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        if app.show_help {
                            app.show_help = false;
                        } else {
                            app.back();
                        }
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("🤖 AI Agent Selector")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Main content based on state
    match app.state {
        AppState::CategorySelection => {
            render_category_selection(f, chunks[1], app);
        }
        AppState::AgentSelection => {
            render_agent_selection(f, chunks[1], app);
        }
        AppState::AgentDetails => {
            render_agent_details(f, chunks[1], app);
        }
        AppState::TaskSelection => {
            render_task_selection(f, chunks[1], app);
        }
        AppState::OutputGeneration => {
            render_output_generation(f, chunks[1], app);
        }
        _ => {}
    }

    // Status bar
    let status_text = match app.state {
        AppState::CategorySelection => "Select Category | ↑↓: Navigate, Enter: Select, h: Help, q: Quit",
        AppState::AgentSelection => "Select Agent | ↑↓: Navigate, Enter: Select, Esc: Back",
        AppState::AgentDetails => "Agent Details | Enter: View Tasks, Esc: Back",
        AppState::TaskSelection => "Select Task | ↑↓: Navigate, Enter: Select, Esc: Back",
        AppState::OutputGeneration => "Output Generated | s: Save to file, Esc: Back",
        _ => "Navigation: ↑↓, Select: Enter, Back: Esc, Help: h, Quit: q",
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);

    // Help popup
    if app.show_help {
        render_help_popup(f);
    }
}

fn render_category_selection(f: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .filtered_categories
        .iter()
        .map(|category| {
            ListItem::new(vec![
                Line::from(Span::styled(
                    &category.name,
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::raw(&category.description)),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📂 Agent Categories")
        )
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.category_list_state);
}

fn render_agent_selection(f: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .filtered_agents
        .iter()
        .map(|agent| {
            ListItem::new(vec![
                Line::from(Span::styled(
                    &agent.name,
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::raw(&agent.description)),
                Line::from(Span::styled(
                    format!("🎯 {} capabilities", agent.capabilities.len()),
                    Style::default().fg(Color::Magenta),
                )),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("🤖 Agents - {}", 
                    app.filter_category.as_ref().unwrap_or(&"All".to_string())))
        )
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.agent_list_state);
}

fn render_agent_details(f: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    if let Some(agent) = &app.current_agent {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // Agent details
        let mut details = vec![
            Line::from(Span::styled(
                &agent.name,
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled("📝 Description:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(Span::raw(&agent.description)),
            Line::from(""),
            Line::from(Span::styled("🎯 Capabilities:", Style::default().add_modifier(Modifier::BOLD))),
        ];

        for capability in &agent.capabilities {
            details.push(Line::from(format!("• {}", capability)));
        }

        let details_paragraph = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title("Agent Details"))
            .wrap(Wrap { trim: true });

        f.render_widget(details_paragraph, chunks[0]);

        // Default tasks
        let task_items: Vec<ListItem> = agent
            .default_tasks
            .iter()
            .map(|task| {
                ListItem::new(vec![
                    Line::from(Span::styled(
                        &task.title,
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::raw(&task.description)),
                    Line::from(format!("⏱️ {} | 🎯 {}", task.estimated_duration, task.priority)),
                ])
            })
            .collect();

        let task_list = List::new(task_items)
            .block(Block::default().borders(Borders::ALL).title("📋 Default Tasks"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("▶ ");

        f.render_widget(task_list, chunks[1]);
    }
}

fn render_task_selection(f: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    if let Some(agent) = &app.current_agent {
        let items: Vec<ListItem> = agent
            .default_tasks
            .iter()
            .map(|task| {
                ListItem::new(vec![
                    Line::from(Span::styled(
                        &task.title,
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::raw(&task.description)),
                    Line::from(format!("⏱️ {} | 🎯 {} | 📦 {}", 
                        task.estimated_duration, task.priority, task.template)),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("📋 Select Task for {}", agent.name))
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut app.task_list_state);
    }
}

fn render_output_generation(f: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    if let Ok(output) = app.generate_output() {
        let paragraph = Paragraph::new(output)
            .block(Block::default().borders(Borders::ALL).title("📄 Generated Configuration"))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}

fn render_help_popup(f: &mut Frame) {
    let area = centered_rect(80, 60, f.size());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("🔧 AI Agent Selector Help", 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  ↑↓    - Navigate through lists"),
        Line::from("  Enter - Select item"),
        Line::from("  Esc   - Go back / Close help"),
        Line::from("  h     - Toggle this help"),
        Line::from("  q     - Quit application"),
        Line::from(""),
        Line::from("Workflow:"),
        Line::from("  1. Select a category"),
        Line::from("  2. Choose an agent"),
        Line::from("  3. View agent details"),
        Line::from("  4. Select tasks"),
        Line::from("  5. Generate configuration"),
        Line::from("  6. Save output (press 's')"),
        Line::from(""),
        Line::from("Features:"),
        Line::from("  • Browse predefined agents"),
        Line::from("  • Filter by category"),
        Line::from("  • View agent capabilities"),
        Line::from("  • Select tasks"),
        Line::from("  • Generate YAML config"),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });

    f.render_widget(help_paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}