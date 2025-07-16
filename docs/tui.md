# Nox TUI Implementation Plan

## Overview

This document outlines the implementation plan for a Terminal User Interface (TUI) for the Nox Autonomous Agent Ecosystem. The TUI will provide a comprehensive visual interface for managing agents, tasks, execution, monitoring, and system operations.

## 1. Technical Foundation

### 1.1 Technology Stack
- **Primary Framework**: `ratatui` (formerly `tui-rs`) - Modern Rust TUI library
- **Backend**: `crossterm` - Cross-platform terminal manipulation
- **Event Handling**: `tokio` - Async event processing
- **State Management**: Custom state management with `tokio::sync`
- **Integration**: Direct integration with existing `nox::core` modules

### 1.2 Dependencies to Add
```toml
[dependencies]
ratatui = "0.24"
crossterm = "0.27"
tokio = { version = "1.28", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
```

### 1.3 Architecture
```
src/
├── tui/
│   ├── mod.rs              # TUI module exports
│   ├── app.rs              # Main application state
│   ├── ui.rs               # UI rendering logic
│   ├── events.rs           # Event handling
│   ├── components/         # Reusable UI components
│   │   ├── mod.rs
│   │   ├── agent_list.rs   # Agent list component
│   │   ├── task_list.rs    # Task list component
│   │   ├── detail_panel.rs # Detail view component
│   │   ├── status_bar.rs   # Status bar component
│   │   └── help_panel.rs   # Help/shortcuts panel
│   ├── screens/            # Full screen views
│   │   ├── mod.rs
│   │   ├── dashboard.rs    # Main dashboard
│   │   ├── agents.rs       # Agent management screen
│   │   ├── tasks.rs        # Task management screen
│   │   ├── execution.rs    # Task execution monitoring
│   │   └── logs.rs         # System logs viewer
│   └── utils/              # Utility functions
│       ├── mod.rs
│       ├── colors.rs       # Color scheme
│       └── formatting.rs   # Text formatting helpers
```

## 2. User Interface Design

