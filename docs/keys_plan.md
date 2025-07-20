# TUI Keys Implementation Plan

## Overview
This document outlines the implementation plan for missing keyboard shortcuts and key bindings in the Nox TUI application. After reviewing all ratatui screens, several visual keys are displayed to users but lack proper implementation.

## Analysis Summary

### Current Status
- **Total Visual Keys Found**: 47 distinct key bindings displayed across all screens
- **Properly Implemented**: 28 keys (59%)
- **Missing Implementation**: 19 keys (41%)
- **Key Conflicts**: 3 major conflicts identified

### Critical Issues
1. **Execution Screen**: Almost all displayed keys lack implementation
2. **Logs Screen**: Majority of displayed keys lack implementation  
3. **Dashboard Screen**: Navigation keys missing
4. **Key Conflicts**: Same keys used for different functions across screens

## Implementation Tasks

### 🔥 **CRITICAL PRIORITY - Execution Screen**

#### Task 1: Implement Execution Screen Key Handler
**File**: `src/tui/app.rs`
**Location**: Add new method `handle_execution_keys()` and update `handle_key_input()` line 239

**Keys to implement**:
- `Space` - Pause/Resume execution
- `Del` - Cancel execution  
- `Enter` - View execution details
- `P` - Pause execution
- `R` - Resume execution
- `C` - Cancel execution

**Implementation Notes**:
- Add execution screen handler similar to `handle_agents_keys()` and `handle_tasks_keys()`
- Implement pause/resume functionality for running tasks
- Add cancel confirmation dialog for active executions
- Create execution details view/dialog

#### Task 2: Create Execution Operations
**File**: `src/tui/app.rs`
**New Methods**:
- `pause_execution(&mut self, task: Task)`
- `resume_execution(&mut self, task: Task)`
- `cancel_execution(&mut self, task: Task)`
- `show_execution_details(&mut self, task: Task)`

### 🔥 **CRITICAL PRIORITY - Logs Screen**

#### Task 3: Implement Logs Screen Key Handler
**File**: `src/tui/app.rs`
**Location**: Add new method `handle_logs_keys()` and update `handle_key_input()` line 239

**Keys to implement**:
- `F` - Toggle filter panel
- `C` - Clear logs
- `S` - Save logs to file
- `/` - Search in logs
- `Space` - Toggle auto-scroll
- `Home/End` - Jump to beginning/end
- `Enter` - View log entry details
- `R` - Refresh logs
- `A` - Toggle auto-scroll

**Implementation Notes**:
- Add logs screen handler method
- Implement log filtering system
- Add file save dialog for logs
- Create log search functionality
- Add auto-scroll toggle state

#### Task 4: Create Logs Operations
**File**: `src/tui/app.rs`
**New Methods**:
- `toggle_log_filter(&mut self)`
- `clear_logs(&mut self)`
- `save_logs_to_file(&mut self)`
- `search_logs(&mut self)`
- `toggle_auto_scroll(&mut self)`
- `jump_to_log_start(&mut self)`
- `jump_to_log_end(&mut self)`
- `show_log_details(&mut self, log_entry: LogEntry)`

### 🔥 **CRITICAL PRIORITY - Dashboard Screen**

#### Task 5: Implement Dashboard Screen Key Handler
**File**: `src/tui/app.rs`
**Location**: Add new method `handle_dashboard_keys()` and update `handle_key_input()` line 239

**Keys to implement**:
- `←/→` - Navigate dashboard panels
- `Enter` - Select/activate dashboard item

**Implementation Notes**:
- Add dashboard-specific navigation
- Implement panel switching within dashboard
- Add item selection functionality

#### Task 6: Create Dashboard Operations
**File**: `src/tui/app.rs`
**New Methods**:
- `navigate_dashboard_left(&mut self)`
- `navigate_dashboard_right(&mut self)`
- `select_dashboard_item(&mut self)`

### 🔶 **HIGH PRIORITY - Missing Global Keys**

#### Task 7: Add Missing Global Key Support
**File**: `src/tui/app.rs`
**Location**: Update `handle_global_keys()` method (lines 244-261)

**Keys to implement**:
- `H` - Help (currently shown in header but not implemented)

**Implementation Notes**:
- Add `H` key as alternative to `?` and `F1` for help
- Update help text to reflect all available help keys

### 🔶 **HIGH PRIORITY - Task Filter Keys**

#### Task 8: Implement Task Filter Keys
**File**: `src/tui/app.rs`
**Location**: Update `handle_tasks_keys()` method (lines 384-418)

**Keys to implement**:
- `A` - Show all tasks filter
- `R` - Show running tasks filter  
- `P` - Show pending tasks filter
- `C` - Show completed tasks filter

**Implementation Notes**:
- Add filter state management to AppState
- Implement filter logic for task lists
- Update task display to respect filters

#### Task 9: Create Task Filter Operations
**File**: `src/tui/app.rs`
**New Methods**:
- `filter_tasks_all(&mut self)`
- `filter_tasks_running(&mut self)`
- `filter_tasks_pending(&mut self)`
- `filter_tasks_completed(&mut self)`

### 🔶 **HIGH PRIORITY - Agent Restart Function**

#### Task 10: Implement Agent Restart Key
**File**: `src/tui/app.rs`
**Location**: Update `handle_agents_keys()` method (lines 347-381)

**Keys to implement**:
- `R` - Restart agent

