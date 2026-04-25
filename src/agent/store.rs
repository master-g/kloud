//! Session transcript store and reducer.

use std::time::Instant;

use crate::llm::response::StopReason;

use super::message::{
    ActivityAccent, ActivityEntry, ActivityEntryKind, DisplayBlock, LiveActivity, MessageLevel,
    MessageType, SpinnerMode, ToolStatus, TranscriptMessage, build_message_lookups,
    group_messages_for_display, normalize_messages, reorder_messages_in_ui,
};
use super::session_event::SessionEvent;
use super::view::{AssistantStatus, PendingPermissionView, PendingToast, Screen, SessionView};

const MAX_ACTIVITY_ITEMS: usize = 8;

/// Per-model token pricing (USD per million tokens).
#[derive(Debug, Clone, Copy)]
struct PricingTier {
    input_per_million: f64,
    output_per_million: f64,
}

impl PricingTier {
    const fn new(input: f64, output: f64) -> Self {
        Self {
            input_per_million: input,
            output_per_million: output,
        }
    }

    fn cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        (self.input_per_million * input_tokens as f64
            + self.output_per_million * output_tokens as f64)
            / 1_000_000.0
    }
}

const DEFAULT_PRICING: PricingTier = PricingTier::new(3.0, 15.0);

fn pricing_for_model(model: &str) -> PricingTier {
    static PRICING_TABLE: &[(&str, PricingTier)] = &[
        ("claude-sonnet-4-6", PricingTier::new(3.0, 15.0)),
        ("claude-opus-4-7", PricingTier::new(15.0, 75.0)),
        ("claude-haiku-4-5", PricingTier::new(0.80, 4.0)),
    ];
    PRICING_TABLE
        .iter()
        .find(|(name, _)| model.starts_with(name))
        .map(|(_, tier)| *tier)
        .unwrap_or(DEFAULT_PRICING)
}

const SPINNER_VERBS: &[&str] = &[
    "Thinking",
    "Planning",
    "Reasoning",
    "Inspecting",
    "Working",
    "Tracing",
    "Checking",
    "Exploring",
];