### 2.1 Main Dashboard Layout
```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Nox Agent Ecosystem v0.1.0                                    [Q] Quit [H] Help │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─── System Status ───┐  ┌─── Agent Summary ───┐  ┌─── Task Summary ───┐     │
│  │ Status: ● Running   │  │ Active:    3/5      │  │ Running:     2     │     │
│  │ Uptime: 2h 45m     │  │ Inactive:  2/5      │  │ Pending:     7     │     │
│  │ CPU:    45%        │  │ Error:     0/5      │  │ Completed:   24    │     │
│  │ Memory: 1.2GB      │  │                     │  │ Failed:      1     │     │
│  └────────────────────┘  └─────────────────────┘  └─────────────────────┘     │
│                                                                                 │
│  ┌─── Recent Activity ──────────────────────────────────────────────────────┐  │
│  │ [14:23] Agent 'CodeReviewer' started task 'Review PR #123'             │  │
│  │ [14:20] Task 'Generate report' completed successfully                   │  │
│  │ [14:15] Agent 'NewsBot' status changed to Active                        │  │
│  │ [14:10] New task 'Analyze logs' created for agent 'DataAnalyst'        │  │
│  │ [14:05] Agent 'TestRunner' completed task 'Run integration tests'      │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│ [1] Agents  [2] Tasks  [3] Execution  [4] Logs  [Tab] Next  [←/→] Navigate    │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Agent Management Screen
```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Agent Management                                               [ESC] Back       │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─── Agent List ────────────────────┐  ┌─── Agent Details ──────────────────┐ │
│  │ > NewsBot           ● Active      │  │ Name: NewsBot                      │ │
│  │   CodeReviewer      ○ Inactive    │  │ ID: agent-1752272015775           │ │
│  │   DataAnalyst       ● Active      │  │ Status: ● Active                   │ │
│  │   TestRunner        ○ Inactive    │  │ Created: 2025-07-12 10:30:00      │ │
│  │   DocGenerator      ● Active      │  │ Last Active: 2025-07-15 14:23:15  │ │
│  │                                   │  │                                    │ │
│  │ [N] New  [E] Edit  [D] Delete     │  │ System Prompt:                     │ │
│  │ [S] Start  [T] Stop  [R] Restart  │  │ You are a news analysis agent     │ │
│  │                                   │  │ specializing in AI developments... │ │
│  │                                   │  │                                    │ │
│  │ Total: 5 agents                   │  │ Resource Limits:                   │ │
│  │ Active: 3, Inactive: 2            │  │ Memory: 512MB, CPU: 50%           │ │
│  └───────────────────────────────────┘  └────────────────────────────────────┘ │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│ [↑/↓] Select  [Enter] Action  [Tab] Switch Panel  [F1] Help                    │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Task Management Screen
```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Task Management                                                [ESC] Back       │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─── Task List ─────────────────────┐  ┌─── Task Details ───────────────────┐ │
│  │ > Analyze quantum papers  📋 Todo │  │ Title: Analyze quantum papers      │ │
│  │   Generate daily report   🔄 Run  │  │ Agent: NewsBot                     │ │
│  │   Review code changes     ✅ Done │  │ Status: 📋 Todo                    │ │
│  │   Update documentation    ❌ Fail │  │ Priority: High                     │ │
│  │   Run integration tests   📋 Todo │  │ Created: 2025-07-15 10:30:00      │ │
│  │                                   │  │                                    │ │
│  │ Filter: [A] All [R] Running       │  │ Description:                       │ │
│  │         [P] Pending [C] Complete  │  │ Review and summarize recent        │ │
│  │                                   │  │ quantum computing papers from      │ │
│  │ [N] New  [E] Execute  [C] Cancel  │  │ ArXiv and identify key trends...   │ │
│  │ [U] Update  [D] Delete            │  │                                    │ │
│  │                                   │  │ Progress: ████████░░ 80%           │ │
│  │ Total: 34 tasks                   │  │                                    │ │
│  │ 2 Running, 7 Pending, 24 Done    │  │ Last Update: 2025-07-15 14:20:00  │ │
│  └───────────────────────────────────┘  └────────────────────────────────────┘ │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│ [↑/↓] Select  [Enter] Action  [Tab] Switch Panel  [F] Filter                   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 2.4 Task Execution Monitoring
```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Task Execution Monitor                                         [ESC] Back       │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─── Running Tasks ─────────────────┐  ┌─── Execution Output ───────────────┐ │
│  │ > Generate report    NewsBot  45s │  │ Task: Generate report              │ │
│  │   Analyze logs       DataBot  2m  │  │ Agent: NewsBot                     │ │
│  │                                   │  │ Started: 14:20:15                  │ │
│  │ Queue (3):                        │  │ ──────────────────────────────────── │ │
│  │   Review PR #123     CodeBot      │  │ [14:20:15] Task started            │ │
│  │   Update docs        DocBot       │  │ [14:20:16] Fetching data sources   │ │
│  │   Run tests          TestBot      │  │ [14:20:18] Processing 247 articles │ │
│  │                                   │  │ [14:20:22] Analyzing sentiment...  │ │
│  │ [C] Cancel  [P] Pause  [R] Resume │  │ [14:20:35] Generating summary...   │ │
│  │                                   │  │ [14:20:45] ████████████░░ 85%      │ │
│  │ Active: 2, Queued: 3, Slots: 5   │  │                                    │ │
│  └───────────────────────────────────┘  └────────────────────────────────────┘ │
│                                                                                 │
│  ┌─── System Resources ──────────────────────────────────────────────────────┐ │
│  │ CPU: ████████████████████████████████████████████████████████████░░ 85%  │ │
│  │ Memory: ████████████████████████████████████████████████████░░░░░░ 68%    │ │
│  │ Active Processes: 2/5    Claude CLI Sessions: 2                          │ │
│  └───────────────────────────────────────────────────────────────────────────┘ │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│ [↑/↓] Select  [Enter] View Details  [Space] Pause/Resume  [Del] Cancel        │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 2.5 System Logs Viewer
```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ System Logs                                                    [ESC] Back       │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─── Log Filters ───┐  ┌─── Log Entries ────────────────────────────────────┐ │
│  │ Level:            │  │ [14:23:15] INFO  Agent 'NewsBot' started task      │ │
│  │ ■ ERROR           │  │ [14:23:10] INFO  Task 'Generate report' queued     │ │
│  │ ■ WARN            │  │ [14:23:05] WARN  High memory usage detected (85%)  │ │
│  │ ■ INFO            │  │ [14:22:58] INFO  Agent 'CodeReviewer' went active  │ │
│  │ □ DEBUG           │  │ [14:22:45] ERROR Task 'Update docs' failed         │ │
│  │                   │  │ [14:22:30] INFO  System started successfully       │ │
│  │ Source:           │  │ [14:22:15] INFO  Registry initialized              │ │
│  │ ■ System          │  │ [14:22:10] INFO  Git repository ready              │ │
│  │ ■ Agents          │  │ [14:22:05] INFO  Claude CLI integration verified   │ │
│  │ ■ Tasks           │  │ [14:22:00] INFO  Configuration loaded              │ │
│  │ ■ API             │  │ [14:21:55] INFO  Starting Nox v0.1.0               │ │
│  │ □ WebSocket       │  │                                                    │ │
│  │                   │  │ [F] Filter  [C] Clear  [S] Save  [/] Search        │ │
│  │ [R] Refresh       │  │ Showing: 1,247 entries                            │ │
│  │ [A] Auto-scroll   │  │ Filtered: 247 entries                             │ │
│  └───────────────────┘  └────────────────────────────────────────────────────┘ │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│ [↑/↓] Scroll  [Home/End] Jump  [Space] Toggle Filter  [Enter] View Details     │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 3. Implementation Phases

### Phase 1: Foundation (Week 1-2)
**Goal**: Basic TUI framework and navigation

#### 3.1 Core Infrastructure
- [ ] Set up `ratatui` and `crossterm` dependencies
- [ ] Create basic app structure with state management
- [ ] Implement event handling system
- [ ] Create basic layout with header, content, and status bar
- [ ] Add keyboard navigation between screens

#### 3.2 Basic Components
- [ ] Status bar component with help text
- [ ] Header component with title and system info
- [ ] List component for agents/tasks
- [ ] Detail panel component for item details
- [ ] Basic color scheme and styling

#### 3.3 Integration Setup
- [ ] Integrate with existing `nox::core` modules
- [ ] Create TUI-specific data structures
- [ ] Implement real-time data updates
- [ ] Add error handling and user feedback

### Phase 2: Core Functionality (Week 3-4)
**Goal**: Agent and task management screens

#### 3.4 Dashboard Screen
- [ ] System status overview
- [ ] Agent summary statistics
- [ ] Task summary statistics
- [ ] Recent activity feed
- [ ] Navigation shortcuts

#### 3.5 Agent Management
- [ ] Agent list with status indicators
- [ ] Agent detail view with full information
- [ ] Agent creation form
- [ ] Agent editing capabilities
- [ ] Agent start/stop/restart actions
- [ ] Agent deletion with confirmation

#### 3.6 Task Management
- [ ] Task list with filtering capabilities
- [ ] Task detail view with progress
- [ ] Task creation form
- [ ] Task status updates
- [ ] Task execution triggers
- [ ] Task cancellation

### Phase 3: Advanced Features (Week 5-6)
**Goal**: Monitoring and execution views

#### 3.7 Execution Monitoring
- [ ] Real-time task execution view
- [ ] Queue management
- [ ] Progress tracking with visual indicators
- [ ] Resource usage monitoring
- [ ] Task output streaming
- [ ] Execution history

#### 3.8 System Logs
- [ ] Log viewer with filtering
- [ ] Real-time log updates
- [ ] Log level filtering
- [ ] Source filtering (agents, tasks, system)
- [ ] Log search functionality
- [ ] Log export capabilities

### Phase 4: Polish and Enhancement (Week 7-8)
**Goal**: User experience improvements

#### 3.9 User Experience
- [ ] Keyboard shortcuts and help system
- [ ] Responsive layout for different terminal sizes
- [ ] Improved error handling and user feedback
- [ ] Configuration persistence
- [ ] Performance optimization
- [ ] Accessibility improvements

#### 3.10 Advanced Features
- [ ] Bulk operations (multi-select)
- [ ] Custom views and layouts
- [ ] Themes and color customization
- [ ] Plugin system for custom components
- [ ] Integration with external tools
- [ ] Export/import functionality

## 4. Key Features

### 4.1 Real-time Updates
- Live status updates for agents and tasks
- Real-time log streaming
- Automatic refresh of data
- Event-driven UI updates

### 4.2 Keyboard Navigation
- Vim-style navigation (hjkl)
- Tab/Shift+Tab for panel switching
- Arrow keys for list navigation
- Enter for selection/action
- Escape for back/cancel

### 4.3 Visual Indicators
- **Agent Status**: ● Active, ○ Inactive, ⚠ Error, ⏸ Starting, ⏹ Stopping
- **Task Status**: 📋 Todo, 🔄 Running, ✅ Done, ❌ Failed, ⏸ Cancelled
- **Priority**: 🔴 High, 🟡 Medium, 🟢 Low
- **Progress**: Visual progress bars
- **Resource Usage**: CPU/Memory graphs

### 4.4 Filtering and Search
- Filter agents by status
- Filter tasks by status, priority, agent
- Search by name, description, ID
- Date range filtering
- Custom filter combinations

### 4.5 Actions and Operations
- **Agent Actions**: Start, Stop, Restart, Edit, Delete
- **Task Actions**: Execute, Cancel, Update, Delete
- **System Actions**: Refresh, Export, Settings
- **Bulk Operations**: Multi-select with batch actions

## 5. Technical Implementation Details

### 5.1 State Management
```rust
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_screen: Screen,
    pub agents: Vec<Agent>,
    pub tasks: Vec<Task>,
    pub selected_agent: Option<usize>,
    pub selected_task: Option<usize>,
    pub logs: Vec<LogEntry>,
    pub system_status: SystemStatus,
    pub filters: FilterState,
    pub should_quit: bool,
}
```

### 5.2 Event System
```rust
#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    AgentUpdated(Agent),
    TaskUpdated(Task),
    LogReceived(LogEntry),
    SystemStatusUpdated(SystemStatus),
}
```

### 5.3 Component Architecture
```rust
pub trait Component {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState);
    fn handle_event(&mut self, event: &AppEvent, state: &mut AppState) -> bool;
}
```

### 5.4 Integration Points
- Direct calls to `nox::core::agent_manager`
- Direct calls to `nox::core::task_manager`
- Real-time updates via `tokio::sync::watch`
- Async task execution monitoring
- Log streaming integration

## 6. Testing Strategy

### 6.1 Unit Tests
- Component rendering tests
- Event handling tests
- State management tests
- Integration with core modules

### 6.2 Integration Tests
- End-to-end workflow tests
- Real-time update tests
- Error handling tests
- Performance tests

### 6.3 Manual Testing
- User experience testing
- Keyboard navigation testing
- Visual rendering testing
- Cross-platform compatibility

## 7. Deployment and Distribution

### 7.1 Build Configuration
```toml
[[bin]]
name = "nox-tui"
path = "src/tui/main.rs"
```

### 7.2 Installation
```bash
cargo install --path . --bin nox-tui
```

### 7.3 Usage
```bash
# Launch TUI
nox-tui

