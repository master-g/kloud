## 1. Theme System

- [x] 1.1 Create `src/ui/tui/theme/mod.rs` with `Theme` and `ThemeColors` structs
- [x] 1.2 Implement 6 built-in themes (dark, light, dark-ansi, light-ansi, dark-daltonized, light-daltonized) with semantic color definitions matching Claude Code's theme.ts
- [x] 1.3 Add terminal color capability detection (COLORTERM check) and auto-select ANSI variant
- [x] 1.4 Add `/theme` command handler in Session and `UiAction::ChangeTheme`/`AppEvent::ThemeChanged` events
- [x] 1.5 Migrate all hardcoded colors in `widgets.rs` to use `theme.colors.*` semantic references
- [x] 1.6 Write unit tests for theme registry and color mapping

## 2. Component Architecture

- [x] 2.1 Create `src/ui/tui/components/mod.rs` with component registration and re-exports
- [x] 2.2 Extract message rendering into `components/messages.rs`
- [x] 2.3 Extract input rendering into `components/input.rs`
- [x] 2.4 Extract activity/status rendering into `components/activity.rs`
- [x] 2.5 Update `tui/mod.rs` event loop to use new component modules
- [x] 2.6 Verify `cargo test` passes after refactor with no behavioral changes

## 3. Multi-line Input

- [x] 3.1 Create `TextArea` widget struct with multi-line buffer, cursor position, and line tracking
- [x] 3.2 Implement multi-line rendering (dynamic height, cursor line highlight, line wrapping)
- [x] 3.3 Implement `Shift+Enter` / `Alt+Enter` for newline insertion vs `Enter` for submit
- [x] 3.4 Implement input history navigation (`Up`/`Down` at start/end of buffer)
- [x] 3.5 Implement slash command autocomplete menu (filter + display + Tab/Enter select)
- [x] 3.6 Implement basic Vim mode (Normal/Insert toggle, h/j/k/l, dd, x, i, Esc)
- [x] 3.7 Write tests for TextArea editing operations and history navigation

## 4. Virtual Scrolling

- [x] 4.1 Create `VirtualScroll` struct with message index, line height cache, and visible range tracking
- [x] 4.2 Implement line height calculation and caching (invalidate on terminal resize)
- [x] 4.3 Implement viewport calculation (visible messages + 5 buffer)
- [x] 4.4 Implement keyboard scrolling: j/k (1 line), Ctrl+U/Ctrl+D (half page), G/gg (end/top)
- [x] 4.5 Implement auto-scroll behavior (enabled when at bottom, disabled when user scrolls up)
- [x] 4.6 Add "auto-scroll paused" visual indicator
- [x] 4.7 Integrate virtual scroll into `components/messages.rs`
- [x] 4.8 Performance test with 1000 messages to verify <16ms render time

## 5. Streaming Animations

- [x] 5.1 Create `AnimationEngine` struct with tick timing, shimmer phase, breathing phase, and fade state
- [x] 5.2 Implement shimmer effect (left-to-right gradient sweep on activity line during streaming)
- [x] 5.3 Implement thinking breathing effect (slow opacity pulse, rainbow after 3s)
- [x] 5.4 Implement stall detection (3s no-token threshold) and blinking amber indicator
- [x] 5.5 Implement token count smooth animation (10% per tick toward target, no overshoot)
- [x] 5.6 Implement activity fade-out (1s / 20 frames after turn completion)
- [x] 5.7 Implement tool execution progress blinking (500ms filled/hollow toggle)
- [x] 5.8 Add `StreamStalled` and `StreamResumed` SessionEvent variants
- [x] 5.9 Wire animation engine into 50ms tick in TUI event loop

## 6. Tool Rendering

- [x] 6.1 Implement tool status indicators: running (blinking ●), success (green ●), error (red ●), cancelled (gray ○)
- [x] 6.2 Implement collapsible tool output (default collapsed if >5 lines, Tab to toggle)
- [x] 6.3 Implement batch tool grouping (header with count and progress)
- [x] 6.4 Implement tool type differentiation: Bash ($prefix, monospace), File (path highlight), WebSearch (URL style)
- [x] 6.5 Add `UiAction::ToggleToolCollapse(block_index)` and handle in SessionView
- [x] 6.6 Write tests for collapse state management and tool grouping logic

## 7. Status Bar

- [x] 7.1 Create `components/status_bar.rs` with full-width bottom status bar
- [x] 7.2 Display: model name, permission mode, token usage (in/out), cost, branch, elapsed time
- [x] 7.3 Implement context meter (progress bar with color thresholds: green <70%, yellow 70-90%, red >90%)
- [x] 7.4 Implement worktree indicator (show worktree name when in worktree, omit otherwise)
- [x] 7.5 Wire live token/cost updates from streaming pipeline to status bar

## 8. Search

- [x] 8.1 Implement `/` key to activate search mode (search bar at top of message area)
- [x] 8.2 Implement case-insensitive substring matching across all message text
- [x] 8.3 Implement match highlighting with `suggestion` color background
- [x] 8.4 Implement `n`/`N` navigation between matches with scroll-to-center
- [x] 8.5 Implement wrap-around with "search wrapped" indicator
- [x] 8.6 Implement match count display (e.g., `[3/12]`)
- [x] 8.7 Implement `Esc` to exit search and restore previous scroll position
- [x] 8.8 Add `Screen::Search` to SessionView and `UiAction::Search*` variants

## 9. Integration and Polish

- [x] 9.1 Extend `AppEvent` and `UiAction` enums with all new variants
- [x] 9.2 Update `TuiState` to hold `Theme`, `AnimationEngine`, `VirtualScroll`, and search state
- [x] 9.3 Full end-to-end test: start TUI, send message, stream response, use tools, search history
- [x] 9.4 Verify `cargo fmt --all && cargo clippy -- -W warnings` passes
- [x] 9.5 Verify all existing tests still pass
- [x] 9.6 Manual QA: verify visual fidelity against Claude Code screenshots
