## 1. Virtual Scroll Fixes

- [x] 1.1 Call `ensure_cache` before `visible_message_range` in `messages.rs` render path
- [x] 1.2 Fix `msg_idx` to add virtual scroll start offset for collapse state lookups
- [x] 1.3 Fix `scroll_down` in `virtual_scroll.rs` — pass `visible_lines` param instead of hardcoding 0

## 2. Tool Rendering Fixes

- [x] 2.1 Move batch tool header rendering above block loop, print once per message with `batch_header_printed` flag
- [x] 2.2 Auto-populate collapse map on first Tab press — scan for nearest tool result >5 lines

## 3. Screen & Scroll State Fixes

- [x] 3.1 Preserve `Screen::Search` in `sync_from_active_view` — skip screen overwrite when searching
- [x] 3.2 Fix PageDown to only re-enable auto-scroll when actually at bottom

## 4. Theme & Cleanup

- [x] 4.1 Fix daltonized warning/success colors — increase luminance delta to >30
- [x] 4.2 Add `#[derive(Default)]` to `SearchState`
- [x] 4.3 Remove unnecessary `clone()` in search execute hot path

## 5. Verification

- [x] 5.1 Run `cargo check` — zero errors
- [x] 5.2 Run `cargo test` — all tests pass
- [x] 5.3 Run `cargo clippy -- -W warnings` — no new warnings
