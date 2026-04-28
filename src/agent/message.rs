//! Internal transcript and display message types.
#![allow(missing_docs)]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crate::llm::response::StopReason;
use crate::tools::ToolResultKind;

/// Stable identifier for transcript messages.
pub type MessageId = u64;

static NEXT_MESSAGE_ID: AtomicU64 = AtomicU64::new(1);

/// Generate the next stable transcript message ID.
pub fn next_message_id() -> MessageId {
    NEXT_MESSAGE_ID.fetch_add(1, Ordering::Relaxed)
}

/// Status of a tool block in the transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    Pending,
    Running,
    Done,
    Errored,
}

/// Severity level for system messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
}

/// High-level transcript message kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    User,
    Assistant,
    System {
        level: MessageLevel,
    },
    Progress {
        parent_tool_use_id: Option<String>,
    },
}

/// Activity accent family for the transient status line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityAccent {
    Info,
    Tool,
}

/// Current spinner mode for the active query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerMode {
    Requesting,
    Responding,
    ToolUse,
    Thinking,
}

/// Ephemeral activity line data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveActivity {
    pub message: String,
    pub accent: ActivityAccent,
    pub mode: SpinnerMode,
    pub started_at: Instant,
    pub last_signal_at: Instant,
}

/// Short activity log item shown in the side panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityEntryKind {
    User,
    Assistant,
    Tool,
    Success,
    Error,
    Meta,
}

/// A single activity log item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityEntry {
    pub kind: ActivityEntryKind,
    pub text: String,
}

/// A renderable content block.
#[derive(Debug, Clone)]
pub enum DisplayBlock {
    Text(String),
    Thinking(String),
    RedactedThinking(String),
    ToolUse {
        id: String,
        name: String,
        display_name: String,
        server_name: Option<String>,
        input: serde_json::Value,
        input_json: String,
        input_preview: String,
        status: ToolStatus,
        rendered_use: Vec<ratatui::text::Line<'static>>,
        progress_text: Option<String>,
    },
    ToolResult {
        tool_use_id: String,
        name: String,
        server_name: Option<String>,
        output: String,
        kind: ToolResultKind,
        rendered_result: Vec<ratatui::text::Line<'static>>,
    },
}

impl DisplayBlock {
    /// Returns the tool-use id if the block refers to one.
    pub fn tool_use_id(&self) -> Option<&str> {
        match self {
            Self::ToolUse {
                id,
                ..
            } => Some(id),
            Self::ToolResult {
                tool_use_id,
                ..
            } => Some(tool_use_id),
            _ => None,
        }
    }

    /// Returns whether this block is a user-side tool result.
    pub fn is_tool_result(&self) -> bool {
        matches!(self, Self::ToolResult { .. })
    }
}

/// Durable transcript message used by the session store.
#[derive(Debug, Clone)]
pub struct TranscriptMessage {
    pub id: MessageId,
    pub message_type: MessageType,
    pub blocks: Vec<DisplayBlock>,
    pub stop_reason: Option<StopReason>,
    pub is_meta: bool,
}

impl TranscriptMessage {
    /// Create a new transcript message with a fresh ID.
    pub fn new(message_type: MessageType, blocks: Vec<DisplayBlock>) -> Self {
        Self {
            id: next_message_id(),
            message_type,
            blocks,
            stop_reason: None,
            is_meta: false,
        }
    }
}

/// Display-oriented message grouped for rendering.
#[derive(Debug, Clone)]
pub struct DisplayMessage {
    pub id: MessageId,
    pub message_type: MessageType,
    pub blocks: Vec<DisplayBlock>,
    pub created_at: Option<std::time::Instant>,
}

/// One normalized block-level message.
#[derive(Debug, Clone)]
pub struct NormalizedMessage {
    pub source_id: MessageId,
    pub display_group_id: MessageId,
    pub message_type: MessageType,
    pub block: DisplayBlock,
    pub stop_reason: Option<StopReason>,
}

/// Precomputed relationships between tool use and tool result blocks.
#[derive(Debug, Clone, Default)]
pub struct MessageLookups {
    pub assistant_message_id_by_tool_use_id: HashMap<String, MessageId>,
    pub tool_result_by_tool_use_id: HashMap<String, (MessageId, DisplayBlock)>,
    pub resolved_tool_use_ids: HashSet<String>,
    pub errored_tool_use_ids: HashSet<String>,
}

