## Why

`TuiState` (995 lines) is a monolith mixing message state, animation, input, scroll,
and theme management. All 48 fields live in one struct. Layout uses dynamic row
constraints but lacks overlay/modal layers needed by T4–T6. T0 is complete — now is
the right time to reorganize before adding visual complexity in T2+.

## What Changes

- Split `TuiState` into 5 focused sub-modules: `app_state`, `input_state`,
  `scroll_state`, `anim_state`, `theme_state`
- Extract animation types (`ActivityClock`, `StalledState`, `TokenCounter`,
  `ThinkingStatus`) from `state.rs` to `state/anim_state.rs`
- Move `SearchState`, `AutocompleteState` to `state/input_state.rs`
- Move `Notification`, `CollapsedTools` to `state/app_state.rs`
- Move `ActivityEntry`, `LiveActivity`, `ActivitySnapshot` to `state/anim_state.rs`
- Eliminate redundant `scroll: u16` field — route all scroll through VirtualScroll
- Relocate `src/ui/constants.rs` → `src/ui/tui/constants.rs`
- Add dynamic bottom height (variable input area based on `TextArea.line_count()`)
- Establish overlay render pattern (already exists in autocomplete) as documented
  convention for T4/T6

## Capabilities

### New Capabilities

- `state-split`: TuiState decomposed into focused sub-modules with thin facade
- `dynamic-input-height`: Input area height adapts to multi-line content

### Modified Capabilities

- `tui-virtual-scroll`: Remove redundant `scroll: u16`, consolidate scroll operations
  through VirtualScroll API
- `sticky-scroll`: No spec change — already implemented, just removing the duplicate
  `scroll: u16` field that bypasses it

## Impact

- **Files**: `state.rs` (split into 5 files), `mod.rs` (tick delegation),
  `input.rs` (scroll calls), `widgets/mod.rs` (dynamic constraint), `widgets/layout.rs`
- **Dependencies**: No new crates. Internal reorganization only.
- **API**: `TuiState` public interface preserved through facade pattern. Internal
  field access may need `pub` adjustments.
- **Risk**: Low — pure structural refactor. All existing tests must pass unchanged.
