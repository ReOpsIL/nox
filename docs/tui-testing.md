# TUI Testing Framework Plan

## Overview

This document outlines a comprehensive testing strategy for the Nox TUI application, providing automated testing capabilities similar to Playwright for web applications.

## 1. Testing Architecture

### 1.1 Multi-Layer Testing Approach
```
┌─────────────────────────────────────────┐
│           E2E Tests                     │  ← Full workflow testing
├─────────────────────────────────────────┤
│           Integration Tests             │  ← Component interaction
├─────────────────────────────────────────┤
│           Unit Tests                    │  ← Individual components
├─────────────────────────────────────────┤
│           Visual Tests                  │  ← Screen rendering
└─────────────────────────────────────────┘
```

### 1.2 Technology Stack
```toml
[dev-dependencies]
# Core testing
ratatui = { version = "0.24", features = ["testing"] }
expectrl = "0.7"              # Terminal automation
tokio-test = "0.4"            # Async testing
tempfile = "3.5"              # Temporary files
serial_test = "3.0"           # Serial test execution

# Visual testing
image = "0.24"                # Image comparison
termshot = "0.1"              # Terminal screenshots
```

## 2. Testing Framework Components

### 2.1 Core Test Harness
```rust
// tests/tui/harness.rs
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub struct TuiTestHarness {
    app: App,
    terminal: Terminal<TestBackend>,
    events: Vec<AppEvent>,
    width: u16,
    height: u16,
}

impl TuiTestHarness {
    pub fn new(width: u16, height: u16) -> Self {
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend).unwrap();
        let app = App::new();
        
        Self {
            app,
            terminal,
            events: Vec::new(),
            width,
            height,
        }
    }
    
    // Input simulation
    pub fn send_key(&mut self, key: KeyCode) -> &mut Self {
        self.send_key_with_modifiers(key, KeyModifiers::empty())
    }
    
    pub fn send_key_with_modifiers(&mut self, key: KeyCode, modifiers: KeyModifiers) -> &mut Self {
        let event = KeyEvent::new(key, modifiers);
        self.app.handle_key_event(event);
        self.render();
        self
    }
    
    pub fn send_text(&mut self, text: &str) -> &mut Self {
        for ch in text.chars() {
            self.send_key(KeyCode::Char(ch));
        }
        self
    }
    
    pub fn press_enter(&mut self) -> &mut Self {
        self.send_key(KeyCode::Enter)
    }
    
    pub fn press_escape(&mut self) -> &mut Self {
        self.send_key(KeyCode::Esc)
    }
    
    pub fn press_tab(&mut self) -> &mut Self {
        self.send_key(KeyCode::Tab)
    }
    
    // Navigation helpers
    pub fn navigate_up(&mut self) -> &mut Self {
        self.send_key(KeyCode::Up)
    }
    
    pub fn navigate_down(&mut self) -> &mut Self {
        self.send_key(KeyCode::Down)
    }
    
    pub fn navigate_left(&mut self) -> &mut Self {
        self.send_key(KeyCode::Left)
    }
    
    pub fn navigate_right(&mut self) -> &mut Self {
        self.send_key(KeyCode::Right)
    }
    
    // Rendering
    pub fn render(&mut self) -> &mut Self {
        self.app.render(&mut self.terminal).unwrap();
        self
    }
    
    pub fn tick(&mut self) -> &mut Self {
        self.app.tick();
        self.render();
        self
    }
    
    // Waiting and timing
    pub fn wait_for_condition<F>(&mut self, condition: F, timeout: Duration) -> &mut Self
    where
        F: Fn(&TuiTestHarness) -> bool,
    {
        let start = std::time::Instant::now();
        while !condition(self) && start.elapsed() < timeout {
            self.tick();
            std::thread::sleep(Duration::from_millis(50));
        }
        self
    }
    
    pub fn wait_for_text(&mut self, text: &str, timeout: Duration) -> &mut Self {
        self.wait_for_condition(|h| h.screen_contains(text), timeout)
    }
    
    // Assertions
    pub fn assert_screen_contains(&self, text: &str) -> &Self {
        assert!(self.screen_contains(text), "Screen does not contain: {}", text);
        self
    }
    
    pub fn assert_screen_not_contains(&self, text: &str) -> &Self {
        assert!(!self.screen_contains(text), "Screen contains: {}", text);
        self
    }
    
    pub fn assert_current_screen(&self, expected: Screen) -> &Self {
        assert_eq!(self.app.current_screen(), expected);
        self
    }
    
    pub fn assert_selected_agent(&self, expected: &str) -> &Self {
        if let Some(agent) = self.app.selected_agent() {
            assert_eq!(agent.name, expected);
        } else {
            panic!("No agent selected");
        }
        self
    }
    
    pub fn assert_selected_task(&self, expected: &str) -> &Self {
        if let Some(task) = self.app.selected_task() {
            assert_eq!(task.title, expected);
        } else {
            panic!("No task selected");
        }
        self
    }
    
    pub fn assert_status_bar_contains(&self, text: &str) -> &Self {
        let buffer = self.terminal.backend().buffer();
        let last_line = buffer.content.split('\n').last().unwrap_or("");
        assert!(last_line.contains(text), "Status bar does not contain: {}", text);
        self
    }
    
    // Screen inspection
    pub fn screen_contains(&self, text: &str) -> bool {
        let buffer = self.terminal.backend().buffer();
        buffer.content.contains(text)
    }
    
    pub fn get_screen_text(&self) -> String {
        let buffer = self.terminal.backend().buffer();
        buffer.content.clone()
    }
    
    pub fn get_screen_lines(&self) -> Vec<String> {
        self.get_screen_text().lines().map(|s| s.to_string()).collect()
    }
    
    pub fn current_screen(&self) -> Screen {
        self.app.current_screen()
    }
    
    pub fn selected_agent(&self) -> Option<&Agent> {
        self.app.selected_agent()
    }
    
    pub fn selected_task(&self) -> Option<&Task> {
        self.app.selected_task()
    }
    
    // Visual testing
    pub fn save_screenshot(&self, filename: &str) -> &Self {
        let buffer = self.terminal.backend().buffer();
        std::fs::write(filename, &buffer.content).unwrap();
        self
    }
    
    pub fn compare_screenshot(&self, reference_file: &str) -> bool {
        let current = self.get_screen_text();
        let reference = std::fs::read_to_string(reference_file).unwrap_or_default();
        current == reference
    }
    
    // Debugging
    pub fn debug_print_screen(&self) -> &Self {
        println!("=== SCREEN DEBUG ===");
        println!("{}", self.get_screen_text());
        println!("=== END DEBUG ===");
        self
    }
}
```

