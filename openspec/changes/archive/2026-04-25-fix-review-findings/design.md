## Context

Code review on the `extend-tui-showcase` change found 3 issues. Two are functional risks, one is a UX nit.

## Goals / Non-Goals

**Goals:**
- Allow Ctrl+C exit during permission prompt
- Remove `[toast]` prefix from visible transcript messages
- Guard message count tracking against list shrinkage

**Non-Goals:**
- Refactoring the notification system beyond these fixes
- Adding new features

## Decisions

1. **Ctrl+C in permission mode**: Add explicit `(KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(UiAction::Exit)` before the catch-all `_ => return None` in the permission key handler. This lets users quit even while the prompt is shown.

2. **Toast prefix leak**: Instead of embedding `[toast]` in the system message content, add a `SessionEvent::NotificationRequested { text, level }` variant. The session emits this event, the store records it in a `pending_notifications` field, and the view exposes it. The TUI reads it from the view and clears it. System messages stay clean. **Simpler alternative**: keep the current approach but strip the `[toast]` prefix in `sync_from_active_view` before checking — but the system message still has the prefix in the transcript. Better: change `/notify` to emit both a clean system message AND a separate notification trigger.

   **Simplest fix**: Change the toast detection to look for system messages whose content starts with `[toast]`, create the notification from the stripped text, but also modify the system message content in the store to remove the prefix. This keeps it in one layer.

   **Actually simplest**: Add a `notifications: Vec<NotificationEntry>` to `SessionView`. The store pushes to it when `SystemMessageAdded` fires with `[toast]` prefix. The TUI reads it and clears. No prefix in the actual message content — strip it before storing in `SystemMessageAdded`.

   Final decision: Strip `[toast]` prefix in the `handle_slash_command` for `/notify` — emit two events: one clean `SystemMessageAdded` and emit the toast text separately. Wait, we don't have a toast event. OK, the cleanest minimal fix: In `run.rs`, emit `SystemMessageAdded` without the `[toast]` prefix. Pass the prefix only as a separate field or use a different mechanism. Since adding a new SessionEvent variant is the cleanest long-term fix but heavy for a bugfix, go with: strip the prefix before storing, detect toasts by checking if the original args started with something. Actually, simplest: keep the `[toast]` prefix in the event content, but in `state.rs` toast detection, also strip the prefix from the system message's DisplayBlock::Text after creating the toast. No — that mutates messages which are borrowed.

   **Final decision**: Change `/notify` handler to emit `SystemMessageAdded` with clean content (no `[toast]`), and separately push a notification via a new lightweight mechanism. The simplest: add a `Vec<String>` field to `SessionView` called `pending_toasts` that the store populates. TUI reads and clears it. This avoids any prefix hack.

3. **Message count shrinkage**: Add `self.last_seen_message_count = self.last_seen_message_count.min(new_count)` before the comparison.

## Risks / Trade-offs

- Adding `pending_toasts` to `SessionView` increases the view struct size but keeps the mechanism clean
- The Ctrl+C fix is a one-liner with no risk
