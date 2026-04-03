//! App ↔ UI communication protocol.
//!
//! The session publishes a full [`SessionView`](crate::agent::view::SessionView)
//! snapshot whenever transcript or query state changes. UI backends keep only
//! local input/animation state and render from the latest snapshot.

use crate::agent::view::{Screen, SessionView};

/// Events sent **from** the application session **to** the UI.
#[derive(Debug, Clone)]
pub enum AppEvent {
	/// Replace the current UI snapshot with the latest session view.
	View(Box<SessionView>),
	/// The session is shutting down; the UI should exit its event loop.
	Shutdown,
}

/// Actions sent **from** the UI **to** the application session.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum UiAction {
	SendMessage(String),
	CancelTurn,
	Exit,
	SlashCommand {
		command: String,
		args: String,
	},
	SetScreen(Screen),
	SetTranscriptShowAll(bool),
}
