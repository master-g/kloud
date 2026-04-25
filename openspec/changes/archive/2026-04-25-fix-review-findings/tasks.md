## 1. Ctrl+C Escape in Permission Prompt

- [x] 1.1 In `src/ui/tui/input.rs`, add `(KeyCode::Char('c'), KeyModifiers::CONTROL) => return Some(UiAction::Exit)` before `_ => return None` in the permission key handler block

## 2. Toast Prefix Leak Fix

- [x] 2.1 Add `pending_toasts: Vec<PendingToast>` to `SessionView` and `SessionStore`
- [x] 2.2 Add `PendingToast` struct with `text: String` and `level: MessageLevel` fields
- [x] 2.3 In `SessionStore::apply`, when `SystemMessageAdded` fires with `[toast]` prefix: strip the prefix, push the stripped text to `pending_toasts`, store the clean text in the message
- [x] 2.4 In `TuiState::sync_from_active_view`, read `pending_toasts` from the view, create `Notification` structs, clear the source list tracking
- [x] 2.5 Update `/notify` handler in `run.rs` to emit clean content (remove `[toast]` prefix wrapping — the store handles detection)

## 3. Message Count Shrinkage Guard

- [x] 3.1 In `src/ui/tui/state.rs` `sync_from_active_view`, add `self.last_seen_message_count = self.last_seen_message_count.min(new_count)` before the toast detection comparison

## 4. Build Verification

- [x] 4.1 `cargo check`
- [x] 4.2 `cargo clippy -- -W warnings`
- [x] 4.3 `cargo test`
