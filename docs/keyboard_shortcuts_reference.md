# NOX TUI Keyboard Shortcuts Reference

Generated: 2025-07-18 05:17:11 UTC

## Global Shortcuts

These shortcuts work on all screens:

| Key | Action | Description |
|-----|--------|--------------|
| `q` / `Q` | Quit | Exit the application |
| `?` / `F1` / `h` / `H` | Help | Show help dialog |
| `Tab` | Next Screen | Navigate to next screen |
| `Shift+Tab` | Previous Screen | Navigate to previous screen |
| `1` | Dashboard | Switch to Dashboard screen |
| `2` | Agents | Switch to Agents screen |
| `3` | Tasks | Switch to Tasks screen |
| `4` | Execution | Switch to Execution screen |
| `5` | Logs | Switch to Logs screen |
| `Up` | Navigate Up | Move selection up in lists |
| `Down` | Navigate Down | Move selection down in lists |

## Dashboard Screen

| Key | Action | Description |
|-----|--------|--------------|
| `Left` | Navigate Left | Move dashboard focus left |
| `Right` | Navigate Right | Move dashboard focus right |
| `Enter` | Select Item | Activate selected dashboard item |

## Agents Screen

| Key | Action | Description |
|-----|--------|--------------|
| `n` / `N` | New Agent | Open create agent form |
| `e` / `E` | Edit Agent | Open edit agent form |
| `s` / `S` | Start Agent | Start selected agent |
| `t` / `T` | Stop Agent | Show stop agent confirmation |
| `d` / `D` | Delete Agent | Show delete agent confirmation |
| `r` / `R` | Restart Agent | Show restart agent confirmation |
| `Enter` | View Details | Show agent details dialog |
| `/` | Search | Activate search mode |
| `f` / `F` | Filter | Show filter options |

## Tasks Screen

| Key | Action | Description |
|-----|--------|--------------|
| `n` / `N` | New Task | Open create task form |
| `e` / `E` | Execute Task | Execute selected task |
| `u` / `U` | Update Task | Open edit task form |
| `d` / `D` | Delete Task | Show delete task confirmation |
| `c` / `C` | Cancel Task | Show cancel task confirmation |
| `Enter` | View Details | Show task details dialog |
| `a` / `A` | Filter All | Show all tasks |
| `r` / `R` | Filter Running | Show running tasks |
| `p` / `P` | Filter Pending | Show pending tasks |
| `/` | Search | Activate search mode |
| `f` / `F` | Filter | Show filter options |

## Execution Screen

| Key | Action | Description |
|-----|--------|--------------|
| `Space` | Pause/Resume | Toggle execution pause |
| `Delete` | Cancel | Cancel execution |
| `Enter` | View Details | Show execution details |
| `p` / `P` | Pause | Pause execution |
| `r` / `R` | Resume | Resume execution |
| `c` / `C` | Cancel | Cancel execution |
| `/` | Search | Activate search mode |
| `f` / `F` | Filter | Show filter options |

## Logs Screen

| Key | Action | Description |
|-----|--------|--------------|
| `f` / `F` | Toggle Filter | Toggle log filter panel |
| `c` / `C` | Clear Logs | Show clear logs confirmation |
| `s` / `S` | Save Logs | Save logs to file |
| `/` | Search | Search in logs |
| `Space` | Auto-scroll | Toggle auto-scroll |
| `Home` | Jump to Start | Jump to beginning of logs |
| `End` | Jump to End | Jump to end of logs |
| `Enter` | View Details | Show log entry details |
| `r` / `R` | Refresh | Refresh logs |
| `a` / `A` | Auto-scroll | Toggle auto-scroll |

## Implementation Status

This documentation is generated from the testing framework specifications.
Run `nox-test-tui run-all` to test the actual implementation status of these shortcuts.

## Usage Notes

- Keys are case-insensitive unless otherwise noted
- Some actions may require selecting an item first
- Global shortcuts work from any screen
- Form and dialog interactions may temporarily override these shortcuts