/// Store-backed session state.
#[derive(Debug, Clone)]
pub struct SessionStore {
    messages: Vec<TranscriptMessage>,
    current_assistant: Option<TranscriptMessage>,
    status: AssistantStatus,
    input_tokens: u32,
    output_tokens: u32,
    active_tools: Vec<String>,
    last_stop_reason: Option<StopReason>,
    recent_activity: Vec<ActivityEntry>,
    live_activity: Option<LiveActivity>,
    response_char_count: usize,
    screen: Screen,
    transcript_show_all: bool,
    pending_permission: Option<PendingPermissionView>,
    turn_verb: String,
    revision: u64,
    session_cost: f64,
    session_started_at: Option<Instant>,
    model_name: String,
    pending_toasts: Vec<PendingToast>,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStore {
    /// Create a new empty session store.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            current_assistant: None,
            status: AssistantStatus::Idle,
            input_tokens: 0,
            output_tokens: 0,
            active_tools: Vec::new(),
            last_stop_reason: None,
            recent_activity: Vec::new(),
            live_activity: None,
            response_char_count: 0,
            screen: Screen::Prompt,
            transcript_show_all: false,
            pending_permission: None,
            turn_verb: pick_random_verb(),
            revision: 0,
            session_cost: 0.0,
            session_started_at: None,
            model_name: String::new(),
            pending_toasts: Vec::new(),
        }
    }

    /// Apply a reducer event to the store.
    pub fn apply(&mut self, event: SessionEvent) {
        match event {
            SessionEvent::UserMessageSubmitted {
                text,
            } => {
                self.messages.push(TranscriptMessage::new(
                    MessageType::User,
                    vec![DisplayBlock::Text(text.clone())],
                ));
                self.record_activity(
                    ActivityEntryKind::User,
                    format!("You asked: {}", truncate_for_activity(&text)),
                );
            }
            SessionEvent::QueryStarted => {
                if self.session_started_at.is_none() {
                    self.session_started_at = Some(Instant::now());
                }
                self.status = AssistantStatus::Streaming;
                self.last_stop_reason = None;
                self.response_char_count = 0;
                self.turn_verb = pick_random_verb();
                self.set_activity(SpinnerMode::Requesting, ActivityAccent::Info);
                self.record_activity(ActivityEntryKind::Assistant, "Assistant started a turn");
            }
            SessionEvent::QueryCancelling => {
                self.status = AssistantStatus::Cancelling;
                self.record_activity(ActivityEntryKind::Meta, "Cancellation requested");
            }
            SessionEvent::QueryCompleted {
                stop_reason,
            } => {
                self.status = AssistantStatus::Idle;
                self.live_activity = None;
                if let Some(stop_reason) = stop_reason {
                    self.last_stop_reason = Some(stop_reason.clone());
                    self.record_activity(
                        ActivityEntryKind::Meta,
                        format!("Stop reason: {}", format_stop_reason(&stop_reason)),
                    );
                }
            }
            SessionEvent::InterruptRecorded {
                content,
            } => {
                self.messages.push(TranscriptMessage::new(
                    MessageType::System {
                        level: MessageLevel::Warning,
                    },
                    vec![DisplayBlock::Text(content.clone())],
                ));
                self.record_activity(ActivityEntryKind::Meta, truncate_for_activity(&content));
            }
            SessionEvent::AssistantMessageStarted => {
                self.current_assistant =
                    Some(TranscriptMessage::new(MessageType::Assistant, Vec::new()));
                self.touch_or_set_activity(SpinnerMode::Requesting, ActivityAccent::Info);
            }
            SessionEvent::AssistantTextDelta {
                text,
            } => {
                if text.is_empty() {
                    self.bump_revision();
                    return;
                }
                self.ensure_current_assistant();
                self.response_char_count += text.len();
                self.touch_or_set_activity(SpinnerMode::Responding, ActivityAccent::Info);
                if let Some(message) = self.current_assistant.as_mut() {
                    match message.blocks.last_mut() {
                        Some(DisplayBlock::Text(buffer)) => buffer.push_str(&text),
                        _ => message.blocks.push(DisplayBlock::Text(text)),
                    }
                }
            }
            SessionEvent::AssistantThinkingDelta {
                text,
            } => {
                if text.is_empty() {
                    self.bump_revision();
                    return;
                }
                self.ensure_current_assistant();
                self.response_char_count += text.len();
                self.touch_or_set_activity(SpinnerMode::Thinking, ActivityAccent::Info);
                if let Some(message) = self.current_assistant.as_mut() {
                    match message.blocks.last_mut() {
                        Some(DisplayBlock::Thinking(buffer)) => buffer.push_str(&text),
                        _ => message.blocks.push(DisplayBlock::Thinking(text)),
                    }
                }
            }
            SessionEvent::AssistantRedactedThinking {
                text,
            } => {
                self.ensure_current_assistant();
                self.touch_or_set_activity(SpinnerMode::Thinking, ActivityAccent::Info);
                if let Some(message) = self.current_assistant.as_mut() {
                    match message.blocks.last_mut() {
                        Some(DisplayBlock::RedactedThinking(buffer)) => buffer.push_str(&text),
                        _ => message.blocks.push(DisplayBlock::RedactedThinking(text.clone())),
                    }
                }
                if text.is_empty() {
                    self.record_activity(
                        ActivityEntryKind::Meta,
                        "Redacted thinking block received",
                    );
                }
            }
            SessionEvent::AssistantToolUseStarted {
                id,
                name,
                server_name,
                input,
            } => {
                self.ensure_current_assistant();
                self.touch_or_set_activity(SpinnerMode::ToolUse, ActivityAccent::Tool);
                if let Some(message) = self.current_assistant.as_mut() {
                    message.blocks.push(DisplayBlock::ToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        server_name,
                        input_json: serde_json::to_string(&input).unwrap_or_default(),
                        input_preview: format_json_preview(&input),
                        input,
                        status: ToolStatus::Pending,
                    });
                }
                self.record_activity(ActivityEntryKind::Tool, format!("Prepared tool `{name}`"));
            }
            SessionEvent::AssistantToolUseInputJsonDelta {
                id,
                partial_json,
            } => {
                if let Some(message) = self.current_assistant.as_mut() {
                    for block in message.blocks.iter_mut().rev() {
                        if let DisplayBlock::ToolUse {
                            id: block_id,
                            input,
                            input_json,
                            input_preview,
                            ..
                        } = block
                            && block_id == &id
                        {
                            input_json.push_str(&partial_json);
                            if let Ok(next_input) =
                                serde_json::from_str::<serde_json::Value>(input_json)
                            {
                                *input = next_input;
                                *input_preview = format_json_preview(input);
                            }
                            break;
                        }
                    }
                }
            }
            SessionEvent::AssistantMessageCommitted {
                blocks,
                stop_reason,
            } => {
                self.current_assistant = None;
                if !blocks.is_empty() {
                    let mut message = TranscriptMessage::new(MessageType::Assistant, blocks);
                    message.stop_reason = Some(stop_reason.clone());
                    self.messages.push(message);
                }
                self.last_stop_reason = Some(stop_reason);
            }
            SessionEvent::ToolExecutionStarted {
                id,
                name,
                server_name: _,
            } => {
                self.touch_or_set_activity(SpinnerMode::ToolUse, ActivityAccent::Tool);
                self.update_tool_use_status(&id, ToolStatus::Running);
                if !self.active_tools.iter().any(|active| active == &name) {
                    self.active_tools.push(name.clone());
                }
                self.record_activity(ActivityEntryKind::Tool, format!("Started tool `{name}`"));
            }
            SessionEvent::ToolExecutionFinished {
                id,
                name,
                server_name,
                output,
                is_error,
            } => {
                self.update_tool_use_status(
                    &id,
                    if is_error {
                        ToolStatus::Errored
                    } else {
                        ToolStatus::Done
                    },
                );
                self.append_tool_result(
                    id.clone(),
                    name.clone(),
                    server_name,
                    output.clone(),
                    is_error,
                );
                if let Some(index) = self.active_tools.iter().position(|active| active == &name) {
                    self.active_tools.remove(index);
                }
                if is_error {
                    self.record_activity(
                        ActivityEntryKind::Error,
                        format!("Tool `{name}` failed: {}", truncate_for_activity(&output)),
                    );
                } else {
                    self.record_activity(
                        ActivityEntryKind::Success,
                        format!("Tool `{name}` finished: {}", truncate_for_activity(&output)),
                    );
                }
            }
            SessionEvent::SystemMessageAdded {
                content,
                level,
            } => {
                let (clean_text, is_toast) = match content.strip_prefix("[toast] ") {
                    Some(rest) => (rest.to_string(), true),
                    None => (content.clone(), false),
                };
                if is_toast {
                    self.pending_toasts.push(PendingToast {
                        text: clean_text.clone(),
                        level,
                    });
                }
                self.messages.push(TranscriptMessage::new(
                    MessageType::System {
                        level,
                    },
                    vec![DisplayBlock::Text(clean_text.clone())],
                ));
                self.record_activity(
                    match level {
                        MessageLevel::Error => ActivityEntryKind::Error,
                        MessageLevel::Info | MessageLevel::Warning => ActivityEntryKind::Meta,
                    },
                    truncate_for_activity(&clean_text),
                );
            }
            SessionEvent::UsageUpdated {
                input_tokens,
                output_tokens,
            } => {
                let delta_in = input_tokens.saturating_sub(self.input_tokens);
                let delta_out = output_tokens.saturating_sub(self.output_tokens);
                self.input_tokens = input_tokens;
                self.output_tokens = output_tokens;
                let pricing = pricing_for_model(&self.model_name);
                self.session_cost += pricing.cost(delta_in, delta_out);
            }
            SessionEvent::ScreenChanged {
                screen,
            } => {
                self.screen = screen;
            }
            SessionEvent::TranscriptShowAllChanged {
                show_all,
            } => {
                self.transcript_show_all = show_all;
            }
            SessionEvent::StreamStalled => {
                self.record_activity(ActivityEntryKind::Meta, "Stream stalled");
            }
            SessionEvent::StreamResumed => {
                self.record_activity(ActivityEntryKind::Meta, "Stream resumed");
            }
            SessionEvent::PendingPermissionChanged {
                pending_permission,
            } => {
                self.pending_permission = pending_permission;
            }
        }

        self.bump_revision();
    }

    /// Build a snapshot for UI backends.
    pub fn view(&self) -> SessionView {
        let normalized =
            normalize_messages(self.messages.as_slice(), self.current_assistant.as_ref());
        let lookups = build_message_lookups(normalized.as_slice());
        let reordered = reorder_messages_in_ui(normalized.as_slice(), &lookups);
        let messages = group_messages_for_display(reordered.as_slice());

        SessionView {
            revision: self.revision,
            messages,
            status: self.status,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            active_tools: self.active_tools.clone(),
            last_stop_reason: self.last_stop_reason.clone(),
            recent_activity: self.recent_activity.clone(),
            live_activity: self.live_activity.clone(),
            response_char_count: self.response_char_count,
            screen: self.screen,
            transcript_show_all: self.transcript_show_all,
            pending_permission: self.pending_permission.clone(),
            session_cost: self.session_cost,
            session_duration: self
                .session_started_at
                .map_or(std::time::Duration::ZERO, |start| start.elapsed()),
            pending_toasts: self.pending_toasts.clone(),
        }
    }

    /// Update the model name displayed in the status bar.
    pub fn set_model_name(&mut self, name: String) {
        self.model_name = name;
    }

    fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    fn ensure_current_assistant(&mut self) {
        if self.current_assistant.is_none() {
            self.current_assistant =
                Some(TranscriptMessage::new(MessageType::Assistant, Vec::new()));
        }
    }

    fn update_tool_use_status(&mut self, tool_use_id: &str, status: ToolStatus) {
        for message in self.messages.iter_mut().rev() {
            if message.message_type != MessageType::Assistant {
                continue;
            }
            for block in &mut message.blocks {
                if let DisplayBlock::ToolUse {
                    id,
                    status: block_status,
                    ..
                } = block
                    && id == tool_use_id
                {
                    *block_status = status;
                    return;
                }
            }
        }
    }

    fn append_tool_result(
        &mut self,
        tool_use_id: String,
        name: String,
        server_name: Option<String>,
        output: String,
        is_error: bool,
    ) {
        let block = DisplayBlock::ToolResult {
            tool_use_id,
            name,
            server_name,
            output,
            is_error,
        };

        let can_append = self.messages.last().is_some_and(|last| {
            last.message_type == MessageType::User
                && last.blocks.iter().all(DisplayBlock::is_tool_result)
        });

        if can_append {
            if let Some(last) = self.messages.last_mut() {
                last.blocks.push(block);
            }
            return;
        }

        self.messages.push(TranscriptMessage::new(MessageType::User, vec![block]));
    }

    fn set_activity(&mut self, mode: SpinnerMode, accent: ActivityAccent) {
        let now = Instant::now();
        self.live_activity = Some(LiveActivity {
            message: self.turn_verb.clone(),
            accent,
            mode,
            started_at: now,
            last_signal_at: now,
        });
    }

    fn touch_or_set_activity(&mut self, mode: SpinnerMode, accent: ActivityAccent) {
        if let Some(activity) = self.live_activity.as_mut() {
            activity.mode = mode;
            activity.accent = accent;
            activity.last_signal_at = Instant::now();
        } else {
            self.set_activity(mode, accent);
        }
    }

    fn record_activity(&mut self, kind: ActivityEntryKind, text: impl Into<String>) {
        self.recent_activity.push(ActivityEntry {
            kind,
            text: text.into(),
        });
        if self.recent_activity.len() > MAX_ACTIVITY_ITEMS {
            let overflow = self.recent_activity.len() - MAX_ACTIVITY_ITEMS;
            self.recent_activity.drain(0..overflow);
        }
    }
}

