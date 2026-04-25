//! UI abstraction layer.
//!
//! The [`UiBackend`] trait is the single boundary between the application
//! session and any concrete UI implementation (stdio, ratatui, websocket, …).
//! Communication happens exclusively through [`AppEvent`] / [`UiAction`]
//! channels.

use crossterm::event::Event as CrosstermEvent;
use tokio::sync::mpsc;

use super::events::{AppEvent, UiAction};

/// Channel capacity for App → UI events.
const EVENT_CHANNEL_SIZE: usize = 256;
/// Channel capacity for UI → App actions.
const ACTION_CHANNEL_SIZE: usize = 32;

/// The UI-side half of the channel pair (receives events, sends actions).
pub struct UiChannels {
    /// Receive application events (text deltas, errors, shutdown, …).
    pub event_rx: mpsc::Receiver<AppEvent>,
    /// Send user actions back to the session.
    pub action_tx: mpsc::Sender<UiAction>,
    /// Optional receiver for synthetic keyboard events (demo/test only).
    /// When `None`, the TUI event loop ignores this branch.
    pub key_inject_rx: Option<mpsc::Receiver<CrosstermEvent>>,
}

/// The session-side half of the channel pair (sends events, receives actions).
pub struct UiHandle {
    /// Send events to the UI.
    pub event_tx: mpsc::Sender<AppEvent>,
    /// Receive user actions from the UI.
    pub action_rx: mpsc::Receiver<UiAction>,
}

/// Create a matched pair of [`UiChannels`] (for the UI) and [`UiHandle`]
/// (for the session).
pub fn create_ui_channels() -> (UiChannels, UiHandle) {
    let (event_tx, event_rx) = mpsc::channel(EVENT_CHANNEL_SIZE);
    let (action_tx, action_rx) = mpsc::channel(ACTION_CHANNEL_SIZE);

    let channels = UiChannels {
        event_rx,
        action_tx,
        key_inject_rx: None,
    };
    let handle = UiHandle {
        event_tx,
        action_rx,
    };

    (channels, handle)
}

/// A UI backend that owns the terminal / output and runs an event loop.
///
/// Different backends have fundamentally different event loop shapes
/// (ratatui uses `tokio::select!`, stdio uses blocking `read_line`), so the
/// trait has a single `run` method rather than fine-grained render/input hooks.
/// The channel contract **is** the abstraction.
#[async_trait::async_trait]
pub trait UiBackend: Send {
    /// Run the UI event loop until shutdown.
    ///
    /// The implementor should:
    /// 1. Read from `channels.event_rx` and display events to the user.
    /// 2. Capture user input and send it via `channels.action_tx`.
    /// 3. Return when it receives [`AppEvent::Shutdown`] or the event channel
    ///    closes.
    async fn run(self, channels: UiChannels) -> crate::Result<()>;
}
