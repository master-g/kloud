//! App state: messages, screen, permissions, notifications, tools.
#![allow(missing_docs)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

use crate::agent::message::{ActivityEntry, DisplayMessage, LiveActivity};
use crate::agent::view::{PendingPermissionView, Screen};

use super::Notification;
use super::anim_state::ActivitySnapshot;

/// Grouped app state: messages, screen, permissions, tools, notifications.
#[derive(Debug, Clone)]
pub struct AppState {
    pub messages: Vec<DisplayMessage>,
    pub screen: Screen,
    pub pending_permission: Option<PendingPermissionView>,
    pub notifications: Vec<Notification>,
    pub collapsed_tools: BTreeMap<usize, bool>,
    pub collapsed_thinking: HashSet<usize>,
    pub active_tools: Vec<String>,
    pub tool_start_times: HashMap<String, Instant>,
    pub recent_activity: Vec<ActivityEntry>,
    pub live_activity: Option<LiveActivity>,
    pub activity_snapshot: Option<ActivitySnapshot>,
    pub show_timestamps: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            screen: Screen::Prompt,
            pending_permission: None,
            notifications: Vec::new(),
            collapsed_tools: BTreeMap::new(),
            collapsed_thinking: HashSet::new(),
            active_tools: Vec::new(),
            tool_start_times: std::collections::HashMap::new(),
            recent_activity: Vec::new(),
            live_activity: None,
            activity_snapshot: None,
            show_timestamps: false,
        }
    }
}
