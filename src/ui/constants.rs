//! Shared UI constants for the stdio and TUI frontends.

/// Frames per second for the TUI render loop.
pub const TUI_FPS: u64 = 20;

/// Number of recent activity entries kept in the dashboard side panel.
pub const MAX_ACTIVITY_ITEMS: usize = 8;

/// Maximum characters shown in truncated activity feed previews.
pub const ACTIVITY_PREVIEW_MAX_CHARS: usize = 48;

/// Width of the context usage meter in the status bar.
pub const CONTEXT_METER_WIDTH: usize = 10;

/// Animation cadence divisor for the live activity line.
pub const ACTIVITY_TICK_DIVISOR: u64 = 4;

/// Glyph frames for the live activity animation.
pub const ACTIVITY_FRAMES: [&str; 6] = ["·", "✻", "✽", "✶", "✳", "✢"];

/// Time window used to fade live activity saturation after the last signal.
pub const ACTIVITY_FADE_WINDOW_MS: u128 = 9_500;

/// Lowest saturation retained by the live activity gradient.
pub const ACTIVITY_MIN_RETAIN: f32 = 0.24;

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
