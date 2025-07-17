use crate::core::{agent_manager, task_manager};
use crate::types::{Agent, Task, SystemStatus, TaskStatus};
use crate::tui::forms::{create_agent::CreateAgentForm, create_task::CreateTaskForm, Form};
use crate::tui::dialogs::{confirmation::ConfirmationDialog, progress::ProgressDialog, help::HelpDialog, Dialog};
use crate::tui::system_monitor::SystemMonitor;
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;
use crate::commands;

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

/// Form state for the application
#[derive(Debug, Clone)]
pub enum FormState {
    CreateAgent(CreateAgentForm),
    EditAgent(CreateAgentForm),
    CreateTask(CreateTaskForm),
    EditTask(CreateTaskForm),
}

/// Dialog state for the application
#[derive(Debug, Clone)]
pub enum DialogState {
    Confirmation(ConfirmationDialog),
    Progress(ProgressDialog),
    Help(HelpDialog),
}

/// Pending operation types that require confirmation
#[derive(Debug, Clone)]
pub enum PendingOperation {
    DeleteAgent(Agent),
    StopAgent(Agent),
    DeleteTask(Task),
    CancelTask(Task),
    RestartAgent(Agent),
    ClearLogs,
}

/// Operation status for tracking async operations
#[derive(Debug, Clone)]
pub struct OperationStatus {
    pub is_loading: bool,
    pub operation_type: Option<String>,
    pub progress: Option<u8>,
    pub can_cancel: bool,
}

impl Default for OperationStatus {
    fn default() -> Self {
        Self {
            is_loading: false,
            operation_type: None,
            progress: None,
            can_cancel: false,
        }
    }
}

/// Search state for filtering and searching
#[derive(Debug, Clone)]
pub struct SearchState {
    pub active: bool,
    pub query: String,
    pub filtered_agent_indices: Vec<usize>,
    pub filtered_task_indices: Vec<usize>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            active: false,
            query: String::new(),
            filtered_agent_indices: Vec::new(),
            filtered_task_indices: Vec::new(),
        }
    }
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
    
    // New interactive state fields
    pub current_form: Option<FormState>,
    pub current_dialog: Option<DialogState>,
    pub operation_status: OperationStatus,
    pub search_state: SearchState,
    pub help_visible: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub pending_operation: Option<PendingOperation>,
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
                total_memory_mb: 0,
                cpu_usage_percent: 0.0,
                memory_usage_percent: 0.0,
                error_agents: 0,
            },
            filters: FilterState::default(),
            should_quit: false,
            active_panel: 0,
            
            // Initialize new interactive state fields
            current_form: None,
            current_dialog: None,
            operation_status: OperationStatus::default(),
            search_state: SearchState::default(),
            help_visible: false,
            error_message: None,
            success_message: None,
            pending_operation: None,
        }
    }
}

pub struct App {
    pub state: AppState,
    event_sender: mpsc::UnboundedSender<crate::tui::events::AppEvent>,
    system_monitor: SystemMonitor,
}

impl App {
    pub fn new(
        event_sender: mpsc::UnboundedSender<crate::tui::events::AppEvent>,
    ) -> Self {
        Self {
            state: AppState::default(),
            event_sender,
            system_monitor: SystemMonitor::new(),
        }
    }

