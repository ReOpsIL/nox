use crate::core::{agent_manager, task_manager};
use crate::types::{Agent, Task, SystemStatus, TaskStatus};
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    Agents,
    Tasks,
    Execution,
    Logs,
}

#[derive(Debug, Clone)]
pub struct FilterState {
    pub agent_status_filter: Option<String>,
    pub task_status_filter: Option<String>,
    pub search_query: String,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            agent_status_filter: None,
            task_status_filter: None,
            search_query: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub current_screen: Screen,
    pub agents: Vec<Agent>,
    pub tasks: Vec<Task>,
    pub selected_agent: Option<usize>,
    pub selected_task: Option<usize>,
    pub system_status: SystemStatus,
    pub filters: FilterState,
    pub should_quit: bool,
    pub active_panel: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::Dashboard,
            agents: Vec::new(),
            tasks: Vec::new(),
            selected_agent: None,
            selected_task: None,
            system_status: SystemStatus {
                active_agents: 0,
                inactive_agents: 0,
                tasks_by_status: HashMap::new(),
                uptime_seconds: 0,
                memory_usage_mb: 0,
                cpu_usage_percent: 0.0,
            },
            filters: FilterState::default(),
            should_quit: false,
            active_panel: 0,
        }
    }
}

pub struct App {
    pub state: AppState,
    event_sender: mpsc::UnboundedSender<crate::tui::events::AppEvent>,
}

impl App {
    pub fn new(
        event_sender: mpsc::UnboundedSender<crate::tui::events::AppEvent>,
    ) -> Self {
        Self {
            state: AppState::default(),
            event_sender,
        }
    }

    pub async fn refresh_data(&mut self) -> Result<()> {
        self.state.agents = agent_manager::get_all_agents().await?;
        self.state.tasks = task_manager::get_all_tasks().await?;
        
        let active_agents = self.state.agents.iter()
            .filter(|a| matches!(a.status, crate::types::AgentStatus::Active))
            .count();
        let inactive_agents = self.state.agents.len() - active_agents;

        let mut tasks_by_status = HashMap::new();
        for task in &self.state.tasks {
            *tasks_by_status.entry(task.status.clone()).or_insert(0) += 1;
        }

        self.state.system_status = SystemStatus {
            active_agents,
            inactive_agents,
            tasks_by_status,
            uptime_seconds: 0, // TODO: Track actual uptime
            memory_usage_mb: 0, // TODO: Get actual memory usage
            cpu_usage_percent: 0.0, // TODO: Get actual CPU usage
        };

        Ok(())
    }

    pub fn quit(&mut self) {
        self.state.should_quit = true;
    }

    pub fn next_screen(&mut self) {
        self.state.current_screen = match self.state.current_screen {
            Screen::Dashboard => Screen::Agents,
            Screen::Agents => Screen::Tasks,
            Screen::Tasks => Screen::Execution,
            Screen::Execution => Screen::Logs,
            Screen::Logs => Screen::Dashboard,
        };
    }

    pub fn previous_screen(&mut self) {
        self.state.current_screen = match self.state.current_screen {
            Screen::Dashboard => Screen::Logs,
            Screen::Agents => Screen::Dashboard,
            Screen::Tasks => Screen::Agents,
            Screen::Execution => Screen::Tasks,
            Screen::Logs => Screen::Execution,
        };
    }

    pub fn handle_key_input(&mut self, key: crossterm::event::KeyCode) {
        match key {
            crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('Q') => {
                self.quit();
            }
            crossterm::event::KeyCode::Tab => {
                self.next_screen();
            }
            crossterm::event::KeyCode::BackTab => {
                self.previous_screen();
            }
            crossterm::event::KeyCode::Char('1') => {
                self.state.current_screen = Screen::Dashboard;
            }
            crossterm::event::KeyCode::Char('2') => {
                self.state.current_screen = Screen::Agents;
            }
            crossterm::event::KeyCode::Char('3') => {
                self.state.current_screen = Screen::Tasks;
            }
            crossterm::event::KeyCode::Char('4') => {
                self.state.current_screen = Screen::Execution;
            }
            crossterm::event::KeyCode::Char('5') => {
                self.state.current_screen = Screen::Logs;
            }
            crossterm::event::KeyCode::Up => {
                self.handle_list_navigation(-1);
            }
            crossterm::event::KeyCode::Down => {
                self.handle_list_navigation(1);
            }
            _ => {}
        }
    }

    fn handle_list_navigation(&mut self, direction: i32) {
        match self.state.current_screen {
            Screen::Agents => {
                let agent_count = self.state.agents.len();
                if agent_count > 0 {
                    let current = self.state.selected_agent.unwrap_or(0);
                    let new_index = if direction > 0 {
                        (current + 1) % agent_count
                    } else {
                        if current == 0 { agent_count - 1 } else { current - 1 }
                    };
                    self.state.selected_agent = Some(new_index);
                }
            }
            Screen::Tasks => {
                let task_count = self.state.tasks.len();
                if task_count > 0 {
                    let current = self.state.selected_task.unwrap_or(0);
                    let new_index = if direction > 0 {
                        (current + 1) % task_count
                    } else {
                        if current == 0 { task_count - 1 } else { current - 1 }
                    };
                    self.state.selected_task = Some(new_index);
                }
            }
            _ => {}
        }
    }
}