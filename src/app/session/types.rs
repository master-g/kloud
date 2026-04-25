use std::collections::HashMap;
use std::pin::Pin;

use futures::Stream;

use crate::llm::response::{StopReason, StreamEvent};
use crate::llm::types::{CacheControl, ContentBlock};
use crate::tools::ToolCall;

pub(super) type AssistantStream =
    Pin<Box<dyn Stream<Item = Result<StreamEvent, crate::error::LlmError>> + Send>>;

/// Represents a pending tool use request that the model has requested.
pub(super) struct PendingToolUse {
    /// Unique identifier for this pending tool use request.
    pub(super) id: String,
    /// Name of the tool being requested.
    pub(super) name: String,
    /// The input JSON that was passed to the model for this tool use request.
    pub(super) start_input: serde_json::Value,
    /// The current input JSON for this tool use request.
    pub(super) input_json: String,
    /// Optional cache control directives for this tool use request.
    pub(super) cache_control: Option<CacheControl>,
}

/// Represents a content block that is currently being streamed from the model
/// but has not yet completed.
pub(super) enum PendingBlock {
    /// A text block that is being streamed from the model but has not yet completed.
    Text {
        text: String,
        cache_control: Option<CacheControl>,
    },
    /// A thinking block that is being streamed from the model but has not yet completed.
    Thinking {
        thinking: String,
        signature: Option<String>,
    },
    /// A tool use block that is being streamed from the model but has not yet completed.
    ToolUse(PendingToolUse),
}

/// Represents a pending tool dispatch that the model has requested but we have
/// not yet sent to the tool registry.
pub(super) struct PendingDispatch {
    /// Unique identifier for this pending tool dispatch, which matches the tool use ID from the model.
    pub(super) tool_use_id: String,
    /// The tool call to dispatch.
    pub(super) call: ToolCall,
}

pub(super) enum StreamingActionOutcome {
    Next(TurnState),
    Exit,
}

/// Short-lived state for the *current* assistant turn only.
///
/// This is the "inner loop" runtime state:
/// - `Idle` means no model stream is active, so the session is waiting for the
///   next user action
/// - `Streaming` means one assistant turn is in progress and we are consuming
///   SSE events from the model
///
/// Keeping this separate from `Session` makes the control flow easier to
/// evolve into the later agent/tool loop, where one user turn may contain
/// multiple model -> tool -> model steps.
///
/// Current high-level flow:
///
/// ```text
/// +------+      +------------------+      +------------------------+
/// | UI   | ---> | Session::run()   | ---> | TurnState::Idle        |
/// +------+      +------------------+      +------------------------+
///                      |                           |
///                      | UiAction::SendMessage     |
///                      v                           |
///               +-------------------------+        |
///               | start_streaming_turn()  |        |
///               +-------------------------+        |
///                      |                           |
///                      v                           |
///               +-------------------------+        |
///               | TurnState::Streaming    | -------+
///               | - stream                |
///               | - stop_reason           |
///               | - completed blocks      |
///               | - pending blocks        |
///               +-------------------------+
///                      |
///                      | StreamEvent*
///                      v
///               +-------------------------+
///               | handle_stream_event()   |
///               | - start => pending      |
///               | - delta => update       |
///               | - stop  => finalize     |
///               +-------------------------+
///                      |
///                      | stream ends
///                      v
///               +-------------------------+
///               | finish_streaming_turn() |
///               +-------------------------+
///                      |
///                      v
///               +-------------------------+
///               | TurnState::Idle         |
///               +-------------------------+
/// ```
///
/// Tool-loop path:
///
/// ```text
/// user message
///   -> LLM stream
///   -> assistant tool_use
///   -> dispatch tool_registry
///   -> tool_result message
///   -> LLM stream again
///   -> ... until stop_reason == end_turn
/// ```
pub(super) enum TurnState {
    Idle,
    Streaming {
        /// The live SSE stream for the assistant turn.
        stream: AssistantStream,
        /// Updated near the end of the stream from `MessageDelta`.
        /// This tells us whether the model finished normally or stopped for a
        /// special reason such as `tool_use`.
        stop_reason: StopReason,
        /// Remembers tool names by tool-use id so later tool-result UI events can
        /// append stable labels to transcript records.
        tool_names_by_id: HashMap<String, String>,
        /// Remembers MCP server names by tool-use id.
        server_names_by_id: HashMap<String, String>,
        /// Completed blocks that we have seen the end of but haven't yet persisted.
        completed_assistant_blocks: Vec<ContentBlock>,
        /// Pending blocks that have started but not yet completed. Indexed by stream index.
        pending_blocks_by_index: HashMap<u32, PendingBlock>,
    },
}
