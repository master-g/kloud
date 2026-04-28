## Context

T1 module refactor split `state.rs` into `state/` sub-modules. Post-merge code review found a direction bug in scroll, dead code from partial accessor migration, and a visibility mismatch. All fixes are localized to `src/ui/tui/state/`.

## Goals / Non-Goals

**Goals:**
- Fix scroll direction bug before T2 builds on it
- Remove dead code introduced during refactor migration
- Correct visibility annotations to match actual usage
- Shore up edge-case safety

**Non-Goals:**
- No refactoring beyond what the audit flagged
- No new features or behavior changes
- No changes to widget or input handler layers

## Decisions

1. **Delete `theme_state.rs`** rather than stub it — theme is managed via `Theme` struct passed as parameter, not state. No `TuiState` field references it. Re-add when T5 implements theme switching.

2. **Remove unused accessors** (`messages()`, `screen()`, etc.) — audit shows all callers use `state.app.*` directly. The facade pattern was abandoned during refactor.

3. **Reuse first `Instant::now()`** in `sync_from_active_view` — the two calls are ~100 lines apart with no intervening I/O, so temporal drift is negligible. Single timestamp is more correct for "this sync cycle."

## Risks / Trade-offs

- Removing `theme_state.rs` and accessors is unrevertable if downstream code expected them — but `cargo check` + `cargo test` pass without them. → Verify after deletion.
