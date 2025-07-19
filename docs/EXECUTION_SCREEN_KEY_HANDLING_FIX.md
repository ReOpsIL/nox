# Execution Screen Key Handling Fix

## Issue
In the "Running Tasks" page (Execution screen), the A, P, R keys were still handling events even after all tasks were canceled and the running tasks list was empty. This caused actions to be triggered on non-running tasks or when there were no running tasks at all.

## Root Cause
The key handlers for the Execution screen were using `get_selected_task()` which returns any selected task, not just running tasks. This meant that actions could be triggered on completed, cancelled, or todo tasks, even when there were no running tasks visible.

## Solution

### 1. **Enhanced Task Selection Logic** (`src/tui/app.rs`)

#### Added `get_selected_running_task()` method:
```rust
fn get_selected_running_task(&self) -> Option<&Task> {
    if let Some(task) = self.get_selected_task() {
        if task.status == crate::types::TaskStatus::InProgress {
            return Some(task);
        }
    }
    None
}
```

#### Added `has_running_tasks()` helper:
```rust
fn has_running_tasks(&self) -> bool {
    self.state.tasks.iter().any(|task| task.status == crate::types::TaskStatus::InProgress)
}
```

### 2. **Updated Key Handlers** (`src/tui/app.rs`)

#### All execution screen key handlers now use `get_selected_running_task()`:
- **Space** - Pause/Resume execution
- **Delete** - Cancel execution  
- **Enter** - View execution details
- **P** - Pause execution
- **R** - Resume execution
- **C** - Cancel execution

#### Added explicit 'A' key handler:
```rust
crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Char('A') => {
    // A - Cancel All running tasks (only if there are running tasks)
    if self.has_running_tasks() {
        self.cancel_all_running_tasks();
    }
}
```

### 3. **Enhanced Auto-Selection Logic** (`src/tui/app.rs`)

#### Updated `auto_select_running_task()` method:
```rust
fn auto_select_running_task(&mut self) {
    let running_task_indices: Vec<usize> = self.state.tasks
        .iter()
        .enumerate()
        .filter(|(_, task)| task.status == crate::types::TaskStatus::InProgress)
        .map(|(i, _)| i)
        .collect();
    
    if !running_task_indices.is_empty() {
        // Select first running task if none selected or selected task is not running
        if let Some(selected_idx) = self.state.selected_task {
            if !running_task_indices.contains(&selected_idx) {
                self.state.selected_task = Some(running_task_indices[0]);
            }
        } else {
            self.state.selected_task = Some(running_task_indices[0]);
        }
    } else {
        // If no running tasks, clear the selection to prevent actions on non-running tasks
        self.state.selected_task = None;
    }
}
```

### 4. **Added Cancel All Functionality** (`src/tui/app.rs`)

#### New `cancel_all_running_tasks()` method:
```rust
fn cancel_all_running_tasks(&mut self) {
    let running_count = self.state.tasks.iter()
        .filter(|task| task.status == crate::types::TaskStatus::InProgress)
        .count();
    
    if running_count > 0 {
        let dialog = ConfirmationDialog::new(
            "Cancel All Running Tasks".to_string(),
            format!("Are you sure you want to cancel all {} running tasks?", running_count)
        );
        self.state.current_dialog = Some(DialogState::Confirmation(dialog));
        self.state.success_message = Some(format!("Prepared to cancel {} running tasks", running_count));
    }
}
```

### 5. **Enhanced UI Feedback** (`src/tui/screens/execution.rs`)

#### Dynamic footer based on running tasks:
```rust
let footer_text = if running_tasks.len() > 0 {
    format!(
        "[C] Cancel  [P] Pause  [R] Resume  [A] Cancel All\n\
         \n\
         Active: {}, Queued: {}, Slots: 5",
        running_tasks.len(),
        queued_tasks.len()
    )
} else {
    format!(
        "No running tasks\n\
         \n\
         Active: {}, Queued: {}, Slots: 5",
        running_tasks.len(),
        queued_tasks.len()
    )
};
```

### 6. **Data Refresh Integration** (`src/tui/app.rs`)

#### Updated `refresh_data()` method:
```rust
// Auto-select running task if we're on the execution screen
if self.state.current_screen == Screen::Execution {
    self.auto_select_running_task();
}
```

## Behavior Changes

### Before Fix:
- A, P, R keys could trigger actions on any selected task (even completed/cancelled ones)
- Keys worked even when there were no running tasks
- No visual feedback about available actions
- Selection could point to non-running tasks

### After Fix:
- **A, P, R keys only work when there are running tasks and a running task is selected**
- **'A' key now explicitly cancels all running tasks (with confirmation)**
- **Footer shows different text when no running tasks are available**
- **Selection automatically clears when no running tasks exist**
- **Key actions are properly guarded against non-running tasks**

## User Experience

### When Running Tasks Exist:
- Navigation works normally with arrow keys
- Actions (P, R, C, A) work only on running tasks
- Footer shows: `[C] Cancel  [P] Pause  [R] Resume  [A] Cancel All`
- Selection highlights running tasks

### When No Running Tasks:
- Selection is cleared (no task selected)
- Key actions (P, R, C, A) are ignored
- Footer shows: `No running tasks`
- Navigation keys are ignored

## Key Features

1. **Proper State Management**: Selection state is properly managed based on running tasks
2. **Action Guarding**: All actions require running tasks and valid selection
3. **Visual Feedback**: UI clearly indicates when actions are available
4. **Cancel All**: New bulk action for canceling all running tasks
5. **Automatic Cleanup**: Selection clears automatically when no running tasks

## Files Modified

1. `/Users/dovcaspi/develop/nox/src/tui/app.rs`
   - Added `get_selected_running_task()` method
   - Added `has_running_tasks()` helper
   - Updated all execution key handlers to use proper guards
   - Added explicit 'A' key handler
   - Enhanced `auto_select_running_task()` logic
   - Added `cancel_all_running_tasks()` method
   - Updated `refresh_data()` to auto-select running tasks

2. `/Users/dovcaspi/develop/nox/src/tui/screens/execution.rs`
   - Updated footer to show dynamic text based on running tasks
   - Added '[A] Cancel All' to footer when running tasks exist

The fix ensures that execution screen actions only work when appropriate, preventing confusing behavior and providing clear feedback to users about available actions.