### 2.2 Test Macros and Helpers
```rust
// tests/tui/macros.rs

macro_rules! tui_test {
    ($name:ident, $width:expr, $height:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let mut harness = TuiTestHarness::new($width, $height);
            $body(harness);
        }
    };
}

macro_rules! assert_screen {
    ($harness:expr, contains $text:expr) => {
        $harness.assert_screen_contains($text)
    };
    ($harness:expr, not_contains $text:expr) => {
        $harness.assert_screen_not_contains($text)
    };
    ($harness:expr, screen $screen:expr) => {
        $harness.assert_current_screen($screen)
    };
}

// Fluent testing DSL
pub trait TuiTestDsl {
    fn when_i_press(&mut self, key: KeyCode) -> &mut Self;
    fn when_i_type(&mut self, text: &str) -> &mut Self;
    fn when_i_navigate_to(&mut self, screen: Screen) -> &mut Self;
    fn then_i_should_see(&mut self, text: &str) -> &mut Self;
    fn then_i_should_not_see(&mut self, text: &str) -> &mut Self;
    fn then_i_should_be_on(&mut self, screen: Screen) -> &mut Self;
}

impl TuiTestDsl for TuiTestHarness {
    fn when_i_press(&mut self, key: KeyCode) -> &mut Self {
        self.send_key(key)
    }
    
    fn when_i_type(&mut self, text: &str) -> &mut Self {
        self.send_text(text)
    }
    
    fn when_i_navigate_to(&mut self, screen: Screen) -> &mut Self {
        match screen {
            Screen::Dashboard => self.send_key(KeyCode::Char('0')),
            Screen::Agents => self.send_key(KeyCode::Char('1')),
            Screen::Tasks => self.send_key(KeyCode::Char('2')),
            Screen::Execution => self.send_key(KeyCode::Char('3')),
            Screen::Logs => self.send_key(KeyCode::Char('4')),
        }
    }
    
    fn then_i_should_see(&mut self, text: &str) -> &mut Self {
        self.assert_screen_contains(text)
    }
    
    fn then_i_should_not_see(&mut self, text: &str) -> &mut Self {
        self.assert_screen_not_contains(text)
    }
    
    fn then_i_should_be_on(&mut self, screen: Screen) -> &mut Self {
        self.assert_current_screen(screen)
    }
}
```