/// Split transcript messages so each block can be independently reordered.
pub fn normalize_messages(
    messages: &[TranscriptMessage],
    streaming_assistant: Option<&TranscriptMessage>,
) -> Vec<NormalizedMessage> {
    let mut normalized = Vec::new();

    // into_iter on Option<&T> yields &T; .iter() would yield &&T (type mismatch with chain)
    #[allow(clippy::useless_conversion)]
    for message in messages.iter().chain(streaming_assistant.into_iter()) {
        for block in &message.blocks {
            normalized.push(NormalizedMessage {
                source_id: message.id,
                display_group_id: message.id,
                message_type: message.message_type.clone(),
                block: block.clone(),
                stop_reason: message.stop_reason.clone(),
            });
        }
    }

    normalized
}

/// Build O(1) relationships used by the render pipeline.
pub fn build_message_lookups(messages: &[NormalizedMessage]) -> MessageLookups {
    let mut lookups = MessageLookups::default();

    for message in messages {
        match (&message.message_type, &message.block) {
            (
                MessageType::Assistant,
                DisplayBlock::ToolUse {
                    id,
                    status,
                    ..
                },
            ) => {
                lookups.assistant_message_id_by_tool_use_id.insert(id.clone(), message.source_id);
                if matches!(status, ToolStatus::Done | ToolStatus::Errored) {
                    lookups.resolved_tool_use_ids.insert(id.clone());
                }
                if *status == ToolStatus::Errored {
                    lookups.errored_tool_use_ids.insert(id.clone());
                }
            }
            (
                MessageType::User,
                DisplayBlock::ToolResult {
                    tool_use_id,
                    kind,
                    ..
                },
            ) => {
                lookups
                    .tool_result_by_tool_use_id
                    .insert(tool_use_id.clone(), (message.source_id, message.block.clone()));
                lookups.resolved_tool_use_ids.insert(tool_use_id.clone());
                if matches!(kind, ToolResultKind::Error) {
                    lookups.errored_tool_use_ids.insert(tool_use_id.clone());
                }
            }
            _ => {}
        }
    }

    lookups
}

/// Reorder normalized blocks so tool results render directly after their tool use.
pub fn reorder_messages_in_ui(
    messages: &[NormalizedMessage],
    lookups: &MessageLookups,
) -> Vec<NormalizedMessage> {
    let mut reordered = Vec::new();
    let mut inserted_tool_results = HashSet::new();

    for message in messages {
        match (&message.message_type, &message.block) {
            (
                MessageType::Assistant,
                DisplayBlock::ToolUse {
                    id,
                    ..
                },
            ) => {
                reordered.push(message.clone());
                if let Some((source_id, tool_result)) = lookups.tool_result_by_tool_use_id.get(id) {
                    inserted_tool_results.insert(id.clone());
                    reordered.push(NormalizedMessage {
                        source_id: *source_id,
                        display_group_id: message.source_id,
                        message_type: MessageType::User,
                        block: tool_result.clone(),
                        stop_reason: None,
                    });
                }
            }
            (
                MessageType::User,
                DisplayBlock::ToolResult {
                    tool_use_id,
                    ..
                },
            ) if lookups.assistant_message_id_by_tool_use_id.contains_key(tool_use_id) => {
                if !inserted_tool_results.contains(tool_use_id) {
                    continue;
                }
            }
            _ => reordered.push(message.clone()),
        }
    }

    reordered
}

/// Group reordered block-level messages back into renderable transcript messages.
pub fn group_messages_for_display(messages: &[NormalizedMessage]) -> Vec<DisplayMessage> {
    let mut grouped = Vec::new();

    for message in messages {
        let should_append =
            grouped.last().is_some_and(|last: &DisplayMessage| last.id == message.display_group_id);

        if should_append {
            if let Some(last) = grouped.last_mut() {
                last.blocks.push(message.block.clone());
            }
            continue;
        }

        grouped.push(DisplayMessage {
            id: message.display_group_id,
            message_type: message.message_type.clone(),
            blocks: vec![message.block.clone()],
            created_at: Some(std::time::Instant::now()),
        });
    }

    grouped
}
