# Running Tasks Navigation Fix

## Issue
The Running Tasks list in the TUI Execution screen had no navigation support - no selected task highlighting and arrow keys were not working for navigation.

## Root Cause
The `render_running_tasks` function in `/Users/dovcaspi/develop/nox/src/tui/screens/execution.rs` was displaying tasks without any selection state or visual indicators for the selected task.

## Solution
### 1. Added Navigation Logic (`src/tui/app.rs`)

**Enhanced `handle_list_navigation()` function:**
- Added `Screen::Execution` case to handle arrow key navigation
- Filters tasks to only show running tasks (`TaskStatus::InProgress`)
- Maintains proper selection state when navigating through running tasks

**Added `auto_select_running_task()` method:**
- Automatically selects the first running task when entering the Execution screen
- Ensures a running task is selected if the current selection is not a running task
- Called when switching to Execution screen via Tab navigation or direct key (4)

**Updated screen navigation methods:**
- `next_screen()` and `previous_screen()` now auto-select running tasks when entering Execution screen
- Direct navigation (pressing '4') also auto-selects running tasks

### 2. Enhanced Visual Selection (`src/tui/screens/execution.rs`)

**Updated `render_running_tasks()` function:**
- Changed to track both task index and task object: `Vec<(usize, &Task)>`
- Added selection indicator (`>` for selected, ` ` for unselected)
- Applied different styles for selected vs unselected tasks:
  - Selected: `selected_style()` for title and agent name
  - Unselected: `highlight_style()` for title, `primary_style()` for agent name

## Key Changes

### Navigation Logic
```rust
Screen::Execution => {
    // Get running tasks for navigation
    let running_task_indices: Vec<usize> = self.state.tasks
        .iter()
        .enumerate()
        .filter(|(_, task)| task.status == crate::types::TaskStatus::InProgress)
        .map(|(i, _)| i)
        .collect();
    
    // Navigate through running tasks only
    let new_display_pos = if direction > 0 {
        (current_display_pos + 1) % running_count
    } else {
        if current_display_pos == 0 { running_count - 1 } else { current_display_pos - 1 }
    };
    
    self.state.selected_task = Some(running_task_indices[new_display_pos]);
}
```

### Visual Selection
```rust
// Check if this task is selected
let is_selected = state.selected_task == Some(*task_index);
let selection_indicator = if is_selected { ">" } else { " " };

let title_style = if is_selected { selected_style() } else { highlight_style() };
let agent_style = if is_selected { selected_style() } else { primary_style() };
```

### Auto-Selection
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
    }
}
```

## User Experience Improvements

1. **Visual Feedback**: Selected running tasks now show a clear `>` indicator and highlighted styling
2. **Arrow Key Navigation**: Up/Down arrows now work to navigate between running tasks
3. **Auto-Selection**: When entering the Execution screen, the first running task is automatically selected
4. **Consistent Navigation**: Navigation behavior now matches other screens (Tasks, Agents)

## Testing

Users can now:
1. Navigate to the Execution screen (press '4' or Tab to reach it)
2. See the first running task automatically selected (if any exist)
3. Use Up/Down arrow keys to navigate between running tasks
4. See visual selection indicators and highlighting
5. Use existing execution commands (C for cancel, P for pause, etc.) on the selected task

## Backward Compatibility

- All existing functionality remains unchanged
- No breaking changes to existing key bindings
- Execution screen commands still work as before
- Only adds new navigation capabilities

## Files Modified

1. `/Users/dovcaspi/develop/nox/src/tui/app.rs`
   - Added `Screen::Execution` case to `handle_list_navigation()`
   - Added `auto_select_running_task()` method
   - Updated `next_screen()` and `previous_screen()` methods
   - Updated direct navigation handler for '4' key

2. `/Users/dovcaspi/develop/nox/src/tui/screens/execution.rs`
   - Modified `render_running_tasks()` to track task indices
   - Added selection visual indicators
   - Applied conditional styling based on selection state

The fix ensures that the Running Tasks list now has full navigation support consistent with other screens in the TUI application.