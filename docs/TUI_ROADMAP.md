# TUI Roadmap — Restoring Claude Code Visual Fidelity

This document tracks the independent TUI workstream. The goal is pixel-level visual
parity with the original Claude Code (as observed in `docs/cc/`), decoupled from the
main project milestones (M1–M7). TUI code can be written by agents.

> **Audited 2026-04-26** — Code verification found several inaccuracies. Changes
> marked with `[AUDIT]` throughout. See "Audit Findings" section at the end.

---

## Guiding Principle

> The TUI module is a visual replica of Claude Code. It evolves independently, driven
> by visual fidelity, not by backend readiness. The `UiBackend` trait / `AppEvent`
> protocol (`src/ui/events.rs`) is the stable boundary.

---

## Current State

### What Already Matches CC

- Brand color `#D77757` (Claude orange)
- 12-frame bounce spinner animation
- Glimmer / shimmer effect on activity text
- Dark + light theme system (6 color schemes)
- Virtual scrolling for message lists
- Vim mode in text input
- Activity line with live status + spinner
- Comprehensive markdown (headings, emphasis, code blocks, lists, blockquotes, links, rules)
- Tool status icons (color-coded: success/error/subtle) + blink animation
- Tool result collapse (>5 lines shows "Tab to expand")
- Diff rendering with syntax coloring + line numbers (dead code, not wired)
- Stall detection with color interpolation
- Token counter animation
- Permission prompt (inline in message area)
- Search mode
- Autocomplete state + rendering (dead code, not wired)

### What's Missing

| Area | CC Reference | kloud Status |
|------|-------------|--------------|
| Message rendering detail | `MessageRow.tsx` (47K) | Basic — no collapse, selection, timestamps |
| Tool block borders | Multiple `*ToolUse*.tsx` files | No border containers, no purple tool names |
| Diff rendering (wired) | `FileEditToolDiff.tsx`, `StructuredDiff.tsx` | Implemented but dead code — needs wiring |
| Thinking blocks | CC shimmer blocks | Shimmer exists, no collapse |
| Markdown tables | `MarkdownTable.tsx` (46K) | Not implemented |
| Syntax highlighting | `HighlightedCode.tsx` | Not implemented |
| Command typeahead | `useTypeahead.tsx` (207K) | Partially implemented (needs wiring) |
| History search | `HistorySearchDialog.tsx` | Not implemented |
| File path completion | `useTypeahead.tsx` `@` trigger | Not implemented |
| Paste handling | `usePasteHandler.ts` | Not implemented |
| Permission dialogs | `components/permissions/` (multi-type) | Duplicate: widget dead code + inline in messages |
| Stats panel | `Stats.tsx` (149K) | Basic status bar |
| Status line | `StatusLine.tsx` (48K) | Simplified version |
| Global search | `GlobalSearchDialog.tsx` | Not implemented |
| Doctor screen | `Doctor.tsx` (71K) | Not implemented |
| Modal system | CC `▔` bottom-anchored panes | Not implemented |
| Task list | `TaskListV2.tsx` (49K) | Not implemented |
| Agent status | `CoordinatorAgentStatus.tsx` | Not implemented |
| Scroll position management | `ScrollBox` (ink/) | VirtualScroll tracks scroll state + sticky scroll. Redundant `scroll: u16` field needs consolidation |
| NewMessagesPill | `FullscreenLayout.tsx:371-378` | Not implemented — shows "N new messages" when scrolled up |
| StickyPromptHeader | `FullscreenLayout.tsx:339-344` | Not implemented — pins current input text at top when scrolled down |

---

## Protocol Boundary

### Current Protocol (`src/ui/events.rs`)

```
Session ──AppEvent::View(SessionView)──→ TUI
TUI ──UiAction──→ Session
```

### Planned Protocol Extensions

| Phase | Extension | Reason | Status |
|-------|-----------|--------|--------|
| T2 | `DisplayBlock` richer tool metadata | Tool icons, borders, collapse state | Partial — `server_name` exists, may need category icon |
| T3 | `UiAction` completion request/response | Typeahead | Partial — search protocol exists, autocomplete wiring needed |
| T4 | `PendingPermissionView` → typed variants | Per-tool permission dialogs | Pending |
| T4 | `UiAction::PermissionResponse` → include "always" | Remember permission decisions | Pending — only `{ allowed: bool }` exists |
| ~~T5~~ | ~~`SessionView` add `context_pct`~~ | ~~Context window meter~~ | **Already implemented** in `layout.rs:274` |
| ~~T6~~ | ~~`Screen` enum expansion~~ | ~~New screens~~ | **Not needed** — CC has 2 screens, rest are overlays |