## 3. Test Categories

### 3.1 Visual Rendering Tests
```rust
// tests/tui/visual_tests.rs
use super::harness::*;

tui_test!(test_dashboard_layout, 80, 24, |mut harness| {
    harness
        .render()
        .assert_screen_contains("Nox Agent Ecosystem")
        .assert_screen_contains("System Status")
        .assert_screen_contains("Agent Summary")
        .assert_screen_contains("Task Summary")
        .assert_screen_contains("Recent Activity");
});

tui_test!(test_agent_list_rendering, 80, 24, |mut harness| {
    harness
        .navigate_to(Screen::Agents)
        .assert_screen_contains("Agent Management")
        .assert_screen_contains("Agent List")
        .assert_screen_contains("Agent Details")
        .assert_screen_contains("[N] New")
        .assert_screen_contains("[E] Edit");
});

tui_test!(test_responsive_layout, 120, 40, |mut harness| {
    harness
        .render()
        .assert_screen_contains("Nox Agent Ecosystem")
        // Test that wider layout shows more information
        .assert_screen_contains("Extended view");
});
```

### 3.2 Navigation Tests
```rust
// tests/tui/navigation_tests.rs
use super::harness::*;

tui_test!(test_screen_navigation, 80, 24, |mut harness| {
    harness
        .when_i_navigate_to(Screen::Agents)
        .then_i_should_be_on(Screen::Agents)
        .then_i_should_see("Agent Management")
        
        .when_i_navigate_to(Screen::Tasks)
        .then_i_should_be_on(Screen::Tasks)
        .then_i_should_see("Task Management")
        
        .when_i_press(KeyCode::Esc)
        .then_i_should_be_on(Screen::Dashboard);
});

tui_test!(test_list_navigation, 80, 24, |mut harness| {
    harness
        .navigate_to(Screen::Agents)
        .navigate_down()
        .navigate_down()
        .assert_selected_agent("DataAnalyst")
        
        .navigate_up()
        .assert_selected_agent("CodeReviewer");
});

tui_test!(test_tab_navigation, 80, 24, |mut harness| {
    harness
        .navigate_to(Screen::Agents)
        .press_tab()
        .assert_screen_contains("Agent Details")
        
        .press_tab()
        .assert_screen_contains("Agent List");
});
```

### 3.3 Workflow Tests
```rust
// tests/tui/workflow_tests.rs
use super::harness::*;

tui_test!(test_create_agent_workflow, 80, 24, |mut harness| {
    harness
        .navigate_to(Screen::Agents)
        .send_key(KeyCode::Char('n'))
        .assert_screen_contains("Create New Agent")
        
        .send_text("TestAgent")
        .press_tab()
        .send_text("You are a test agent")
        .press_enter()
        
        .assert_screen_contains("Agent created successfully")
        .assert_screen_contains("TestAgent");
});

tui_test!(test_execute_task_workflow, 80, 24, |mut harness| {
    harness
        .navigate_to(Screen::Tasks)
        .send_key(KeyCode::Char('n'))
        .send_text("Test Task")
        .press_tab()
        .send_text("This is a test task")
        .press_enter()
        
        .send_key(KeyCode::Char('e'))
        .wait_for_text("Task executing", Duration::from_secs(2))
        .assert_screen_contains("Task executing");
});

tui_test!(test_monitor_execution_workflow, 80, 24, |mut harness| {
    harness
        .navigate_to(Screen::Execution)
        .assert_screen_contains("Task Execution Monitor")
        .assert_screen_contains("Running Tasks")
        .assert_screen_contains("System Resources");
});
```

