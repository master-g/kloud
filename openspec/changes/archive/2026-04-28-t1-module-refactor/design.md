## Context

`TuiState` in `state.rs` (995 lines) holds 48 fields spanning 5 domains: app state
(messages, permissions, notifications), input (text area, history, autocomplete),
scroll (VirtualScroll + redundant `scroll: u16`), animation (4 animation subsystems),
and theme. The `tick()` method (`mod.rs:157-168`) is only 12 lines that delegates to
field methods on self-contained animation structs.

Layout uses dynamic `Vec<Constraint>` in `widgets/mod.rs` — conditionally adds title
bar and notification row. Input area is hardcoded `Constraint::Length(3)`. Overlay
pattern already exists in autocomplete popup (`layout.rs:166-212`).

Animation types (`ActivityClock`, `StalledState`, `TokenCounter`, `ThinkingStatus`)
are self-contained structs in `state.rs:28-231` with their own fields and methods —
no complex ownership coupling.

## Goals / Non-Goals

**Goals:**

- Split `state.rs` into `state/` directory with 5 focused sub-modules
- Thin `TuiState` facade preserves existing public API
- Eliminate redundant `scroll: u16` field
- Dynamic input height based on `TextArea.line_count()`
- All existing tests updated for new field paths and pass

**Non-Goals:**

- New visual features (T2 scope)
- Overlay/modal infrastructure (T4/T6 scope)
- `Tickable` trait or complex trait abstractions
- Renaming `widgets/` to `components/` — cosmetic, not worth the import churn
- New crates or external dependencies
- `Screen::Search` variant decision — deferred to separate change

## Decisions

### D1: Facade pattern over full decomposition

`TuiState` stays as a single struct but fields are grouped into sub-structs:

```rust
pub struct TuiState {
    pub app: AppState,       // messages, permissions, notifications
    pub input: InputState,   // text_area, history, autocomplete, search
    pub scroll: ScrollState, // VirtualScroll wrapper
    pub anim: AnimState,     // clock, stall, token, thinking
    pub theme: ThemeState,   // theme switching
    // ...config fields that don't fit sub-modules (model, workspace, etc.)
}
```

**Why not separate structs with independent ownership?** `tick()` and render methods
need `&mut TuiState` — splitting into truly independent owners requires passing multiple
`&mut` references. Sub-structs within TuiState keep the single-owner pattern.

**Alternative considered**: `Tickable` trait with `fn tick(&mut self, dt: Duration)`.
Rejected — adds indirection for no benefit when `tick()` is already trivial delegation.

### D2: Extend VirtualScroll, don't replace

`virtual_scroll.rs` already has sticky scroll (`auto_scroll` flag), scroll position
tracking, and up/down/bottom/top methods. The redundant `scroll: u16` field in
TuiState will be removed. All scroll operations currently using `state.scroll` in
`input.rs` will be routed through `state.scroll.virtual_scroll.scroll_up()` etc.

### D3: Dynamic bottom height via `line_count()`

`widgets/mod.rs:48` changes from `Constraint::Length(3)` to
`Constraint::Length(input_height)` where `input_height` is computed from
`state.input.text_area.line_count().clamp(3, max_height/2)`. `max_height` is
the terminal rows minus allocated space for other sections.

### D4: File-level module split, not directory migration

Current `widgets/` (10 files) stays as-is. `state.rs` → `state/` directory.
`constants.rs` moved from `src/ui/` to `src/ui/tui/`. No directory renames.

### D5: Cross-domain methods — path change only, no logic split

8 methods access multiple sub-struct domains. The most complex:
- `apply_view()` (state.rs:546-601): updates anim + messages + input + app + tools
- `sync_from_active_view()` (state.rs:603-655): same scope

Strategy: these methods stay on `TuiState` (the facade). Field access paths
change (e.g., `self.messages` → `self.app.messages`) but method logic and
control flow remain identical. No method decomposition needed — the facade
exists precisely to contain these cross-cutting operations.

## Risks / Trade-offs

- **Field access through sub-structs** → All `state.field` becomes `state.app.field`
  or `state.anim.field`. Mechanical change but touches many files. Mitigation: grep
  all field accesses before starting, use `replace_all` where possible.
- **`pub` field exposure** → Sub-structs need `pub` fields for render methods.
  No encapsulation benefit from getters — TUI is internal code. Accept `pub` fields.
- **Test breakage** → Tests access `TuiState` fields directly. All must update to
  sub-struct paths. Mitigation: `cargo test` after each sub-module extraction.
- **Merge conflicts with concurrent work** → T1 is a large refactor. Coordinate
  with other branches. Mitigation: complete T1 before starting T2+.