---

## Seven Phases

```
T0 ─── T1 ─── T2 ─── T3 ─── T4 ─── T5 ─── T6
Wire   Refactor Msgs   Input  Perms   Stats  Screens
```

---

### T0: Dead Code Wiring — ✅ COMPLETED (commit `6a1e28f`)

All 6 dead-code widgets wired into render path. `#[allow(dead_code)]` annotations removed.
`cargo clippy` clean. Diff colors, permission widget, `/` autocomplete, toasts, hints,
tool spinners all active.

---

### T1: Module Refactor — Component Architecture

**Goal**: Split monolithic `TuiState` (995 lines) into focused sub-modules,
extend layout architecture toward CC's 4-layer model.

> **Audited 2026-04-28** — Code verification found several inaccuracies in
> original T1 plan. Changes marked with `[T1-AUDIT]` below.

**Layout Architecture Upgrade** [T1-AUDIT: revised]:

CC's `FullscreenLayout` (docs/cc/components/FullscreenLayout.tsx) uses a
4-layer content model:

1. **scrollable** (`flexGrow=1`, `overflow=hidden`) — ScrollBox with sticky scroll
2. **bottom** (`flexShrink=0`, `maxHeight=50%`) — input, task list, immediate commands
3. **overlay** (inside ScrollBox) — permission requests, floating status
4. **modal** (absolute, bottom-anchored with `▔` divider) — slash command dialogs

kloud currently uses **dynamic** `Vec<Constraint>` (conditionally adds title bar,
notification row — not purely "fixed" rows). T1 must extend this:

- **VirtualScroll upgrade** (`virtual_scroll.rs`): [T1-AUDIT] VirtualScroll already
  implements sticky scroll (`auto_scroll` flag, disabled on scroll up, re-enabled
  on scroll to bottom). It tracks `scroll_offset` and provides
  `scroll_up/down/to_bottom/to_top()`. **Do NOT replace with new ScrollArea** —
  extend VirtualScroll with any missing CC features instead.
- **Eliminate `scroll: u16` field** [T1-AUDIT]: `TuiState` has redundant scroll
  tracking — `scroll: u16` (simple offset, used by `input.rs`) and
  `VirtualScroll.scroll_offset` do the same thing. They are not synchronized.
  T1 must remove `scroll: u16` and route all scroll operations through
  VirtualScroll.
- **Dynamic bottom height**: `Constraint::Length(3)` at `widgets/mod.rs:48`
  needs to become variable. TextArea already exposes `line_count()`. Change is
  trivial: `Constraint::Length(text_area.line_count().clamp(3, height/2))`.
- **Overlay render pass**: Autocomplete popup (`layout.rs:166-212`) already uses
  overlay pattern — renders above input area via absolute positioning. Extend
  this pattern for permissions (T4) and modals (T6). No new infrastructure
  needed, just consistent use of existing pattern.

**Target Structure** [T1-AUDIT: revised — builds on existing `widgets/`, no parallel `render/`]:

