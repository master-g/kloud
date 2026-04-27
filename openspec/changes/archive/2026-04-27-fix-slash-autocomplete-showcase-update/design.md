## Context

After t0-dead-code-wiring wired 6 widget functions into the render path, two input/rendering
bugs prevent them from being properly demonstrated:

1. `input.rs:61` maps `/` → `SearchActivate` when text area is empty, blocking autocomplete
2. `layout.rs::render_autocomplete()` clips popup height to `area.height` (input chunk = 3 rows)

The `tui_showcase` example has 19 scripted demos but the later interactive phases
(autocomplete, search, permission, notification) fail because `/` triggers search mode.

## Goals / Non-Goals

**Goals:**

- `/` key inserts character and triggers autocomplete (matching CC behavior)
- Search activated by dedicated key (`Ctrl+S`), not `/`
- Autocomplete popup renders at full height using available space above input
- Showcase demonstrates all 6 newly wired widgets
- Showcase interactive phases work correctly

**Non-Goals:**

- Changes to the render path or widget implementations (done in t0)
- New widget functions or capabilities
- Vim mode changes beyond `/` behavior fix

## Decisions

### D1: `/` always inserts character, never activates search

Current `input.rs:61`:
```rust
(KeyCode::Char('/'), KeyModifiers::NONE) if state.text_area.is_empty() => {
    Some(UiAction::SearchActivate)
}
```

This arm is removed entirely. `/` falls through to the insert-mode character handler
(line 160), which inserts `/` and triggers autocomplete filter logic (line 163-165).
In normal mode, `/` is a no-op (falls to `handle_normal_mode` → default → None), matching
Vim convention where `/` searches but our search key is different.

**Why**: CC's behavior is `/` → autocomplete popup. Users expect to type `/help` and see
the autocomplete dropdown, not a search bar.

### D2: Search activated by `Ctrl+S`

Add a new key binding in `input.rs`:
```rust
(KeyCode::Char('s'), KeyModifiers::CONTROL) => Some(UiAction::SearchActivate)
```

This is placed at the top level of the match (before mode-specific handling), so it works
in both insert and normal mode.

**Why**: `Ctrl+S` is a conventional search shortcut. It doesn't conflict with existing
bindings (Ctrl+C, Ctrl+D, Ctrl+O, Ctrl+U, Ctrl+K are taken; Ctrl+S is free).

### D3: Autocomplete popup uses available space above input

Current `layout.rs::render_autocomplete()`:
```rust
height: popup_height.min(area.height)  // area.height = 3 → clipped
```

Fixed: compute available height from `area.y` (distance to top of frame), not `area.height`:
```rust
height: popup_height.min(area.y)
```

This allows the popup to extend up to the full available space above the input area.

**Why**: The popup is an overlay above the input, not inside it. Using `area.height`
(the input chunk height) to limit the popup makes no sense — the popup renders outside
the input area.

### D4: Showcase update strategy

Add new demo methods to `DemoClient` and new entries to the `stream_demos` array:

| Demo | Method | What it shows |
|------|--------|---------------|
| Tool spinners | `tool_spinners_demo()` | Multiple active tools with spinner display |
| Diff tool result | `diff_tool_result_demo()` | Tool result containing unified diff (triggers `looks_like_diff`) |
| Toast notification | Already exists via `/notify` command | No new method needed |
| Hints | Visible in status bar during all demos | No dedicated demo needed |

Fix broken demos:
- **Autocomplete demo**: `/` now correctly inserts character + triggers autocomplete
- **Search demo**: Use `Ctrl+S` key injection instead of `/`

## Risks / Trade-offs

- **[Risk] Ctrl+S terminal conflict** — Some terminals intercept Ctrl+S for flow control
  (XOFF). Mitigation: most modern terminals have disabled this; users can run
  `stty -ixon` if needed. Alternative: `Ctrl+F` (also conventional).
- **[Trade-off] No Vim `/` search** — Vim users expect `/` to search. We use Ctrl+S instead.
  Mitigation: this matches CC's behavior where `/` triggers autocomplete, not search.
