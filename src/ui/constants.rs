//! Shared UI constants for the stdio and TUI frontends.

/// Frames per second for the TUI render loop.
pub const TUI_FPS: u64 = 20;

/// Number of recent activity entries kept in the dashboard side panel.
pub const MAX_ACTIVITY_ITEMS: usize = 8;

/// Maximum characters shown in truncated activity feed previews.
pub const ACTIVITY_PREVIEW_MAX_CHARS: usize = 48;

/// Width of the context usage meter in the status bar.
pub const CONTEXT_METER_WIDTH: usize = 10;

/// Glyph frames for the live activity animation (forward + reverse bounce).
/// Matches Claude Code's `[...DEFAULT_CHARACTERS, ...DEFAULT_CHARACTERS.reverse()]`.
pub const BOUNCE_FRAMES: [&str; 12] = [
	"·", "✢", "✳", "✶", "✻", "✽", // forward
	"✽", "✻", "✶", "✳", "✢", "·", // reverse
];

/// Glyph frame interval in milliseconds (120ms = ~8fps per Claude Code).
pub const GLYPH_FRAME_INTERVAL_MS: u64 = 120;

/// Shimmer speed for requesting mode (50ms per Claude Code).
pub const SHIMMER_SPEED_REQUESTING_MS: u64 = 50;

/// Shimmer speed for other modes (200ms per Claude Code).
pub const SHIMMER_SPEED_OTHER_MS: u64 = 200;

/// Time window used to fade live activity saturation after the last signal.
pub const ACTIVITY_FADE_WINDOW_MS: u128 = 9_500;

/// Lowest saturation retained by the live activity gradient.
pub const ACTIVITY_MIN_RETAIN: f32 = 0.24;

/// Stall detection threshold in milliseconds (3 seconds per Claude Code).
pub const STALL_THRESHOLD_MS: u64 = 3000;

/// Stall fade duration in milliseconds (2 seconds per Claude Code).
pub const STALL_FADE_MS: u64 = 2000;

/// Thinking shimmer delay in milliseconds (3 seconds per Claude Code).
pub const THINKING_DELAY_MS: u64 = 3000;

/// Thinking shimmer glow period in seconds (2 seconds per Claude Code).
pub const THINKING_GLOW_PERIOD_S: f32 = 2.0;

/// Thinking inactive color (gray per Claude Code).
pub const THINKING_INACTIVE: (u8, u8, u8) = (153, 153, 153);

/// Thinking shimmer color (lighter gray per Claude Code).
pub const THINKING_SHIMMER: (u8, u8, u8) = (185, 185, 185);

/// Show token count after this many seconds (30 per Claude Code).
pub const SHOW_TOKENS_AFTER_SECS: u64 = 30;

/// Minimum display time for "thinking" text before transitioning to "thought for Xs" (2s).
pub const THINKING_MIN_DISPLAY_MS: u64 = 2000;

/// How long to show "thought for Xs" before clearing (2s).
pub const THINKING_DURATION_SHOW_MS: u64 = 2000;

/// Bare width of the word "thinking" (used for width gating fallback).
pub const THINKING_BARE_WIDTH: usize = 8;

/// Error red color for stall indicator (per Claude Code).
pub const ERROR_RED: (u8, u8, u8) = (171, 43, 63);

/// How long to keep the last activity line visible after turn end (ms).
pub const ACTIVITY_SNAPSHOT_MS: u64 = 1800;

/// Tool result/use circle icon (macOS: ⏺, elsewhere: ●).
pub const TOOL_CIRCLE: &str = if cfg!(target_os = "macos") {
	"⏺"
} else {
	"●"
};

/// ASCII logo shown in the empty-state welcome card.
pub const DASHBOARD_LOGO: &[&str] = &[
	"   ▄▀▀▀▄▄▄▄▄▄▄▀▀▀▄   ",
	"   █▒▒░░░░░░░░░▒▒█   ",
	"    █░░█░░░░░█░░█    ",
	" ▄▄  █░░░▀█▀░░░█  ▄▄ ",
	"█░░█ ▀▄░░░░░░░▄▀ █░░█",
];

/// Default object shown for thinking activity.
pub const THINKING_ACTIVITY_OBJECT: &str = "through the request";

