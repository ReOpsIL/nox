use ratatui::style::{Color, Style, Modifier};

// Primary color palette - vibrant and dark-background friendly
pub const PRIMARY_COLOR: Color = Color::Rgb(0, 191, 255);        // Deep Sky Blue
pub const SECONDARY_COLOR: Color = Color::Rgb(0, 0, 0);     // Black
pub const ACCENT_COLOR: Color = Color::Rgb(255, 20, 147);       // Deep Pink

// Status colors - bright and visible on dark backgrounds
pub const SUCCESS_COLOR: Color = Color::Rgb(50, 205, 50);       // Lime Green
pub const WARNING_COLOR: Color = Color::Rgb(255, 165, 0);       // Orange
pub const ERROR_COLOR: Color = Color::Rgb(255, 69, 0);          // Red Orange
pub const INFO_COLOR: Color = Color::Rgb(173, 216, 230);        // Light Blue

// Text colors
pub const TEXT_PRIMARY: Color = Color::Rgb(255, 255, 255);      // White
pub const TEXT_SECONDARY: Color = Color::Rgb(192, 192, 192);    // Silver
pub const TEXT_MUTED: Color = Color::Rgb(128, 128, 128);        // Gray
pub const TEXT_HIGHLIGHT: Color = Color::Rgb(255, 215, 0);      // Gold

// Background and UI colors
pub const BACKGROUND_COLOR: Color = Color::Black;
pub const BORDER_COLOR: Color = Color::Rgb(75, 0, 130);         // Indigo
pub const BORDER_ACTIVE: Color = Color::Rgb(0, 0, 0);           // Black
pub const SELECTION_BG: Color = Color::Rgb(25, 25, 112);        // Midnight Blue

// Agent status colors
pub const AGENT_ACTIVE: Color = Color::Rgb(0, 255, 127);        // Spring Green
pub const AGENT_INACTIVE: Color = Color::Rgb(169, 169, 169);    // Dark Gray
pub const AGENT_ERROR: Color = Color::Rgb(255, 0, 0);           // Red
pub const AGENT_STARTING: Color = Color::Rgb(255, 255, 0);      // Yellow
pub const AGENT_STOPPING: Color = Color::Rgb(255, 140, 0);      // Dark Orange

// Task priority colors
pub const PRIORITY_HIGH: Color = Color::Rgb(255, 0, 255);       // Magenta
pub const PRIORITY_MEDIUM: Color = Color::Rgb(255, 165, 0);     // Orange
pub const PRIORITY_LOW: Color = Color::Rgb(173, 255, 47);       // Green Yellow

// Task status colors
pub const TASK_TODO: Color = Color::Rgb(135, 206, 235);         // Sky Blue
pub const TASK_RUNNING: Color = Color::Rgb(0, 255, 255);        // Cyan
pub const TASK_DONE: Color = Color::Rgb(144, 238, 144);         // Light Green
pub const TASK_CANCELLED: Color = Color::Rgb(255, 99, 71);      // Tomato

// Progress and gauge colors
pub const PROGRESS_BG: Color = Color::Rgb(47, 79, 79);          // Dark Slate Gray
pub const PROGRESS_FG: Color = Color::Rgb(0, 206, 209);         // Dark Turquoise

// Style functions
pub fn primary_style() -> Style {
    Style::default().fg(PRIMARY_COLOR).add_modifier(Modifier::BOLD)
}

pub fn secondary_style() -> Style {
    Style::default().fg(SECONDARY_COLOR)
}

pub fn accent_style() -> Style {
    Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)
}

pub fn success_style() -> Style {
    Style::default().fg(SUCCESS_COLOR).add_modifier(Modifier::BOLD)
}

pub fn warning_style() -> Style {
    Style::default().fg(WARNING_COLOR).add_modifier(Modifier::BOLD)
}

pub fn error_style() -> Style {
    Style::default().fg(ERROR_COLOR).add_modifier(Modifier::BOLD)
}

pub fn info_style() -> Style {
    Style::default().fg(INFO_COLOR)
}

pub fn text_primary_style() -> Style {
    Style::default().fg(TEXT_PRIMARY)
}

pub fn text_secondary_style() -> Style {
    Style::default().fg(TEXT_SECONDARY)
}

pub fn muted_style() -> Style {
    Style::default().fg(TEXT_MUTED)
}

pub fn highlight_style() -> Style {
    Style::default().fg(TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default()
        .fg(TEXT_PRIMARY)
        .bg(SELECTION_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn border_style() -> Style {
    Style::default().fg(BORDER_COLOR)
}

pub fn border_active_style() -> Style {
    Style::default().fg(BORDER_ACTIVE).add_modifier(Modifier::BOLD)
}

pub fn title_style() -> Style {
    Style::default()
        .fg(PRIMARY_COLOR)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

// Agent status styles
pub fn agent_active_style() -> Style {
    Style::default().fg(AGENT_ACTIVE).add_modifier(Modifier::BOLD)
}

pub fn agent_inactive_style() -> Style {
    Style::default().fg(AGENT_INACTIVE)
}

pub fn agent_error_style() -> Style {
    Style::default().fg(AGENT_ERROR).add_modifier(Modifier::BOLD)
}

pub fn agent_starting_style() -> Style {
    Style::default().fg(AGENT_STARTING).add_modifier(Modifier::SLOW_BLINK)
}

pub fn agent_stopping_style() -> Style {
    Style::default().fg(AGENT_STOPPING)
}

// Task priority styles
pub fn priority_high_style() -> Style {
    Style::default().fg(PRIORITY_HIGH).add_modifier(Modifier::BOLD)
}

pub fn priority_medium_style() -> Style {
    Style::default().fg(PRIORITY_MEDIUM)
}

pub fn priority_low_style() -> Style {
    Style::default().fg(PRIORITY_LOW)
}

// Task status styles
pub fn task_todo_style() -> Style {
    Style::default().fg(TASK_TODO)
}

pub fn task_running_style() -> Style {
    Style::default().fg(TASK_RUNNING).add_modifier(Modifier::BOLD)
}

pub fn task_done_style() -> Style {
    Style::default().fg(TASK_DONE).add_modifier(Modifier::BOLD)
}

pub fn task_cancelled_style() -> Style {
    Style::default().fg(TASK_CANCELLED).add_modifier(Modifier::ITALIC)
}

// Progress bar styles
pub fn progress_style() -> Style {
    Style::default().fg(PROGRESS_FG).bg(PROGRESS_BG)
}

pub fn progress_complete_style() -> Style {
    Style::default().fg(SUCCESS_COLOR).bg(PROGRESS_BG).add_modifier(Modifier::BOLD)
}