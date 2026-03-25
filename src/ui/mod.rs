//! UI abstraction layer and concrete backends.
//!
//! The [`backend::UiBackend`] trait defines the contract between the
//! application session and any UI implementation. Communication flows
//! through [`events::AppEvent`] (session → UI) and [`events::UiAction`]
//! (UI → session) over mpsc channels.

pub mod backend;
pub mod constants;
pub mod events;
pub mod stdio;
pub mod tui;

pub use backend::{UiBackend, UiChannels, UiHandle, create_ui_channels};
pub use events::{AppEvent, UiAction};
