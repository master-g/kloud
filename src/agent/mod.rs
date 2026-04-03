//! Agent-facing message, event, and store types.
//!
//! This layer sits between transport-level Anthropic events and the UI. It
//! owns the durable transcript, in-flight assistant state, and the view model
//! consumed by UI backends.

pub mod message;
pub mod session_event;
pub mod store;
pub mod view;

pub use message::{
	ActivityAccent, ActivityEntry, ActivityEntryKind, DisplayBlock, DisplayMessage, LiveActivity,
	MessageId, MessageLevel, MessageLookups, MessageType, NormalizedMessage, SpinnerMode,
	ToolStatus, TranscriptMessage, build_message_lookups, group_messages_for_display,
	normalize_messages, reorder_messages_in_ui,
};
pub use session_event::SessionEvent;
pub use store::SessionStore;
pub use view::{AssistantStatus, PendingPermissionView, Screen, SessionView};