```
src/ui/tui/
├── mod.rs              # RatatuiBackend (event loop unchanged)
├── constants.rs        # Move from src/ui/constants.rs (FPS, spinner verbs, timing)
├── state/
│   ├── mod.rs          # TuiState facade (thin layer)
│   ├── app_state.rs    # Messages / screen / permission / notification / collapsed tools
│   ├── input_state.rs  # Input / history / completion / autocomplete
│   ├── scroll_state.rs # VirtualScroll wrapper (eliminates redundant scroll: u16)
│   ├── anim_state.rs   # ActivityClock / StalledState / TokenCounter / ThinkingStatus
│   └── theme_state.rs  # Theme switching
├── widgets/            # Renamed from current widgets/ (10 files already here)
│   ├── mod.rs          # Widget registry + main render()
│   ├── layout.rs       # Fullscreen layout container
│   ├── title_bar.rs    # Top title bar
│   ├── messages.rs     # Message list (uses VirtualScroll) [→ message_list.rs in T2]
│   ├── tool_block.rs   # Tool use / result blocks
│   ├── diff.rs         # File edit diff rendering
│   ├── thinking.rs     # NEW in T2 — thinking block + glimmer shimmer
│   ├── input_bar.rs    # NEW — extracted from layout.rs input rendering
│   ├── status_bar.rs   # Bottom status bar
│   ├── activity_line.rs # Activity line + spinner + tool spinners
│   ├── permission.rs   # Permission dialogs
│   ├── glimmer.rs      # Glimmer/shimmer text effect (already exists)
│   ├── helpers.rs      # Color blending / interpolation / text utils (already exists)
│   └── spinner_glyph.rs # Spinner animation data (already exists)
├── theme.rs            # Theme struct (unchanged)
├── text_area.rs        # TextArea (unchanged)
├── virtual_scroll.rs   # VirtualScroll (extended, NOT replaced)
└── input.rs            # Key mapping (expanded in T3)
```

**What changed from original plan** [T1-AUDIT]:
- Kept `widgets/` name (already 10 files) instead of renaming to `components/`
- Dropped `render/` directory — `glimmer.rs`, `helpers.rs` already in `widgets/`
- `scroll_state.rs` wraps VirtualScroll instead of creating new ScrollArea
- `constants.rs` relocated from `src/ui/constants.rs` (one level up)

**Acceptance**: All existing tests pass. `cargo fmt --all && cargo clippy -- -W warnings` clean.
`scroll: u16` field removed, all scroll ops through VirtualScroll.

**Note** [T1-AUDIT: revised]: Animation types (`ActivityClock`, `StalledState`,
`TokenCounter`, `ThinkingStatus`) are self-contained structs in `state.rs:28-231`
with their own fields and methods. `tick()` in `mod.rs:157-168` is 12 lines that
delegates to field methods. Extraction to `anim_state.rs` is straightforward
file relocation + `use` imports — no need for `Tickable` trait or complex
ownership refactoring.

**Additional state types to relocate** [T1-AUDIT — not in original plan]:
- `SearchState`, `AutocompleteState` → `input_state.rs`
- `Notification` → `app_state.rs`
- `ActivityEntry`, `LiveActivity`, `ActivitySnapshot` → `anim_state.rs`
- `CollapsedTools` (`BTreeMap<usize, bool>`) → `app_state.rs`