# With configuration
nox-tui --config /path/to/config.toml

# Development mode
nox-tui --dev
```

## 8. Future Enhancements

### 8.1 Advanced Features
- Multi-workspace support
- Remote agent management
- Custom dashboard layouts
- Plugin system for extensions
- Integration with external monitoring

### 8.2 Performance Optimizations
- Lazy loading of large datasets
- Virtual scrolling for large lists
- Efficient rendering optimizations
- Memory usage optimization

### 8.3 Accessibility
- Screen reader support
- High contrast themes
- Keyboard-only navigation
- Customizable key bindings

## 9. Success Metrics

### 9.1 User Experience
- Time to complete common tasks
- Navigation efficiency
- Error recovery
- User satisfaction

### 9.2 Technical Performance
- Response time for operations
- Memory usage
- CPU utilization
- Startup time

### 9.3 Feature Completeness
- All core operations available
- Real-time updates working
- Error handling comprehensive
- Cross-platform compatibility

## 10. Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| Phase 1 | 2 weeks | Basic TUI framework, navigation |
| Phase 2 | 2 weeks | Agent/task management screens |
| Phase 3 | 2 weeks | Monitoring, execution views |
| Phase 4 | 2 weeks | Polish, advanced features |

**Total Estimated Time**: 8 weeks for full implementation

## 11. Getting Started

To begin implementation:
1. Add TUI dependencies to `Cargo.toml`
2. Create `src/tui/` directory structure
3. Implement basic app framework
4. Start with dashboard screen
5. Add agent management functionality
6. Expand to task management
7. Add monitoring capabilities
8. Polish and optimize

This plan provides a comprehensive roadmap for implementing a feature-rich TUI that will significantly improve the user experience of the Nox Agent Ecosystem.