### 3.4 Error Handling Tests
```rust
// tests/tui/error_tests.rs
use super::harness::*;

tui_test!(test_invalid_agent_creation, 80, 24, |mut harness| {
    harness
        .navigate_to(Screen::Agents)
        .send_key(KeyCode::Char('n'))
        .press_enter()  // Empty name
        .assert_screen_contains("Agent name is required");
});

tui_test!(test_task_execution_error, 80, 24, |mut harness| {
    harness
        .navigate_to(Screen::Tasks)
        .send_key(KeyCode::Char('e'))  // Execute without selection
        .assert_screen_contains("No task selected");
});
```

### 3.5 Integration Tests
```rust
// tests/tui/integration_tests.rs
use super::harness::*;

tui_test!(test_agent_task_integration, 80, 24, |mut harness| {
    // Create agent
    harness
        .navigate_to(Screen::Agents)
        .send_key(KeyCode::Char('n'))
        .send_text("IntegrationAgent")
        .press_tab()
        .send_text("Integration test agent")
        .press_enter()
        
        // Create task for agent
        .navigate_to(Screen::Tasks)
        .send_key(KeyCode::Char('n'))
        .send_text("Integration Task")
        .press_tab()
        .send_text("Task for integration testing")
        .press_tab()
        .send_text("IntegrationAgent")
        .press_enter()
        
        // Execute task
        .send_key(KeyCode::Char('e'))
        .wait_for_text("Task executing", Duration::from_secs(5))
        
        // Monitor execution
        .navigate_to(Screen::Execution)
        .assert_screen_contains("Integration Task")
        .assert_screen_contains("IntegrationAgent");
});
```

## 4. E2E Testing with Expectrl

### 4.1 Full Application Tests
```rust
// tests/e2e/full_app_tests.rs
use expectrl::spawn;
use std::time::Duration;

#[tokio::test]
async fn test_full_agent_lifecycle() {
    let mut session = spawn("./target/debug/nox-tui").unwrap();
    
    // Wait for startup
    session.expect("Nox Agent Ecosystem").unwrap();
    
    // Navigate to agents
    session.send_line("1").unwrap();
    session.expect("Agent Management").unwrap();
    
    // Create new agent
    session.send_line("n").unwrap();
    session.expect("Agent Name:").unwrap();
    session.send_line("E2EAgent").unwrap();
    session.expect("System Prompt:").unwrap();
    session.send_line("E2E test agent").unwrap();
    session.send_line("").unwrap();  // Confirm creation
    
    // Verify creation
    session.expect("Agent created successfully").unwrap();
    session.expect("E2EAgent").unwrap();
    
    // Start agent
    session.send_line("s").unwrap();
    session.expect("Agent started").unwrap();
    
    // Create task
    session.send_line("2").unwrap();  // Navigate to tasks
    session.expect("Task Management").unwrap();
    session.send_line("n").unwrap();
    session.expect("Task Title:").unwrap();
    session.send_line("E2E Task").unwrap();
    session.expect("Description:").unwrap();
    session.send_line("End-to-end test task").unwrap();
    session.expect("Agent:").unwrap();
    session.send_line("E2EAgent").unwrap();
    session.send_line("").unwrap();
    
    // Execute task
    session.send_line("e").unwrap();
    session.expect("Task executing").unwrap();
    
    // Monitor execution
    session.send_line("3").unwrap();
    session.expect("Task Execution Monitor").unwrap();
    session.expect("E2E Task").unwrap();
    
    // Cleanup
    session.send_line("q").unwrap();
}
```

## 5. Visual Testing and Screenshots

### 5.1 Screenshot Comparison
```rust
// tests/visual/screenshot_tests.rs
use image::{ImageBuffer, Rgb};
use std::process::Command;

pub fn capture_terminal_screenshot(width: u16, height: u16) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // Use termshot or similar tool to capture terminal
    // This is a placeholder implementation
    ImageBuffer::new(width as u32 * 8, height as u32 * 16)
}

#[test]
fn test_dashboard_visual_regression() {
    let mut harness = TuiTestHarness::new(80, 24);
    harness.render();
    
    let screenshot = capture_terminal_screenshot(80, 24);
    let reference = image::open("tests/visual/references/dashboard.png").unwrap();
    
    // Compare images with tolerance
    let diff = image_diff::diff(&screenshot, &reference.to_rgb8());
    assert!(diff < 0.05, "Visual regression detected");
}
```