**Screen enum** [T1-AUDIT]: `Screen` has 3 variants (`Prompt`, `Transcript`,
`Search`), defined in `src/agent/view.rs:21-25`. CC has only 2 (`prompt |
`transcript`). Decide: demote `Search` to a sub-state of `Prompt`, or keep
3 variants. Toast rendering uses dedicated `Constraint::Length(1)` row, not
overlay — confirm this aligns with T6 overlay plans.

---

### T2: Message Rendering — Visual Core

**Goal**: Message area visual fidelity at ~90% of CC.

**Reference mapping**:

| kloud component | CC reference | Key visual elements |
|-----------------|-------------|-------------------|
| `message_row.rs` | `MessageRow.tsx` | User: `❯` prefix, gray bg. Assistant: `●` prefix, brand accent. Timestamps. |
| `tool_block.rs` | `FallbackToolUseErrorMessage.tsx` et al. | Rounded borders, tool name in purple, status icons `◆ ◇ ⟳ ✗`, collapsible output |
| `diff_view.rs` | `FileEditToolDiff.tsx` | Syntax-colored diff, line numbers, context lines |
| `thinking.rs` | CC thinking blocks | Shimmer text effect, collapsible |
| `markdown.rs` | `Markdown.tsx` + `MarkdownTable.tsx` | Tables, syntax highlighting, links |

**Visual specs**:

- User messages: `❯` prefix, subtle gray background
- Assistant messages: `●` prefix, brand color heading
- Tool blocks: bordered container, tool name in purple (`#AF87FF`), status icons
  - `◆` done, `◇` pending, `⟳` running, `✗` errored
- Bash tool output: pink border (bashBorder: bright pink)
- Diff view: red/green syntax coloring, line numbers
- Thinking: shimmer text animation, collapsible after expansion
- Markdown tables: aligned columns with borders
- Code blocks: optional syntax highlighting (syntect or tree-sitter)

**Protocol changes**: `DisplayBlock` may need richer tool metadata (display name, category icon).

---

### T3: Input System — Interaction Core

**Goal**: Input experience close to CC.

**Reference mapping**:

| Feature | CC reference | kloud status | Priority |
|---------|-------------|-------------|----------|
| Multi-line input | `BaseTextInput.tsx` | Done (TextArea) | — |
| Vim mode | `VimTextInput.tsx` | Done | — |
| Command typeahead | `useTypeahead.tsx` (207K) | Partially implemented (needs wiring) | P1 |
| History search | `HistorySearchDialog.tsx` | Not started | P2 |
| File path completion | `useTypeahead.tsx` `@` trigger | Not started | P3 |
| Paste handling | `usePasteHandler.ts` | Not started | P3 |
| Voice input | `useVoice.ts` | Skip for now | P5 |

**Implementation order**:

1. `/` command typeahead popup below input
2. `Ctrl+R` history search dialog
3. `@` file path suggestion popup
4. Multi-line paste detection (auto-expand)

**Protocol changes**: New `UiAction` variants for completion requests.

---

### T4: Permission System — Security Interaction

**Goal**: Permission dialogs visually and functionally match CC.

**Reference**: `docs/cc/components/permissions/`

**Visual spec**:

```
┌─────────────────────────────────────┐
│ ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔ │
│                                     │
│  ⚠ Allow Bash command?              │
│                                     │
│  $ rm -rf /tmp/test                 │
│                                     │
│  ❯ Yes   ○ Yes always   ○ No       │
│                                     │
└─────────────────────────────────────┘
```

**Permission types**:

- Bash command execution → show command preview
- File write / edit → show file path + diff preview
- MCP tool call → show server name + tool name
- Sub-agent permission → show agent name + action

**Protocol changes**:

- `PendingPermissionView` → typed variants per tool category
- `UiAction::PermissionResponse` → add "always allow" option

**Rendering Strategy** (resolves design gap from `t0-proposal-audit.md`):

CC renders permission requests as `overlay` inside the ScrollBox — the user
can scroll up to see message history while the permission UI stays fixed at
the viewport bottom (`FullscreenLayout.tsx:275`, `overlay` prop).

kloud should follow the same pattern:
- Permission request rendered as overlay in the messages area
- NOT inline in the messages list (current approach)
- Requires `ScrollArea` from T1 to manage scroll position
- User can scroll up past the overlay to review conversation context

---

### T5: Status Panel — Information Density

**Goal**: StatusLine and Stats panel match CC.

**Reference**: `StatusLine.tsx` (48K) + `Stats.tsx` (149K)

**Visual spec**:

```
┌─────────────────────────────────────────────────────┐
│ [Sonnet 4.6] [◐ Med] [main] ~/project  1.2k↑ 856↓  │
│ ▓▓▓▓▓▓▓▓░░ 72% ctx  $0.42  3:24  ◆ Read(3)        │
└─────────────────────────────────────────────────────┘
```

**Elements**:

- Model badge (e.g., `Sonnet 4.6`)
- Effort level indicator: `○` low, `◐` medium, `●` high, `◉` max
- Git branch name
- Workspace path
- Token counts (input ↑ / output ↓) with smooth animation
- Context window meter (progress bar)
- Session cost ($)
- Session duration
- Active tool list with count
- Keybinding hints (progressive, width-gated)

**Protocol changes**: ~~`SessionView` needs `context_pct` field~~ [AUDIT: already implemented in
`layout.rs:274` via `input_tokens / max_context_tokens`]. `model`, `effort`, `workspace`,
`branch` are already in `TuiState` from startup config — they don't belong in `SessionView`
unless runtime model/effort switching is added.

---

### T6: Screens & Overlays — Layered UI [AUDIT: scope reduced]

**Goal**: Overlay system + independent CLI commands. **Not** a 7-screen navigation
system — CC only has 2 true screens (`prompt` + `transcript`), everything else is
overlays or separate commands.

**CC actual architecture** (`screens/REPL.tsx:571`):

```typescript
export type Screen = 'prompt' | 'transcript';
// Only 2 screens. Doctor = separate CLI command.
// GlobalSearch = modal overlay. Resume = startup flow.
```

CC uses early-return pattern:

```
if (screen === 'transcript') return <TranscriptLayout />;
// else prompt layout continues
return <PromptLayout />;
```

**Revised scope**:

| Component | CC Pattern | kloud Implementation |
|-----------|-----------|---------------------|
| Prompt screen | `REPL.tsx` main path | Already implemented |
| Transcript screen | `REPL.tsx` early return | Exists, needs polish |
| Search | Overlay on Messages | Already implemented |
| GlobalSearch | Modal overlay (`GlobalSearchDialog.tsx`) | Overlay system needed |
| Doctor | **Independent CLI subcommand** (`claude doctor`) | CLI subcommand, not Screen variant |
| Resume | **Startup flow** (`ResumeConversation.tsx`) | Pre-REPL selection, not Screen variant |

**Overlay system** (replaces "modal system"):

- `▔` (light horizontal) divider line for bottom-anchored panes
- Semi-transparent overlay behind modals
- Transcript peek: show last 2 messages above modal

**Overlay Infrastructure** — three new components:

1. **`overlay.rs`** — renders within the messages area (permissions, status)
   - Uses ratatui's overlapping render (same `Rect`, render after messages)
   - Semi-transparent background effect via styled `Block`
   - Content stays fixed at viewport bottom while user scrolls

2. **`modal.rs`** — bottom-anchored modal with `▔` divider
   - Absolute-positioned over the entire frame
   - `▔` divider line at top (full terminal width, brand color)
   - Modal content with `paddingX=2`
   - `maxHeight = terminal_rows - 2` (peek 2 transcript messages above)
   - Used for: slash command dialogs (`/config`, `/theme`, `/model`)

3. **`pill.rs`** — "N new messages" / "Jump to bottom" pill
   - Absolute-positioned at bottom of messages area
   - Shows when user scrolled up and new content arrives
   - Subscribes to `ScrollArea` scroll state via `useSyncExternalStore`
   - Click/Enter scrolls back to bottom (repins sticky scroll)

Implementation: ratatui allows rendering multiple widgets into the same
`Rect`. No layout `Constraint` changes needed — overlay/modal are rendered
AFTER the main layout split, using the same or overlapping areas.

**Why not 7 screens**: CC's Doctor runs as a separate process (`claude doctor`), not
a REPL screen. GlobalSearch is a modal dialog rendered on top of the current screen.
Resume is a pre-session flow. Keeping `Screen` enum to 2 variants matches CC exactly
and avoids the complexity of per-screen layout dispatch.

---

## Execution Pattern

Each T phase follows this cycle:

```
1. Agent reads current kloud TUI source + CC reference in docs/cc/
2. Agent writes Rust/ratatui implementation
3. Run widget unit tests or showcase.rs for visual verification
4. cargo fmt --all && cargo clippy -- -W warnings
5. Visual comparison (cargo run --example tui_showcase, manual screenshot)
```

**Testing strategy** [AUDIT: added]:

```
┌─────────────────────────────────────────────────┐
│  E2E: tui_showcase.rs with key_inject_rx        │
│  + automatic screenshot comparison              │
├─────────────────────────────────────────────────┤
│  Integration: key_inject + state assertions     │
│  (key_inject_rx already built, needs test harness)
├─────────────────────────────────────────────────┤
│  Unit: widget render → Vec<Line> snapshot       │
│  (ratatui Line is pure data, no terminal needed)│
└─────────────────────────────────────────────────┘
```

`key_inject_rx` in `UiChannels` enables synthetic keyboard events without a real
terminal. Currently only used by `tui_showcase.rs`. Widget rendering functions
return `Vec<Line<'static>>` — suitable for snapshot testing with no I/O.

## Dependency Graph

```
T0 (wire) ──✅ done
T1 (refactor + VirtualScroll extension + dynamic bottom) ──→ T2 (messages)
                                    │                          │
                                    │                          ├──→ T4 (permissions → overlay)
                                    │                          │
                                    └──→ T3 (input) ──────────┼──→ T5 (stats)
                                                               │
                                                               └──→ T6 (overlay + modal + pill)
```

T1 must complete before T2–T6 — it reorganizes module structure and extends
VirtualScroll that all subsequent phases depend on.
T2 and T3 can proceed in parallel after T1. T4 depends on T2 (message
rendering) for overlay pattern. T5 depends on T2 and T3. T6 depends on
T5 for status bar overlay.

---

## Audit Findings (2026-04-26)

### Protocol Boundary — More Capable Than Documented

Roadmap listed 6 planned protocol extensions. Several already exist:

| Roadmap Claim | Actual Status |
|---------------|---------------|
| T2: richer `DisplayBlock` metadata | `DisplayBlock::ToolUse` already carries `server_name`, `input_json`, `input_preview` |
| T5: `SessionView` needs `context_pct` | `layout.rs:274` already computes context % from `input_tokens / max_context_tokens` |
| T3: completion request/response | `UiAction` already has full search protocol: `SearchActivate/Submit/Next/Prev/Exit` |
| T4: `PermissionResponse` needs "always" | Correct — only `{ allowed: bool }` exists |

`UiAction` has 13 variants (not just `SendMessage`/`CancelTurn`). Full protocol is
in `src/ui/events.rs`. The protocol extension table understates current capability.

### T1 Refactor Risk — `tick()` Coupling [T1-AUDIT: revised]

`TuiState::tick()` updates 5 animation subsystems per frame. But `tick()` is
only 12 lines (`mod.rs:157-168`) that delegates to field methods on self-contained
structs. The 4 animation types (`ActivityClock`, `StalledState`, `TokenCounter`,
`ThinkingStatus`) live in `state.rs:28-231` with their own fields and methods.

**Original concern**: "hardest part of T1" requiring `Tickable` trait or complex
ownership refactoring.

**Actual**: Straightforward file relocation. Move struct definitions to
`state/anim_state.rs`, add `use` imports. No trait needed. The module split
(files → directories) and the tick extraction are equally easy.

### DisplayBlock Dead Fields

`DisplayBlock::ToolUse` carries `input: serde_json::Value` and `input_json: String`
that are **never accessed by any UI code**. Only `input_preview` (pre-computed,
truncated to 240 chars at creation time) is used in rendering. These fields belong
to the tool execution engine (M2), not the display type. Consider moving them when
M2's `ToolCall` type is established.

### `show_title_bar` Architecture

Currently hardcoded to `false` in `main.rs`. Not driven by `Screen` enum. Since
CC only has 2 screens and `Screen` already drives layout differences (transcript
omits input area), the simplest fix is:

```rust
impl Screen {
    fn show_title_bar(&self) -> bool {
        match self { Screen::Prompt => false, Screen::Transcript => true }
    }
}
```

No need for per-screen render functions or ScreenLayout structs — the 2-screen
enum keeps things simple, matching CC exactly.

### T6 Scope Reduction

Original T6 planned 7 Screen variants. CC source reveals only 2 (`prompt` |
`transcript`). Doctor is a CLI subcommand, GlobalSearch is a modal overlay, Resume
is a pre-session flow. Revised T6 focuses on overlay infrastructure rather than
screen navigation.

### Layout Architecture Gap (2026-04-28) [T1-AUDIT: revised]

CC uses `FullscreenLayout` with 4 content slots: `scrollable`, `bottom`,
`overlay`, `modal`. kloud uses **dynamic** `Vec<Constraint>` (conditionally
adds title bar, notification row). The gap is:

1. **Scrollable**: VirtualScroll already tracks `scroll_offset`, has `auto_scroll`
   sticky behavior. Not missing — extend, don't replace. [T1-AUDIT: VirtualScroll
   has sticky scroll, original claim was wrong]
2. **Redundant scroll field**: `TuiState.scroll: u16` duplicates
   `VirtualScroll.scroll_offset`. Not synchronized. Must consolidate.
3. **No overlay slot**: Permissions render inline in messages. Autocomplete popup
   (`layout.rs:166-212`) already uses overlay pattern (absolute positioning above
   input). Extend this pattern, don't build new infrastructure.
4. **No modal layer**: Slash command dialogs have no bottom-anchored modal
   with `▔` divider
5. **Fixed bottom height**: Input always 3 lines; CC allows up to 50%
   terminal height. TextArea already has `line_count()` method. Trivial fix:
   `Constraint::Length(text_area.line_count().clamp(3, height/2))`
6. **Missing pill + sticky header**: NewMessagesPill and StickyPromptHeader
   not implemented
7. **Screen::Search variant**: kloud has 3 Screen variants (`Prompt`,
   `Transcript`, `Search`), CC has 2. Decide alignment.
