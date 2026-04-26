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

### T0: Dead Code Wiring — Activate Existing Widgets

**Goal**: Eliminate all `#[allow(dead_code)]` annotations by wiring already-implemented
widgets into the render path. No new rendering code — just connect what exists.

**Dead code inventory** (≈200 lines already written):

| Widget | Location | Lines | What to do |
|--------|----------|-------|------------|
| Diff renderer | `widgets/diff.rs` | 51 | Wire into tool result blocks when output is a diff |
| Permission dialog | `widgets/permission.rs` | 43 | Replace inline permission rendering in `messages.rs:87-104` |
| Autocomplete popup | `layout.rs::render_autocomplete` | ~30 | Wire to `/` key in `input.rs`, use existing `AutocompleteState` |
| Toast notifications | `layout.rs::render_toasts` | ~20 | Wire to `pending_toasts` from `SessionView` |
| Keybinding hints | `layout.rs::render_hints` | ~20 | Show in status bar when width permits |
| Tool spinners | `activity_line.rs::render_tool_spinners` | ~30 | Wire to `active_tools` during tool execution |

**Acceptance**: `#[allow(dead_code)]` removed from all 6 functions. `cargo clippy` clean.
Visual: diff colors appear in tool results, permission uses bordered widget, `/` triggers
autocomplete popup.

**Estimated effort**: ~half day.

---

### T1: Module Refactor — Component Architecture

**Goal**: Split monolithic `TuiState` (980 lines) into focused sub-modules for
maintainability. Functionality unchanged after refactor.

**Target Structure**:

```
src/ui/tui/
├── mod.rs              # RatatuiBackend (event loop unchanged)
├── constants.rs        # FPS, spinner verbs, color values, timing thresholds
├── state/
│   ├── mod.rs          # TuiState facade (thin layer)
│   ├── app_state.rs    # Messages / screen / permission state
│   ├── input_state.rs  # Input / history / completion / autocomplete
│   ├── scroll_state.rs # Virtual scroll / search
│   ├── anim_state.rs   # Spinner / glimmer / stall / token animations
│   └── theme_state.rs  # Theme switching
├── components/
│   ├── mod.rs
│   ├── layout.rs       # Fullscreen layout container
│   ├── title_bar.rs    # Top title bar
│   ├── message_list.rs # Message list (virtual scroll)
│   ├── message_row.rs  # Single message rendering
│   ├── tool_block.rs   # Tool use / result blocks
│   ├── diff_view.rs    # File edit diff
│   ├── thinking.rs     # Thinking block + glimmer shimmer
│   ├── input_bar.rs    # Input area
│   ├── status_bar.rs   # Bottom status bar
│   ├── activity_line.rs # Activity line + spinner + tool spinners
│   ├── permission.rs   # Permission dialogs
│   ├── toast.rs        # Notification toasts
│   └── search.rs       # Search bar
├── render/
│   ├── mod.rs
│   ├── markdown.rs     # pulldown-cmark rendering
│   ├── text.rs         # Text measurement / wrapping
│   ├── color.rs        # Color blending / interpolation (from helpers.rs)
│   └── glimmer.rs      # Glimmer/shimmer text effect
├── theme.rs            # Theme struct (unchanged)
├── text_area.rs        # TextArea (unchanged)
├── virtual_scroll.rs   # VirtualScroll (unchanged)
└── input.rs            # Key mapping (expanded in T3)
```

**Acceptance**: All existing tests pass. `cargo fmt --all && cargo clippy -- -W warnings` clean.

**Note**: Animation types (`ActivityClock`, `StalledState`, `TokenCounter`, `ThinkingStatus`) are tightly coupled to `TuiState` fields via tick methods. Splitting into `anim_state.rs` requires designing a clean trait interface to avoid circular dependencies or excessive `pub` fields.

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

**Why not 7 screens**: CC's Doctor runs as a separate process (`claude doctor`), not
a REPL screen. GlobalSearch is a modal dialog rendered on top of the current screen.
Resume is a pre-session flow. Keeping `Screen` enum to 2 variants matches CC exactly
and avoids the complexity of per-screen layout dispatch.

---

## Execution Pattern

Each T phase follows this cycle:

```
1. Agent reads current kloud TUI source + CC reference in docs/cc/ (1909 files)
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
T0 (wire) ──→ T1 (refactor) ──→ T2 (messages) ──→ T3 (input)
                                 │                  │
                                 ├──────────────────┼──→ T4 (permissions)
                                 │                  │
                                 └──────────────────┴──→ T5 (stats)
                                                        │
                                                        └──→ T6 (screens)
```

T0 must complete first (eliminates dead code before refactor moves files).
T1 must complete before T2–T6. T2 and T3 share `layout.rs` and message/input
rendering paths — prefer sequential (T2 then T3) to avoid merge conflicts.
T4 depends on T2 (message rendering) and T3 (input for permission responses).
T5 depends on T2 and T3. T6 depends on T5 for status bar overlay.

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

### T1 Refactor Risk — `tick()` Coupling

`TuiState::tick()` updates 5 animation subsystems per frame (spinner, glimmer,
stall, token counter, thinking). Splitting `TuiState` into sub-modules requires
either:
- A `Tickable` trait with `fn tick(&mut self, dt: Duration)`
- Or `&mut` references to multiple sub-states in `tick()`

This is the hardest part of T1. The module split (files → directories) is easy;
the ownership refactoring around `tick()` is the real challenge.

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