fn pick_random_verb() -> String {
    let idx = fastrand::usize(0..SPINNER_VERBS.len());
    format!("{}…", SPINNER_VERBS[idx])
}

fn truncate_for_activity(text: &str) -> String {
    const MAX_ACTIVITY_CHARS: usize = 72;
    let mut out = text.chars().take(MAX_ACTIVITY_CHARS).collect::<String>();
    if text.chars().count() > MAX_ACTIVITY_CHARS {
        out.push('…');
    }
    out
}

fn format_json_preview(value: &serde_json::Value) -> String {
    let formatted = match serde_json::to_string_pretty(value) {
        Ok(json) => json,
        Err(_) => value.to_string(),
    };
    truncate_json_preview(formatted, 240)
}

fn truncate_json_preview(value: String, max_chars: usize) -> String {
    let total_chars = value.chars().count();
    if total_chars <= max_chars {
        return value;
    }

    let mut truncated = value.chars().take(max_chars).collect::<String>();
    truncated.push('…');
    truncated
}

fn format_stop_reason(reason: &StopReason) -> &'static str {
    match reason {
        StopReason::EndTurn => "end_turn",
        StopReason::MaxTokens => "max_tokens",
        StopReason::StopSequence => "stop_sequence",
        StopReason::ToolUse => "tool_use",
        StopReason::PauseTurn => "pause_turn",
        StopReason::Refusal => "refusal",
        StopReason::ModelContextWindowExceeded => "ctx_exceeded",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::session_event::SessionEvent;

    #[test]
    fn groups_tool_result_after_matching_tool_use() {
        let mut store = SessionStore::new();
        store.apply(SessionEvent::UserMessageSubmitted {
            text: "hi".into(),
        });
        store.apply(SessionEvent::QueryStarted);
        store.apply(SessionEvent::AssistantMessageStarted);
        store.apply(SessionEvent::AssistantTextDelta {
            text: "Working".into(),
        });
        store.apply(SessionEvent::AssistantToolUseStarted {
            id: "toolu_1".into(),
            name: "read".into(),
            server_name: None,
            input: serde_json::json!({ "path": "README.md" }),
        });
        store.apply(SessionEvent::AssistantMessageCommitted {
            blocks: vec![
                DisplayBlock::Text("Working".into()),
                DisplayBlock::ToolUse {
                    id: "toolu_1".into(),
                    name: "read".into(),
                    server_name: None,
                    input: serde_json::json!({ "path": "README.md" }),
                    input_json: r#"{"path":"README.md"}"#.into(),
                    input_preview: "{\n  \"path\": \"README.md\"\n}".into(),
                    status: ToolStatus::Pending,
                },
            ],
            stop_reason: StopReason::ToolUse,
        });
        store.apply(SessionEvent::ToolExecutionStarted {
            id: "toolu_1".into(),
            name: "read".into(),
            server_name: None,
        });
        store.apply(SessionEvent::ToolExecutionFinished {
            id: "toolu_1".into(),
            name: "read".into(),
            server_name: None,
            output: "ok".into(),
            is_error: false,
        });

        let view = store.view();
        assert_eq!(view.messages.len(), 2);
        let assistant = &view.messages[1];
        assert_eq!(assistant.blocks.len(), 3);
        assert!(matches!(assistant.blocks[1], DisplayBlock::ToolUse { .. }));
        assert!(matches!(assistant.blocks[2], DisplayBlock::ToolResult { .. }));
    }

    #[test]
    fn screen_and_transcript_show_all_are_exposed_in_view() {
        let mut store = SessionStore::new();

        store.apply(SessionEvent::ScreenChanged {
            screen: Screen::Transcript,
        });
        store.apply(SessionEvent::TranscriptShowAllChanged {
            show_all: true,
        });

        let view = store.view();
        assert_eq!(view.screen, Screen::Transcript);
        assert!(view.transcript_show_all);
    }
}
