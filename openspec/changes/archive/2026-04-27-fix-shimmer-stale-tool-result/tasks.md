## 1. Fix Shimmer — Reset StalledState on New Query

- [x] 1.1 Add `reset()` method to `StalledState` in `src/ui/tui/state.rs` — reset `last_response_length` to 0, `last_token_at` to `Instant::now()`, `intensity` to 0.0, `is_stalled` to false
- [x] 1.2 Call `self.stalled_state.reset()` in `apply_view()` when `previous_status != Streaming && self.status == Streaming` (after `sync_from_active_view()` at line ~563)

## 2. Fix Demo Matching — Scope Tool Result Lookup

- [x] 2.1 Rewrite `latest_tool_result()` in `examples/tui_showcase.rs` (line 1367) to only inspect the last user message instead of scanning all messages

## 3. Verification

- [x] 3.1 Run `cargo fmt --all && cargo clippy -- -W warnings` — zero warnings
- [x] 3.2 Run `cargo test` — no regressions
- [x] 3.3 Run `cargo run --example tui_showcase` — verify:
  - Shimmer animates on ALL demos (not just first)
  - Each demo shows correct content (not "Read completed...")
  - Stall demo still turns red after 4.5s silence
  - Tool-using demos dispatch correctly