## 6. Test Configuration

### 6.1 Test Organization
```
tests/
├── tui/
│   ├── mod.rs
│   ├── harness.rs          # Main test harness
│   ├── macros.rs           # Test macros and DSL
│   ├── visual_tests.rs     # Visual rendering tests
│   ├── navigation_tests.rs # Navigation tests
│   ├── workflow_tests.rs   # User workflow tests
│   ├── error_tests.rs      # Error handling tests
│   └── integration_tests.rs # Integration tests
├── e2e/
│   ├── mod.rs
│   ├── full_app_tests.rs   # Full application tests
│   └── performance_tests.rs # Performance tests
├── visual/
│   ├── mod.rs
│   ├── screenshot_tests.rs # Visual regression tests
│   └── references/         # Reference screenshots
└── common/
    ├── mod.rs
    ├── fixtures.rs         # Test fixtures
    └── helpers.rs          # Common test utilities
```

### 6.2 Test Execution
```toml
# Cargo.toml
[profile.test]
opt-level = 0
debug = true

[[test]]
name = "tui_tests"
path = "tests/tui/mod.rs"
harness = false

[[test]]
name = "e2e_tests"
path = "tests/e2e/mod.rs"
harness = false
```

### 6.3 CI/CD Integration
```yaml
# .github/workflows/tui-tests.yml
name: TUI Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Run TUI unit tests
      run: cargo test --test tui_tests
    
    - name: Run E2E tests
      run: cargo test --test e2e_tests
    
    - name: Run visual regression tests
      run: cargo test --test visual_tests
```

## 7. Performance Testing

### 7.1 Load Testing
```rust
// tests/performance/load_tests.rs
use std::time::Instant;

#[tokio::test]
async fn test_large_agent_list_performance() {
    let mut harness = TuiTestHarness::new(80, 24);
    
    // Create many agents
    for i in 0..1000 {
        harness.app.add_agent(Agent::new(
            format!("Agent{}", i),
            "Test agent".to_string(),
        ));
    }
    
    let start = Instant::now();
    harness.navigate_to(Screen::Agents);
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 100, "Agent list rendering too slow");
}
```

## 8. Usage Examples

### 8.1 Basic Test
```rust
use tui_test_harness::*;

#[tokio::test]
async fn test_basic_navigation() {
    let mut harness = TuiTestHarness::new(80, 24);
    
    harness
        .when_i_navigate_to(Screen::Agents)
        .then_i_should_be_on(Screen::Agents)
        .then_i_should_see("Agent Management")
        .when_i_press(KeyCode::Esc)
        .then_i_should_be_on(Screen::Dashboard);
}
```

### 8.2 Complex Workflow Test
```rust
#[tokio::test]
async fn test_complete_workflow() {
    let mut harness = TuiTestHarness::new(80, 24);
    
    harness
        // Create agent
        .navigate_to(Screen::Agents)
        .send_key(KeyCode::Char('n'))
        .send_text("WorkflowAgent")
        .press_tab()
        .send_text("Workflow test agent")
        .press_enter()
        .assert_screen_contains("Agent created successfully")
        
        // Create task
        .navigate_to(Screen::Tasks)
        .send_key(KeyCode::Char('n'))
        .send_text("Workflow Task")
        .press_tab()
        .send_text("Test task description")
        .press_enter()
        .assert_screen_contains("Task created successfully")
        
        // Execute task
        .send_key(KeyCode::Char('e'))
        .wait_for_text("Task executing", Duration::from_secs(5))
        
        // Monitor execution
        .navigate_to(Screen::Execution)
        .assert_screen_contains("Workflow Task")
        .assert_screen_contains("WorkflowAgent");
}
```

This comprehensive testing framework provides:

1. **Unit-level testing** for individual components
2. **Integration testing** for component interactions
3. **E2E testing** for full application workflows
4. **Visual regression testing** for UI consistency
5. **Performance testing** for responsiveness
6. **Fluent DSL** for readable test code

The framework is designed to be:
- **Easy to use** with intuitive APIs
- **Comprehensive** covering all aspects of TUI testing
- **Maintainable** with clear structure and documentation
- **Automated** for CI/CD integration
- **Visual** with screenshot comparison capabilities

Would you like me to start implementing this testing framework alongside the TUI development?