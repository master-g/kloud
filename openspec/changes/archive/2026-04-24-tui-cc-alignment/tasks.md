## 1. Cost Tracker Infrastructure

- [x] 1.1 Create `PricingTier` struct and `PRICING_TABLE` static in `src/agent/store.rs` with entries for claude-sonnet-4-6, claude-opus-4-7, claude-haiku-4-5, and a default tier
- [x] 1.2 Add `session_cost: f64` and `session_started_at: Option<Instant>` fields to `SessionStore`, compute cost on `UsageUpdated`, expose in `SessionView`
- [x] 1.3 Add `session_duration: Duration` computation to `SessionView` from `session_started_at`
- [x] 1.4 Verify `cargo check` passes

## 2. Status Bar Cost and Duration

- [x] 2.1 Add `session_cost` and `session_duration` fields to `TuiState`, populated from `SessionView` in `apply_view`
- [x] 2.2 Update `render_status_bar` in `widgets/layout.rs` to display cost (`$X.XX`) and duration (`Xm Xs`) after model name
- [x] 2.3 Add width-aware truncation: hide cost/duration when terminal width < 80 columns

## 3. Diff Renderer Widget

- [x] 3.1 Create `src/ui/tui/widgets/diff.rs` with `render_diff` function that takes unified diff string and renders colored lines with line-number gutter
- [x] 3.2 Add truncation support: render first N lines with "... M more lines" indicator
- [x] 3.3 Register `diff` module in `widgets/mod.rs`

## 4. Permission Prompt Widget

- [x] 4.1 Add `PermissionResponse { allowed: bool }` variant to `UiAction` in `src/ui/events.rs`
- [x] 4.2 Create `src/ui/tui/widgets/permission.rs` with `render_permission_prompt` function showing label, description, diff preview, and `[y] Allow / [n] Deny` buttons
- [x] 4.3 Add permission prompt key handling in `src/ui/tui/input.rs`: when `pending_permission` is active, `y` sends `PermissionResponse { allowed: true }`, `n` sends `PermissionResponse { allowed: false }`
- [x] 4.4 Integrate permission prompt rendering into `widgets/messages.rs` layout (render below last assistant message when pending)
- [x] 4.5 Handle `PermissionResponse` in `Session::run` idle state (pass through to tool execution layer)

## 5. Command Autocomplete

- [x] 5.1 Add `AutocompleteState` struct to `TuiState` with fields: `visible: bool`, `items: Vec<(String, String)>`, `selected: usize`, `filter: String`
- [x] 5.2 Update `handle_event` in `input.rs`: when input starts with `/`, populate and show autocomplete; `Up`/`Down` navigate, `Tab` completes, `Esc` dismisses
- [x] 5.3 Create `render_autocomplete` function in `widgets/layout.rs`: render filtered command list below input with selection highlight
- [x] 5.4 Add autocomplete rendering to the main layout in `widgets/layout.rs`

## 6. Compact Tool Spinner

- [x] 6.1 Add `tool_start_times: HashMap<String, Instant>` to `TuiState` to track elapsed time per active tool
- [x] 6.2 Create `render_tool_spinners` in `widgets/activity_line.rs`: render `● tool-name X.Xs` for each active tool
- [x] 6.3 Integrate compact spinner: when `active_tools` is non-empty, show compact spinners instead of full shimmer verb in the activity line

## 7. Notification Toasts

- [x] 7.1 Create `Notification` struct with `text: String`, `level: MessageLevel`, `created_at: Instant`, `dismiss_after: Duration`
- [x] 7.2 Add `notifications: Vec<Notification>` to `TuiState`, push on system messages, check expiry in tick loop
- [x] 7.3 Create `render_toasts` function: render up to 2 styled notifications above the input area
- [x] 7.4 Integrate toast rendering into the main layout

## 8. Command Hints Bar

- [x] 8.1 Create `render_hints` function in `widgets/layout.rs` that returns a one-line `Line` based on current state (Idle/Streaming/Search/Transcript, Insert/Normal mode)
- [x] 8.2 Integrate hints bar rendering below the input area in the main layout

## 9. Polish and Verification

- [x] 9.1 Update `examples/tui_showcase.rs` to exercise new features: send `PermissionResponse`, show cost, trigger autocomplete via key injection
- [x] 9.2 Run `cargo fmt --all && cargo clippy -- -W warnings`
- [x] 9.3 Run full demo end-to-end and verify all new widgets render correctly
