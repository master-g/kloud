## Why

Two bugs block the newly-wired widgets from being properly exercised:

1. **`/` key activates search instead of autocomplete** — In `input.rs:61`, pressing `/`
   when the text area is empty triggers `UiAction::SearchActivate`, switching to
   `Screen::Search`. This prevents slash command autocomplete from ever showing.
   CC's actual behavior: `/` inserts the character and triggers the autocomplete popup.

2. **Autocomplete popup height clipped to 3 rows** — `render_autocomplete()` clips
   `popup_height` to `area.height` (the input chunk, always 3). A 4-item popup needs
   6 rows but renders only 3, cutting off content.

These bugs also break the `tui_showcase` example's later interactive phases (autocomplete
demo, search cycle) because `/` sends the showcase into search mode unexpectedly.

Additionally, the showcase lacks demos for the 6 widgets wired in t0-dead-code-wiring:
tool spinners, hints, toasts, diff in tool results, and the working autocomplete/permission.

## What Changes

- Fix `/` key handler: insert `/` character + trigger autocomplete, remove search activation
- Add dedicated search activation key (`Ctrl+S`)
- Fix autocomplete popup height: use available space above input, not `area.height`
- Update `tui_showcase.rs`:
  - Add demos for tool spinners, diff tool result, toasts, hints
  - Fix broken autocomplete demo (was broken by `/` → search activation)
  - Fix broken search demo (use `Ctrl+S` instead of `/`)
  - Update `/cancel` demo flow to work with fixed key bindings

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `command-autocomplete`: `/` key now inserts character and triggers autocomplete instead of search; popup height no longer clipped
- `notification-toast`: showcase demo added for toast notifications
- `command-hints`: showcase demo added for status bar hints
- `tui-tool-rendering`: showcase demo added for tool spinners
- `diff-renderer`: showcase demo added for diff in tool result blocks

## Impact

- **Files modified**: `input.rs` (key handler fix), `layout.rs` (popup height fix), `examples/tui_showcase.rs` (new demos + fix broken ones)
- **No protocol changes** — all data flows through existing types
- **No new dependencies**
- **Risk**: low — key handler change is localized; showcase changes don't affect production code
