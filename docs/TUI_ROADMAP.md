# TUI Roadmap — Restoring Claude Code Visual Fidelity

This document tracks the independent TUI workstream. The goal is pixel-level visual
parity with the original Claude Code (as observed in `docs/cc/`), decoupled from the
main project milestones (M1–M7). TUI code can be written by agents.

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

| Phase | Extension | Reason |
|-------|-----------|--------|
| T2 | `DisplayBlock` richer tool metadata | Tool icons, borders, collapse state |
| T3 | `UiAction` completion request/response | Typeahead |
| T4 | `PendingPermissionView` → typed variants | Per-tool permission dialogs |
| T4 | `UiAction::PermissionResponse` → include "always" | Remember permission decisions |
| T5 | `SessionView` add `context_pct` | Context window meter (dynamic) |
| T6 | `Screen` enum expansion | New screens |

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

**Protocol changes**: `SessionView` needs `context_pct` field (dynamic context window usage). `model`, `effort`, `workspace`, `branch` are already in `TuiState` from startup config — they don't belong in `SessionView` unless runtime model/effort switching is added.

---

### T6: Screens & Navigation — Multi-screen Experience

**Goal**: Multi-screen + modal system.

**Reference mapping**:

| Screen | CC reference | Description |
|--------|-------------|-------------|
| Prompt (main) | `REPL.tsx` | Already implemented |
| Transcript | CC transcript mode | Exists, needs polish |
| Search | `GlobalSearchDialog.tsx` | Full-text search with highlighting |
| Doctor | `Doctor.tsx` (71K) | Diagnostics panel |
| Resume | `ResumeConversation.tsx` | Resume previous session |

**Modal system**:

- `▔` (light horizontal) divider line for bottom-anchored panes
- Semi-transparent overlay behind modals
- Transcript peek: show last 2 messages above modal

---

## Execution Pattern

Each T phase follows this cycle:

```
1. Agent reads current kloud TUI source as reference (docs/cc/ not yet populated)
2. Agent writes Rust/ratatui implementation
3. Construct mock SessionView for render testing (requires test fixture infra)
4. cargo fmt --all && cargo clippy -- -W warnings
5. Visual comparison (cargo run, manual screenshot)
```

**Note**: `docs/cc/` TypeScript references cited throughout are aspirational —
they don't exist in the repo yet. Until they're added, agents should work from
the Rust source and the visual specs in this document.

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
T5 depends on T2 and T3. T6 depends on T5.