**Implementation Notes**:
- Add restart operation that stops then starts agent
- Show progress dialog during restart
- Handle restart confirmation for critical agents

#### Task 11: Create Agent Restart Operation
**File**: `src/tui/app.rs`
**New Methods**:
- `restart_agent_operation(&mut self, agent: Agent)`
- `show_restart_agent_confirmation(&mut self, agent: Agent)`

### 🔶 **HIGH PRIORITY - List Selection Keys**

#### Task 12: Implement Enter Key for List Selection
**File**: `src/tui/app.rs`
**Location**: Update screen-specific key handlers

**Keys to implement**:
- `Enter` - Select/activate list item (context-dependent)

**Implementation Notes**:
- Add Enter key handling to agents screen (show agent details)
- Add Enter key handling to tasks screen (show task details)
- Create detail view dialogs for agents and tasks

#### Task 13: Create Detail View Operations
**File**: `src/tui/app.rs`
**New Methods**:
- `show_agent_details(&mut self, agent: Agent)`
- `show_task_details(&mut self, task: Task)`

### 🔷 **MEDIUM PRIORITY - Key Conflicts Resolution**

#### Task 14: Resolve Key Conflicts
**File**: `src/tui/app.rs`
**Location**: Review and update key mappings

**Conflicts to resolve**:
- `R` key: Used for Restart (agents), Running filter (tasks), Resume (execution)
- `C` key: Used for Cancel (tasks/execution), Clear (logs), Complete filter (tasks)  
- `P` key: Used for Pause (execution), Pending filter (tasks)

**Proposed Solutions**:
- **R Key**: Keep as Restart (agents), use `Ctrl+R` for filters, use `Space` for Resume
- **C Key**: Keep as Cancel (tasks/execution), use `Ctrl+C` for Clear, use `Shift+C` for Complete filter
- **P Key**: Keep as Pause (execution), use `Shift+P` for Pending filter

#### Task 15: Update Key Mappings
**Files**: All screen files and help documentation
**Changes**:
- Update visual key displays in screen footers
- Update help dialog key listings
- Update key handler implementations

### 🔷 **MEDIUM PRIORITY - Advanced Navigation**

#### Task 16: Implement Advanced Navigation Keys
**File**: `src/tui/app.rs`
**Location**: Update navigation handlers

**Keys to implement**:
- `Home/End` - Jump to list beginning/end
- `PageUp/PageDown` - Scroll by page

**Implementation Notes**:
- Add to all list-based screens
- Implement smooth scrolling
- Handle edge cases (empty lists, single items)

### 🔷 **MEDIUM PRIORITY - Search and Filter Enhancement**

#### Task 17: Enhance Search Functionality
**File**: `src/tui/app.rs`
**Location**: Update search implementation

**Improvements**:
- Real-time search results
- Search highlighting
- Search history
- Advanced search operators

#### Task 18: Implement Advanced Filters
**File**: `src/tui/app.rs`
**Location**: Extend filter system

**Features**:
- Multiple filter criteria
- Filter presets
- Filter save/load
- Filter quick toggle

### 🔷 **LOW PRIORITY - Documentation and Polish**

#### Task 19: Update Help Dialog
**File**: `src/tui/dialogs/help.rs`
**Location**: Update help content (lines 92-232)

**Changes**:
- Remove references to unimplemented keys
- Add documentation for new key implementations
- Ensure consistency with actual handlers

#### Task 20: Add Missing Visual Indicators
**Files**: All screen files
**Changes**:
- Add visual feedback for key presses
- Add status indicators for toggles
- Add progress indicators for operations

## Implementation Order

### Phase 1: Core Missing Handlers (Critical)
1. Task 1: Execution Screen Key Handler
2. Task 2: Execution Operations
3. Task 3: Logs Screen Key Handler
4. Task 4: Logs Operations
5. Task 5: Dashboard Screen Key Handler
6. Task 6: Dashboard Operations

### Phase 2: Essential Missing Keys (High Priority)
1. Task 7: Global Key Support (H key)
2. Task 8: Task Filter Keys
3. Task 9: Task Filter Operations
4. Task 10: Agent Restart Key
5. Task 11: Agent Restart Operation
6. Task 12: List Selection Keys
7. Task 13: Detail View Operations

### Phase 3: Conflicts and Enhancement (Medium Priority)
1. Task 14: Key Conflicts Resolution
2. Task 15: Update Key Mappings
3. Task 16: Advanced Navigation
4. Task 17: Search Enhancement
5. Task 18: Advanced Filters

### Phase 4: Polish and Documentation (Low Priority)
1. Task 19: Update Help Dialog
2. Task 20: Visual Indicators

## Estimated Timeline

- **Phase 1**: 2-3 days (Core functionality)
- **Phase 2**: 2-3 days (Essential features)  
- **Phase 3**: 3-4 days (Enhancements)
- **Phase 4**: 1-2 days (Polish)

**Total Estimated Time**: 8-12 days

## Testing Requirements

For each implemented key:
1. **Functional Testing**: Verify key performs expected action
2. **Visual Testing**: Confirm visual feedback is appropriate
3. **Context Testing**: Ensure key works in correct contexts only
4. **Conflict Testing**: Verify no conflicts with existing keys
5. **Edge Case Testing**: Test with empty lists, error states, etc.

## Notes

- All implementations should follow existing code patterns
- Error handling should be consistent with current approach
- Visual feedback should match existing UI style
- Help documentation must be updated with each change
- Consider accessibility implications for key choices