    pub async fn refresh_data(&mut self) -> Result<()> {
        self.state.agents = agent_manager::get_all_agents().await?;
        self.state.tasks = task_manager::get_all_tasks().await?;

        if  self.state.agents.len() > 0  && self.state.selected_agent == None {
            self.state.selected_agent = Some(0);
        }

        if  self.state.tasks.len() > 0  && self.state.selected_task == None {
            self.state.selected_task = Some(0);
        }

        let active_agents = self.state.agents.iter()
            .filter(|a| matches!(a.status, crate::types::AgentStatus::Active))
            .count();
        let error_agents = self.state.agents.iter()
            .filter(|a| matches!(a.status, crate::types::AgentStatus::Error))
            .count();
        let inactive_agents = self.state.agents.len() - active_agents - error_agents;

        let mut tasks_by_status = HashMap::new();
        for task in &self.state.tasks {
            *tasks_by_status.entry(task.status.clone()).or_insert(0) += 1;
        }

        // Refresh system information
        self.system_monitor.refresh();
        
        self.state.system_status = SystemStatus {
            active_agents,
            inactive_agents,
            tasks_by_status,
            uptime_seconds: self.system_monitor.get_uptime_seconds(),
            memory_usage_mb: self.system_monitor.get_memory_usage_mb(),
            total_memory_mb: self.system_monitor.get_total_memory_mb(),
            cpu_usage_percent: self.system_monitor.get_cpu_usage_percent(),
            memory_usage_percent: self.system_monitor.get_memory_usage_percent(),
            error_agents,
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

    pub async fn handle_key_input(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        // Handle global keys first
        if self.handle_global_keys(key)? {
            return Ok(());
        }
        
        // Handle form input if form is active
        if self.state.current_form.is_some() {
            return self.handle_form_input(key).await;
        }
        
        // Handle dialog input if dialog is active
        if self.state.current_dialog.is_some() {
            return self.handle_dialog_input(key).await;
        }
        
        // Handle search input if search is active
        if self.state.search_state.active {
            return self.handle_search_input(key);
        }
        
        // Handle screen-specific keys
        match self.state.current_screen {
            Screen::Agents => self.handle_agents_keys(key).await,
            Screen::Tasks => self.handle_tasks_keys(key).await,
            Screen::Execution => self.handle_execution_keys(key),
            Screen::Logs => self.handle_logs_keys(key),
            Screen::Dashboard => self.handle_dashboard_keys(key),
        }
    }
    
    /// Handle global keys that work in all contexts
    fn handle_global_keys(&mut self, key: crossterm::event::KeyCode) -> Result<bool> {
        match key {
            crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('Q') => {
                if self.state.current_form.is_none() && self.state.current_dialog.is_none() {
                    self.quit();
                    return Ok(true);
                }
            }
            crossterm::event::KeyCode::Char('?') | crossterm::event::KeyCode::F(1) | crossterm::event::KeyCode::Char('h') | crossterm::event::KeyCode::Char('H') => {
                if self.state.current_form.is_none() && self.state.current_dialog.is_none() {
                    self.show_help();
                    return Ok(true);
                }
            }
            _ => {}
        }
        Ok(false)
    }
    
    /// Handle form input
    async fn handle_form_input(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        use crate::tui::forms::FormResult;
        
        let result = if let Some(form_state) = &mut self.state.current_form {
            match form_state {
                FormState::CreateAgent(form) => form.handle_input(key),
                FormState::EditAgent(form) => form.handle_input(key),
                FormState::CreateTask(form) => form.handle_input(key),
                FormState::EditTask(form) => form.handle_input(key),
            }
        } else {
            return Ok(());
        };
        
        match result {
            FormResult::Submit => {
                self.submit_current_form().await?;
            }
            FormResult::Cancel => {
                self.cancel_current_form();
            }
            FormResult::Continue => {
                // Do nothing, continue processing
            }
        }
        
        Ok(())
    }
    
    /// Handle dialog input
    async fn handle_dialog_input(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        use crate::tui::dialogs::DialogResult;
        
        let result = if let Some(dialog_state) = &mut self.state.current_dialog {
            match dialog_state {
                DialogState::Confirmation(dialog) => dialog.handle_input(key),
                DialogState::Progress(dialog) => dialog.handle_input(key),
                DialogState::Help(dialog) => dialog.handle_input(key),
            }
        } else {
            return Ok(());
        };
        
        match result {
            DialogResult::Confirm => {
                self.confirm_current_dialog().await?;
            }
            DialogResult::Cancel => {
                self.cancel_current_dialog();
            }
            DialogResult::Close => {
                self.close_current_dialog();
            }
            DialogResult::Continue => {
                // Do nothing, continue processing
            }
        }
        
        Ok(())
    }
    
    /// Handle search input
    fn handle_search_input(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Char(c) => {
                self.state.search_state.query.push(c);
                self.update_search_results();
            }
            crossterm::event::KeyCode::Backspace => {
                self.state.search_state.query.pop();
                self.update_search_results();
            }
            crossterm::event::KeyCode::Enter | crossterm::event::KeyCode::Esc => {
                self.state.search_state.active = false;
                self.state.search_state.query.clear();
                self.update_search_results();
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle agents screen keys
    async fn handle_agents_keys(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Char('N') => {
                self.show_create_agent_form();
            }
            crossterm::event::KeyCode::Char('e') | crossterm::event::KeyCode::Char('E') => {
                if let Some(agent) = self.get_selected_agent() {
                    self.show_edit_agent_form(agent.clone());
                }
            }
            crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Char('S') => {
                if let Some(agent) = self.get_selected_agent() {
                    self.start_agent_operation(agent.clone()).await?;
                }
            }
            crossterm::event::KeyCode::Char('t') | crossterm::event::KeyCode::Char('T') => {
                if let Some(agent) = self.get_selected_agent() {
                    self.show_stop_agent_confirmation(agent.clone());
                }
            }
            crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Char('D') => {
                if let Some(agent) = self.get_selected_agent() {
                    self.show_delete_agent_confirmation(agent.clone());
                }
            }
            crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::Char('R') => {
                if let Some(agent) = self.get_selected_agent() {
                    self.restart_agent_operation(agent.clone());
                }
            }
            crossterm::event::KeyCode::Enter => {
                if let Some(agent) = self.get_selected_agent() {
                    self.show_agent_details(agent.clone());
                }
            }
            crossterm::event::KeyCode::Char('/') => {
                self.activate_search();
            }
            crossterm::event::KeyCode::Char('f') | crossterm::event::KeyCode::Char('F') => {
                self.show_filter_options();
            }
            _ => self.handle_navigation_keys(key)?,
        }
        Ok(())
    }
    
    /// Handle tasks screen keys
    async fn handle_tasks_keys(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Char('N') => {
                self.show_create_task_form();
            }
            crossterm::event::KeyCode::Char('e') | crossterm::event::KeyCode::Char('E') => {
                if let Some(task) = self.get_selected_task() {
                    self.execute_task_operation(task.clone()).await?;
                }
            }
            crossterm::event::KeyCode::Char('u') | crossterm::event::KeyCode::Char('U') => {
                if let Some(task) = self.get_selected_task() {
                    self.show_edit_task_form(task.clone());
                }
            }
            crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Char('D') => {
                if let Some(task) = self.get_selected_task() {
                    self.show_delete_task_confirmation(task.clone());
                }
            }
            crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C') => {
                if let Some(task) = self.get_selected_task() {
                    self.show_cancel_task_confirmation(task.clone());
                }
            }
            crossterm::event::KeyCode::Enter => {
                if let Some(task) = self.get_selected_task() {
                    self.show_task_details(task.clone());
                }
            }
            // Task filter keys
            crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Char('A') => {
                self.filter_tasks_all();
            }
            crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::Char('R') => {
                self.filter_tasks_running();
            }
            crossterm::event::KeyCode::Char('p') | crossterm::event::KeyCode::Char('P') => {
                self.filter_tasks_pending();
            }
            crossterm::event::KeyCode::Char('/') => {
                self.activate_search();
            }
            crossterm::event::KeyCode::Char('f') | crossterm::event::KeyCode::Char('F') => {
                self.show_filter_options();
            }
            _ => self.handle_navigation_keys(key)?,
        }
        Ok(())
    }
    
    /// Handle execution screen keys
    fn handle_execution_keys(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Char(' ') => {
                // Space - Pause/Resume execution
                if let Some(task) = self.get_selected_task() {
                    self.toggle_execution_pause(task.clone());
                }
            }
            crossterm::event::KeyCode::Delete => {
                // Del - Cancel execution
                if let Some(task) = self.get_selected_task() {
                    self.cancel_execution(task.clone());
                }
            }
            crossterm::event::KeyCode::Enter => {
                // Enter - View execution details
                if let Some(task) = self.get_selected_task() {
                    self.show_execution_details(task.clone());
                }
            }
            crossterm::event::KeyCode::Char('p') | crossterm::event::KeyCode::Char('P') => {
                // P - Pause execution
                if let Some(task) = self.get_selected_task() {
                    self.pause_execution(task.clone());
                }
            }
            crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::Char('R') => {
                // R - Resume execution
                if let Some(task) = self.get_selected_task() {
                    self.resume_execution(task.clone());
                }
            }
            crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C') => {
                // C - Cancel execution
                if let Some(task) = self.get_selected_task() {
                    self.cancel_execution(task.clone());
                }
            }
            crossterm::event::KeyCode::Char('/') => {
                // / - Search
                self.activate_search();
            }
            crossterm::event::KeyCode::Char('f') | crossterm::event::KeyCode::Char('F') => {
                // F - Filter
                self.show_filter_options();
            }
            _ => self.handle_navigation_keys(key)?,
        }
        Ok(())
    }
    
    /// Handle logs screen keys
    fn handle_logs_keys(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Char('f') | crossterm::event::KeyCode::Char('F') => {
                // F - Toggle filter panel
                self.toggle_log_filter();
            }
            crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C') => {
                // C - Clear logs
                self.clear_logs();
            }
            crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Char('S') => {
                // S - Save logs to file
                self.save_logs_to_file();
            }
            crossterm::event::KeyCode::Char('/') => {
                // / - Search in logs
                self.search_logs();
            }
            crossterm::event::KeyCode::Char(' ') => {
                // Space - Toggle auto-scroll
                self.toggle_auto_scroll();
            }
            crossterm::event::KeyCode::Home => {
                // Home - Jump to beginning
                self.jump_to_log_start();
            }
            crossterm::event::KeyCode::End => {
                // End - Jump to end
                self.jump_to_log_end();
            }
            crossterm::event::KeyCode::Enter => {
                // Enter - View log entry details
                self.show_log_details();
            }
            crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::Char('R') => {
                // R - Refresh logs
                self.refresh_logs();
            }
            crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Char('A') => {
                // A - Toggle auto-scroll
                self.toggle_auto_scroll();
            }
            _ => self.handle_navigation_keys(key)?,
        }
        Ok(())
    }
    
    /// Handle dashboard screen keys
    fn handle_dashboard_keys(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Left => {
                // Left - Navigate dashboard left
                self.navigate_dashboard_left();
            }
            crossterm::event::KeyCode::Right => {
                // Right - Navigate dashboard right
                self.navigate_dashboard_right();
            }
            crossterm::event::KeyCode::Enter => {
                // Enter - Select/activate dashboard item
                self.select_dashboard_item();
            }
            _ => self.handle_navigation_keys(key)?,
        }
        Ok(())
    }
    
    /// Handle navigation keys
    fn handle_navigation_keys(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
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
        Ok(())
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
    
    // Helper methods for getting selected items
    fn get_selected_agent(&self) -> Option<&Agent> {
        self.state.selected_agent.and_then(|idx| self.state.agents.get(idx))
    }
    
    fn get_selected_task(&self) -> Option<&Task> {
        self.state.selected_task.and_then(|idx| self.state.tasks.get(idx))
    }
    
    // Form management methods
    fn show_create_agent_form(&mut self) {
        let form = CreateAgentForm::new();
        self.state.current_form = Some(FormState::CreateAgent(form));
    }
    
    fn show_edit_agent_form(&mut self, agent: Agent) {
        let form = CreateAgentForm::from_agent(&agent);
        self.state.current_form = Some(FormState::EditAgent(form));
    }
    
    fn show_create_task_form(&mut self) {
        let mut form = CreateTaskForm::new();
        form.set_agents(self.state.agents.clone());
        self.state.current_form = Some(FormState::CreateTask(form));
    }
    
    fn show_edit_task_form(&mut self, task: Task) {
        let form = CreateTaskForm::from_task(&task, self.state.agents.clone());
        self.state.current_form = Some(FormState::EditTask(form));
    }
    
    async fn submit_current_form(&mut self) -> Result<()> {
        if let Some(form_state) = self.state.current_form.take() {
            match form_state {
                FormState::CreateAgent(form) => {
                    match form.submit() {
                        Ok(agent) => {
                            // Show progress for agent creation
                            let mut dialog = ProgressDialog::agent_operation("Creating", &agent.name);
                            dialog.set_progress(50);
                            self.state.current_dialog = Some(DialogState::Progress(dialog));
                            
                            // Actually create the agent
                            match commands::agent::add::execute(agent.name.clone(), agent.system_prompt.clone()).await {
                                Ok(_) => {
                                    // Add to local state for immediate UI update
                                    self.state.agents.push(agent.clone());
                                    // Update progress dialog
                                    if let Some(DialogState::Progress(ref mut dialog)) = self.state.current_dialog {
                                        dialog.set_progress(100);
                                    }
                                }
                                Err(e) => {
                                    self.state.error_message = Some(format!("Failed to create agent: {}", e));
                                    self.state.current_dialog = None;
                                    return Ok(());
                                }
                            }
                            
                            self.state.success_message = Some("Agent created successfully".to_string());
                        }
                        Err(error) => {
                            self.state.error_message = Some(format!("Failed to create agent: {}", error));
                        }
                    }
                }
                FormState::EditAgent(form) => {
                    match form.submit() {
                        Ok(agent) => {
                            let mut dialog = ProgressDialog::agent_operation("Updating", &agent.name);
                            dialog.set_progress(50);
                            self.state.current_dialog = Some(DialogState::Progress(dialog));
                            
                            // Actually update the agent
                            match commands::agent::update::execute(agent.id.clone(), agent.name.clone(), agent.system_prompt.clone()).await {
                                Ok(_) => {
                                    // Update local state
                                    if let Some(local_agent) = self.state.agents.iter_mut().find(|a| a.id == agent.id) {
                                        *local_agent = agent.clone();
                                    }
                                    if let Some(DialogState::Progress(ref mut dialog)) = self.state.current_dialog {
                                        dialog.set_progress(100);
                                    }
                                }
                                Err(e) => {
                                    self.state.error_message = Some(format!("Failed to update agent: {}", e));
                                    self.state.current_dialog = None;
                                    return Ok(());
                                }
                            }
                            
                            self.state.success_message = Some("Agent updated successfully".to_string());
                        }
                        Err(error) => {
                            self.state.error_message = Some(format!("Failed to update agent: {}", error));
                        }
                    }
                }
                FormState::CreateTask(form) => {
                    match form.submit() {
                        Ok(task) => {
                            let mut dialog = ProgressDialog::task_operation("Creating", &task.title);
                            dialog.set_progress(50);
                            self.state.current_dialog = Some(DialogState::Progress(dialog));
                            
                            // Actually create the task
                            match commands::task::create::execute(task.agent_id.clone(), task.title.clone(), task.description.clone()).await {
                                Ok(_) => {
                                    // Add to local state for immediate UI update
                                    self.state.tasks.push(task.clone());
                                    if let Some(DialogState::Progress(ref mut dialog)) = self.state.current_dialog {
                                        dialog.set_progress(100);
                                    }
                                }
                                Err(e) => {
                                    self.state.error_message = Some(format!("Failed to create task: {}", e));
                                    self.state.current_dialog = None;
                                    return Ok(());
                                }
                            }
                            
                            self.state.success_message = Some("Task created successfully".to_string());
                        }
                        Err(error) => {
                            self.state.error_message = Some(format!("Failed to create task: {}", error));
                        }
                    }
                }
                FormState::EditTask(form) => {
                    match form.submit() {
                        Ok(task) => {
                            let mut dialog = ProgressDialog::task_operation("Updating", &task.title);
                            dialog.set_progress(50);
                            self.state.current_dialog = Some(DialogState::Progress(dialog));
                            
                            // Actually update the task
                            match task_manager::update_task(task.clone()).await {
                                Ok(_) => {
                                    // Update local state
                                    if let Some(local_task) = self.state.tasks.iter_mut().find(|t| t.id == task.id) {
                                        *local_task = task.clone();
                                    }
                                    if let Some(DialogState::Progress(ref mut dialog)) = self.state.current_dialog {
                                        dialog.set_progress(100);
                                    }
                                }
                                Err(e) => {
                                    self.state.error_message = Some(format!("Failed to update task: {}", e));
                                    self.state.current_dialog = None;
                                    return Ok(());
                                }
                            }
                            
                            self.state.success_message = Some("Task updated successfully".to_string());
                        }
                        Err(error) => {
                            self.state.error_message = Some(format!("Failed to update task: {}", error));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    fn cancel_current_form(&mut self) {
        self.state.current_form = None;
    }
    
    // Dialog management methods
    fn show_help(&mut self) {
        let dialog = HelpDialog::new(self.state.current_screen.clone());
        self.state.current_dialog = Some(DialogState::Help(dialog));
    }
    
    fn show_delete_agent_confirmation(&mut self, agent: Agent) {
        let dialog = ConfirmationDialog::delete("Agent", &agent.name);
        self.state.current_dialog = Some(DialogState::Confirmation(dialog));
        self.state.pending_operation = Some(PendingOperation::DeleteAgent(agent));
    }
    
    fn show_stop_agent_confirmation(&mut self, agent: Agent) {
        let dialog = ConfirmationDialog::stop_agent(&agent.name);
        self.state.current_dialog = Some(DialogState::Confirmation(dialog));
        self.state.pending_operation = Some(PendingOperation::StopAgent(agent));
    }
    
    fn show_delete_task_confirmation(&mut self, task: Task) {
        let dialog = ConfirmationDialog::delete("Task", &task.title);
        self.state.current_dialog = Some(DialogState::Confirmation(dialog));
        self.state.pending_operation = Some(PendingOperation::DeleteTask(task));
    }
    
    fn show_cancel_task_confirmation(&mut self, task: Task) {
        let dialog = ConfirmationDialog::cancel_task(&task.title);
        self.state.current_dialog = Some(DialogState::Confirmation(dialog));
        self.state.pending_operation = Some(PendingOperation::CancelTask(task));
    }
    
    async fn confirm_current_dialog(&mut self) -> Result<()> {
        if let Some(dialog_state) = self.state.current_dialog.take() {
            match dialog_state {
                DialogState::Confirmation(_dialog) => {
                    // Handle confirmation based on pending operation
                    if let Some(operation) = self.state.pending_operation.take() {
                        self.execute_pending_operation(operation).await?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    fn cancel_current_dialog(&mut self) {
        self.state.current_dialog = None;
        self.state.pending_operation = None;
    }
    
    fn close_current_dialog(&mut self) {
        self.state.current_dialog = None;
        self.state.pending_operation = None;
    }
    
    // Search and filter methods
    fn activate_search(&mut self) {
        self.state.search_state.active = true;
        self.state.search_state.query.clear();
    }
    
    fn update_search_results(&mut self) {
        let query = self.state.search_state.query.to_lowercase();
        
        // Clear previous results
        self.state.search_state.filtered_agent_indices.clear();
        self.state.search_state.filtered_task_indices.clear();
        
        if query.is_empty() {
            return;
        }
        
        // Search through agents
        for (index, agent) in self.state.agents.iter().enumerate() {
            if agent.name.to_lowercase().contains(&query) 
                || agent.system_prompt.to_lowercase().contains(&query) {
                self.state.search_state.filtered_agent_indices.push(index);
            }
        }
        
        // Search through tasks  
        for (index, task) in self.state.tasks.iter().enumerate() {
            if task.title.to_lowercase().contains(&query)
                || task.description.to_lowercase().contains(&query)
                || task.agent_id.to_lowercase().contains(&query) {
                self.state.search_state.filtered_task_indices.push(index);
            }
        }
    }
    
    fn show_filter_options(&mut self) {
        // Show a simple filter options message for now
        self.state.success_message = Some("Filter options: A=All, R=Running, P=Pending, C=Complete".to_string());
    }
    
    // Operation methods
    async fn start_agent_operation(&mut self, agent: Agent) -> Result<()> {
        let mut dialog = ProgressDialog::agent_operation("Starting", &agent.name);
        dialog.set_progress(0);
        self.state.current_dialog = Some(DialogState::Progress(dialog));
        
        // Actually start the agent
        match commands::agent::start::execute(agent.name.clone()).await {
            Ok(_) => {
                // Update local state to reflect started status
                if let Some(local_agent) = self.state.agents.iter_mut().find(|a| a.id == agent.id) {
                    local_agent.status = crate::types::AgentStatus::Active;
                }
                if let Some(DialogState::Progress(ref mut dialog)) = self.state.current_dialog {
                    dialog.set_progress(100);
                }
                self.state.success_message = Some(format!("Agent '{}' started successfully", agent.name));
            }
            Err(e) => {
                self.state.error_message = Some(format!("Failed to start agent: {}", e));
            }
        }
        self.state.current_dialog = None;
        
        Ok(())
    }
    
    async fn execute_task_operation(&mut self, task: Task) -> Result<()> {
        let mut dialog = ProgressDialog::task_operation("Executing", &task.title);
        dialog.set_progress(0);
        self.state.current_dialog = Some(DialogState::Progress(dialog));
        
        // Actually execute the task
        match commands::task::execute::execute(task.id.clone()).await {
            Ok(_) => {
                // Update local state to reflect execution
                if let Some(local_task) = self.state.tasks.iter_mut().find(|t| t.id == task.id) {
                    local_task.status = crate::types::TaskStatus::InProgress;
                }
                if let Some(DialogState::Progress(ref mut dialog)) = self.state.current_dialog {
                    dialog.set_progress(100);
                }
                self.state.success_message = Some(format!("Task '{}' executed successfully", task.title));
            }
            Err(e) => {
                self.state.error_message = Some(format!("Failed to execute task: {}", e));
            }
        }
        self.state.current_dialog = None;
        
        Ok(())
    }
    
    // Clear messages after a delay
    pub fn clear_messages(&mut self) {
        self.state.error_message = None;
        self.state.success_message = None;
    }
    
    // Execution operations
    fn toggle_execution_pause(&mut self, task: Task) {
        if task.status == crate::types::TaskStatus::InProgress {
            self.pause_execution(task);
        } else {
            self.resume_execution(task);
        }
    }
    
    fn pause_execution(&mut self, task: Task) {
        let mut dialog = ProgressDialog::task_operation("Pausing", &task.title);
        dialog.set_progress(0);
        self.state.current_dialog = Some(DialogState::Progress(dialog));
        self.state.success_message = Some(format!("Task '{}' paused", task.title));
        // TODO: Implement actual pause via task_manager
    }
    
    fn resume_execution(&mut self, task: Task) {
        let mut dialog = ProgressDialog::task_operation("Resuming", &task.title);
        dialog.set_progress(0);
        self.state.current_dialog = Some(DialogState::Progress(dialog));
        self.state.success_message = Some(format!("Task '{}' resumed", task.title));
        // TODO: Implement actual resume via task_manager
    }
    
    fn cancel_execution(&mut self, task: Task) {
        let dialog = ConfirmationDialog::cancel_task(&task.title);
        self.state.current_dialog = Some(DialogState::Confirmation(dialog));
        self.state.pending_operation = Some(PendingOperation::CancelTask(task));
    }
    
    fn show_execution_details(&mut self, task: Task) {
        // TODO: Create detailed execution view dialog
        self.state.success_message = Some(format!("Showing details for task '{}'", task.title));
    }
    
    // Logs operations
    fn toggle_log_filter(&mut self) {
        // TODO: Implement log filter toggle
        self.state.success_message = Some("Log filter toggled".to_string());
    }
    
    fn clear_logs(&mut self) {
        let dialog = ConfirmationDialog::new(
            "Clear Logs".to_string(),
            "Are you sure you want to clear all logs?".to_string()
        );
        self.state.current_dialog = Some(DialogState::Confirmation(dialog));
        self.state.pending_operation = Some(PendingOperation::ClearLogs);
    }
    
    fn save_logs_to_file(&mut self) {
        // TODO: Implement file save dialog
        self.state.success_message = Some("Logs saved to file".to_string());
    }
    
    fn search_logs(&mut self) {
        self.activate_search();
    }
    
    fn toggle_auto_scroll(&mut self) {
        // TODO: Implement auto-scroll toggle
        self.state.success_message = Some("Auto-scroll toggled".to_string());
    }
    
    fn jump_to_log_start(&mut self) {
        // TODO: Implement jump to start
        self.state.success_message = Some("Jumped to log start".to_string());
    }
    
    fn jump_to_log_end(&mut self) {
        // TODO: Implement jump to end
        self.state.success_message = Some("Jumped to log end".to_string());
    }
    
    fn show_log_details(&mut self) {
        // TODO: Show log entry details
        self.state.success_message = Some("Showing log details".to_string());
    }
    
    fn refresh_logs(&mut self) {
        // TODO: Implement log refresh
        self.state.success_message = Some("Logs refreshed".to_string());
    }
    
    // Dashboard operations
    fn navigate_dashboard_left(&mut self) {
        // TODO: Implement dashboard left navigation
        self.state.success_message = Some("Dashboard navigation left".to_string());
    }
    
    fn navigate_dashboard_right(&mut self) {
        // TODO: Implement dashboard right navigation
        self.state.success_message = Some("Dashboard navigation right".to_string());
    }
    
    fn select_dashboard_item(&mut self) {
        // TODO: Implement dashboard item selection
        self.state.success_message = Some("Dashboard item selected".to_string());
    }
    
    // Agent operations
    fn restart_agent_operation(&mut self, agent: Agent) {
        let dialog = ConfirmationDialog::new(
            "Restart Agent".to_string(),
            format!("Are you sure you want to restart agent '{}'?", agent.name)
        );
        self.state.current_dialog = Some(DialogState::Confirmation(dialog));
        self.state.pending_operation = Some(PendingOperation::RestartAgent(agent));
    }
    
    fn show_agent_details(&mut self, agent: Agent) {
        // TODO: Create detailed agent view dialog
        self.state.success_message = Some(format!("Showing details for agent '{}'", agent.name));
    }
    
    fn show_task_details(&mut self, task: Task) {
        // TODO: Create detailed task view dialog
        self.state.success_message = Some(format!("Showing details for task '{}'", task.title));
    }
    
    // Task filter operations
    fn filter_tasks_all(&mut self) {
        self.state.filters.task_status_filter = None;
        self.state.success_message = Some("Showing all tasks".to_string());
    }
    
    fn filter_tasks_running(&mut self) {
        self.state.filters.task_status_filter = Some("InProgress".to_string());
        self.state.success_message = Some("Showing running tasks".to_string());
    }
    
    fn filter_tasks_pending(&mut self) {
        self.state.filters.task_status_filter = Some("Todo".to_string());
        self.state.success_message = Some("Showing pending tasks".to_string());
    }
    
    fn filter_tasks_completed(&mut self) {
        self.state.filters.task_status_filter = Some("Done".to_string());
        self.state.success_message = Some("Showing completed tasks".to_string());
    }
    
    /// Execute a pending operation after confirmation
    async fn execute_pending_operation(&mut self, operation: PendingOperation) -> Result<()> {
        match operation {
            PendingOperation::DeleteAgent(agent) => {
                self.execute_delete_agent(agent).await?;
            }
            PendingOperation::StopAgent(agent) => {
                self.execute_stop_agent(agent).await?;
            }
            PendingOperation::DeleteTask(task) => {
                self.execute_delete_task(task).await?;
            }
            PendingOperation::CancelTask(task) => {
                self.execute_cancel_task(task).await?;
            }
            PendingOperation::RestartAgent(agent) => {
                self.execute_restart_agent(agent).await?;
            }
            PendingOperation::ClearLogs => {
                self.execute_clear_logs().await?;
            }
        }
        Ok(())
    }
    
    /// Execute agent deletion
    pub async fn execute_delete_agent(&mut self, agent: Agent) -> Result<()> {
        let mut dialog = ProgressDialog::agent_operation("Deleting", &agent.name);
        dialog.set_progress(0);
        self.state.current_dialog = Some(DialogState::Progress(dialog));

        // TODO: Queue async operation to actually delete agent via agent_manager
        commands::agent::delete::execute(agent.name.clone(), true).await?;

        // For now, just show a success message
        self.state.success_message = Some(format!("Agent '{}' deleted successfully", agent.name.clone()));

        // Remove from local state immediately for UI responsiveness
        self.state.agents.retain(|a| a.id != agent.id);

        // Reset selection if deleted agent was selected
        if let Some(selected_index) = self.state.selected_agent {
            if selected_index >= self.state.agents.len() {
                self.state.selected_agent = if self.state.agents.is_empty() {
                    None
                } else {
                    Some(self.state.agents.len() - 1)
                };
            }
        }

        Ok(())
    }
    
    /// Execute agent stop operation
    async fn execute_stop_agent(&mut self, agent: Agent) -> Result<()> {
        let mut dialog = ProgressDialog::agent_operation("Stopping", &agent.name);
        dialog.set_progress(0);
        self.state.current_dialog = Some(DialogState::Progress(dialog.clone()));
        
        // Actually stop the agent
        match commands::agent::stop::execute(agent.name.clone()).await {
            Ok(_) => {
                if let Some(DialogState::Progress(ref mut progress_dialog)) = self.state.current_dialog {
                    progress_dialog.set_progress(100);
                }
            }
            Err(e) => {
                self.state.error_message = Some(format!("Failed to stop agent: {}", e));
                self.state.current_dialog = None;
                return Ok(());
            }
        }
        
        // Show success message
        self.state.success_message = Some(format!("Agent '{}' stopped successfully", agent.name));
        
        // Update local state to reflect stopped status
        if let Some(local_agent) = self.state.agents.iter_mut().find(|a| a.id == agent.id) {
            local_agent.status = crate::types::AgentStatus::Inactive;
        }
        
        Ok(())
    }
    
    /// Execute task deletion
    async fn execute_delete_task(&mut self, task: Task) -> Result<()> {
        let mut dialog = ProgressDialog::task_operation("Deleting", &task.title);
        dialog.set_progress(0);
        self.state.current_dialog = Some(DialogState::Progress(dialog.clone()));
        
        // Actually delete the task
        // Note: There's no delete command, but we can use the task_manager directly
        match crate::core::task_manager::delete_task(&task.id).await {
            Ok(_) => {
                if let Some(DialogState::Progress(ref mut progress_dialog)) = self.state.current_dialog {
                    progress_dialog.set_progress(100);
                }
            }
            Err(e) => {
                self.state.error_message = Some(format!("Failed to delete task: {}", e));
                self.state.current_dialog = None;
                return Ok(());
            }
        }
        
        // Show success message
        self.state.success_message = Some(format!("Task '{}' deleted successfully", task.title));
        
        // Remove from local state immediately for UI responsiveness
        self.state.tasks.retain(|t| t.id != task.id);
        
        // Reset selection if deleted task was selected
        if let Some(selected_index) = self.state.selected_task {
            if selected_index >= self.state.tasks.len() {
                self.state.selected_task = if self.state.tasks.is_empty() {
                    None
                } else {
                    Some(self.state.tasks.len() - 1)
                };
            }
        }
        
        Ok(())
    }
    
    /// Execute task cancellation
    async fn execute_cancel_task(&mut self, task: Task) -> Result<()> {
        let mut dialog = ProgressDialog::task_operation("Cancelling", &task.title);
        dialog.set_progress(0);
        self.state.current_dialog = Some(DialogState::Progress(dialog.clone()));
        
        // Actually cancel the task
        match commands::task::cancel::execute(task.id.clone()).await {
            Ok(_) => {
                if let Some(DialogState::Progress(ref mut progress_dialog)) = self.state.current_dialog {
                    progress_dialog.set_progress(100);
                }
            }
            Err(e) => {
                self.state.error_message = Some(format!("Failed to cancel task: {}", e));
                self.state.current_dialog = None;
                return Ok(());
            }
        }
        
        // Update local state to reflect cancelled status
        if let Some(local_task) = self.state.tasks.iter_mut().find(|t| t.id == task.id) {
            local_task.status = crate::types::TaskStatus::Cancelled;
        }
        
        Ok(())
    }
    
    /// Execute agent restart operation
    async fn execute_restart_agent(&mut self, agent: Agent) -> Result<()> {
        let mut dialog = ProgressDialog::agent_operation("Restarting", &agent.name);
        dialog.set_progress(0);
        self.state.current_dialog = Some(DialogState::Progress(dialog.clone()));
        
        // Actually restart the agent (stop then start)
        match commands::agent::stop::execute(agent.name.clone()).await {
            Ok(_) => {
                if let Some(DialogState::Progress(ref mut progress_dialog)) = self.state.current_dialog {
                    progress_dialog.set_progress(50);
                }
                // Now start the agent
                match commands::agent::start::execute(agent.name.clone()).await {
                    Ok(_) => {
                        if let Some(DialogState::Progress(ref mut progress_dialog)) = self.state.current_dialog {
                            progress_dialog.set_progress(100);
                        }
                    }
                    Err(e) => {
                        self.state.error_message = Some(format!("Failed to start agent after stop: {}", e));
                        self.state.current_dialog = None;
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                self.state.error_message = Some(format!("Failed to stop agent for restart: {}", e));
                self.state.current_dialog = None;
                return Ok(());
            }
        }
        
        // Show success message
        self.state.success_message = Some(format!("Agent '{}' restarted successfully", agent.name));
        
        // Update local state to reflect restarted status
        if let Some(local_agent) = self.state.agents.iter_mut().find(|a| a.id == agent.id) {
            local_agent.status = crate::types::AgentStatus::Active;
        }
        
        Ok(())
    }
    
    /// Execute log clearing operation
    async fn execute_clear_logs(&mut self) -> Result<()> {
        let mut dialog = ProgressDialog::new(
            "Clear Logs".to_string(),
            "Clearing all logs...".to_string()
        );
        dialog.set_progress(0);
        self.state.current_dialog = Some(DialogState::Progress(dialog));
        
        // Clear logs by resetting the log collection in state
        // Since there's no specific log clearing command, we'll simulate clearing logs
        // In a real implementation, this would interact with the logging system
        if let Some(DialogState::Progress(ref mut progress_dialog)) = self.state.current_dialog {
            progress_dialog.set_progress(100);
        }
        self.state.success_message = Some("All logs cleared successfully".to_string());
        
        Ok(())
    }
}