/// Verb rotation used while the assistant is thinking.
pub const THINKING_ACTIVITY_VERBS: &[&str] = &[
	"Accomplishing",
	"Actioning",
	"Actualizing",
	"Architecting",
	"Baking",
	"Beaming",
	"Beboppin'",
	"Befuddling",
	"Billowing",
	"Blanching",
	"Bloviating",
	"Boogieing",
	"Boondoggling",
	"Booping",
	"Bootstrapping",
	"Brewing",
	"Burrowing",
	"Calculating",
	"Canoodling",
	"Caramelizing",
	"Cascading",
	"Catapulting",
	"Cerebrating",
	"Channeling",
	"Channelling",
	"Choreographing",
	"Churning",
	"Clauding",
	"Coalescing",
	"Cogitating",
	"Combobulating",
	"Composing",
	"Computing",
	"Concocting",
	"Considering",
	"Contemplating",
	"Cooking",
	"Crafting",
	"Creating",
	"Crunching",
	"Crystallizing",
	"Cultivating",
	"Deciphering",
	"Deliberating",
	"Determining",
	"Dilly-dallying",
	"Discombobulating",
	"Doing",
	"Doodling",
	"Drizzling",
	"Ebbing",
	"Effecting",
	"Elucidating",
	"Embellishing",
	"Enchanting",
	"Envisioning",
	"Evaporating",
	"Fermenting",
	"Fiddle-faddling",
	"Finagling",
	"Flambeing",
	"Flibbertigibbeting",
	"Flowing",
	"Flummoxing",
	"Fluttering",
	"Forging",
	"Forming",
	"Frolicking",
	"Frosting",
	"Gallivanting",
	"Galloping",
	"Garnishing",
	"Generating",
	"Germinating",
	"Gitifying",
	"Grooving",
	"Gusting",
	"Harmonizing",
	"Hashing",
	"Hatching",
	"Herding",
	"Honking",
	"Hullaballooing",
	"Hyperspacing",
	"Ideating",
	"Imagining",
	"Improvising",
	"Incubating",
	"Inferring",
	"Infusing",
	"Ionizing",
	"Jitterbugging",
	"Julienning",
	"Kneading",
	"Leavening",
	"Levitating",
	"Lollygagging",
	"Manifesting",
	"Marinating",
	"Meandering",
	"Metamorphosing",
	"Misting",
	"Moonwalking",
	"Moseying",
	"Mulling",
	"Mustering",
	"Musing",
	"Nebulizing",
	"Nesting",
	"Newspapering",
	"Noodling",
	"Nucleating",
	"Orbiting",
	"Orchestrating",
	"Osmosing",
	"Perambulating",
	"Percolating",
	"Perusing",
	"Philosophising",
	"Photosynthesizing",
	"Pollinating",
	"Pondering",
	"Pontificating",
	"Pouncing",
	"Precipitating",
	"Prestidigitating",
	"Processing",
	"Proofing",
	"Propagating",
	"Puttering",
	"Puzzling",
	"Quantumizing",
	"Razzle-dazzling",
	"Razzmatazzing",
	"Recombobulating",
	"Reticulating",
	"Roosting",
	"Ruminating",
	"Sauteing",
	"Scampering",
	"Schlepping",
	"Scurrying",
	"Seasoning",
	"Shenaniganing",
	"Shimmying",
	"Simmering",
	"Skedaddling",
	"Sketching",
	"Slithering",
	"Smooshing",
	"Sock-hopping",
	"Spelunking",
	"Spinning",
	"Sprouting",
	"Stewing",
	"Sublimating",
	"Swirling",
	"Swooping",
	"Symbioting",
	"Synthesizing",
	"Tempering",
	"Thinking",
	"Thundering",
	"Tinkering",
	"Tomfoolering",
	"Topsy-turvying",
	"Transfiguring",
	"Transmuting",
	"Twisting",
	"Undulating",
	"Unfurling",
	"Unravelling",
	"Vibing",
	"Waddling",
	"Wandering",
	"Warping",
	"Whatchamacalliting",
	"Whirlpooling",
	"Whirring",
	"Whisking",
	"Wibbling",
	"Working",
	"Wrangling",
	"Zesting",
	"Zigzagging",
];

/// Fallback verb when an activity has no explicit verb list.
pub const DEFAULT_ACTIVITY_VERB: &str = "working";

const READ_ACTIVITY_VERBS: &[&str] = &["reading", "scanning", "checking"];
const ECHO_ACTIVITY_VERBS: &[&str] = &["echoing", "formatting", "returning"];
const DEFAULT_TOOL_ACTIVITY_VERBS: &[&str] = &["using", "calling", "waiting"];

/// User-facing verb/object copy for a running tool activity.
pub fn tool_activity_copy(name: &str) -> (&'static [&'static str], String) {
	match name {
		"read" => (READ_ACTIVITY_VERBS, "project files".to_string()),
		"echo" => (ECHO_ACTIVITY_VERBS, "tool output".to_string()),
		other => (DEFAULT_TOOL_ACTIVITY_VERBS, format!("tool `{other}`")),
	}
}
