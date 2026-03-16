//! CLI module using clap derive
//!
//! Provides command-line argument parsing with support for future subcommands.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Kloud CLI application
#[derive(Parser, Debug)]
#[command(name = "kloud")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A minimal Agentic Coding CLI implementation in Rust", long_about = None)]
pub struct Cli {
	/// Increase verbosity (-v, -vv, -vvv)
	#[arg(short = 'v', long, action = clap::ArgAction::Count)]
	pub verbose: u8,

	/// Configuration file path
	#[arg(short = 'c', long, global = true)]
	pub config: Option<PathBuf>,

	/// Disable colored output
	#[arg(long, global = true)]
	pub no_color: bool,

	/// Subcommands
	#[command(subcommand)]
	pub command: Option<Commands>,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
	/// Run the interactive agent (default)
	#[command(name = "run", alias = "r")]
	Run(RunArgs),

	/// Execute a single task and print result
	#[command(name = "exec", alias = "e", visible_alias = "print")]
	Exec(ExecArgs),

	/// Continue the most recent conversation
	#[command(name = "continue", alias = "c")]
	Continue(ContinueArgs),

	/// Resume a conversation by session ID
	#[command(name = "resume", alias = "re")]
	Resume(ResumeArgs),

	/// Check the health of kloud
	Doctor,

	/// Configure and manage MCP servers (future)
	#[command(name = "mcp")]
	Mcp(McpArgs),

	/// Start a server mode (future)
	#[command(name = "serve", alias = "s")]
	Serve(ServeArgs),

	/// Manage configuration
	Config(ConfigArgs),
}

/// Arguments for run command
#[derive(Parser, Debug, Default)]
pub struct RunArgs {
	/// System prompt to use
	#[arg(long)]
	pub system_prompt: Option<String>,

	/// Model for the current session
	#[arg(long)]
	pub model: Option<String>,

	/// Allowed tools (comma or space-separated)
	#[arg(long, alias = "allowed-tools")]
	pub allowed_tools: Option<Vec<String>>,

	/// Disallowed tools (comma or space-separated)
	#[arg(long, alias = "disallowed-tools")]
	pub disallowed_tools: Option<Vec<String>>,

	/// Create a new git worktree for this session
	#[arg(long, short = 'w')]
	pub worktree: Option<String>,

	/// Disable slash commands (skills)
	#[arg(long)]
	pub disable_slash_commands: bool,

	/// Append to the default system prompt
	#[arg(long)]
	pub append_system_prompt: Option<String>,

	/// Effort level (low, medium, high)
	#[arg(long)]
	pub effort: Option<String>,
}

/// Arguments for exec command (non-interactive)
#[derive(Parser, Debug)]
pub struct ExecArgs {
	/// Task description (required)
	#[arg(required = true)]
	pub task: Vec<String>,

	/// Model for the current session
	#[arg(long)]
	pub model: Option<String>,

	/// Allowed tools
	#[arg(long, alias = "allowed-tools")]
	pub allowed_tools: Option<Vec<String>>,

	/// Disallowed tools
	#[arg(long, alias = "disallowed-tools")]
	pub disallowed_tools: Option<Vec<String>>,

	/// Output format: text, json, stream-json
	#[arg(long, default_value = "text")]
	pub output_format: String,

	/// Output file for result
	#[arg(long, short = 'o')]
	pub output: Option<PathBuf>,

	/// Disable streaming output
	#[arg(long)]
	pub no_stream: bool,

	/// System prompt to use
	#[arg(long)]
	pub system_prompt: Option<String>,

	/// Create a new git worktree for this session
	#[arg(long, short = 'w')]
	pub worktree: Option<String>,

	/// Continue the most recent conversation
	#[arg(long)]
	pub continue_session: bool,

	/// Resume a conversation by session ID
	#[arg(long)]
	pub resume: Option<String>,
}

/// Arguments for continue command
#[derive(Parser, Debug)]
pub struct ContinueArgs {
	/// Create a new session ID instead of reusing original
	#[arg(long)]
	pub fork_session: bool,
}

/// Arguments for resume command
#[derive(Parser, Debug)]
pub struct ResumeArgs {
	/// Session ID to resume
	#[arg(value_name = "session-id")]
	pub session_id: Option<String>,

	/// Create a new session ID instead of reusing original
	#[arg(long)]
	pub fork_session: bool,

	/// Open interactive picker with optional search term
	#[arg(long, short = 'p')]
	pub pick: Option<String>,
}

/// Arguments for mcp command (future)
#[derive(Parser, Debug)]
pub struct McpArgs {
	/// MCP subcommand
	#[command(subcommand)]
	pub command: Option<McpCommands>,
}

/// MCP subcommands
#[derive(Subcommand, Debug)]
pub enum McpCommands {
	/// Add an MCP server
	Add {
		/// Server name
		name: String,

		/// Server command or URL
		command_or_url: String,

		/// Additional arguments
		args: Vec<String>,

		/// Transport type (stdio, http, sse)
		#[arg(long)]
		transport: Option<String>,

		/// Environment variables (key=value)
		#[arg(short = 'e', long)]
		env: Option<Vec<String>>,
	},

	/// Remove an MCP server
	Remove {
		/// Server name
		name: String,
	},

	/// List configured MCP servers
	List,

	/// Get details about an MCP server
	Get {
		/// Server name
		name: String,
	},

	/// Start the MCP server
	Serve,
}

/// Arguments for serve command (future)
#[derive(Parser, Debug)]
pub struct ServeArgs {
	/// Server address
	#[arg(long, default_value = "127.0.0.1")]
	pub address: String,

	/// Server port
	#[arg(long, default_value = "8080")]
	pub port: u16,

	/// Enable API key authentication
	#[arg(long)]
	pub auth: bool,
}

/// Arguments for config command
#[derive(Parser, Debug)]
pub struct ConfigArgs {
	/// Config subcommand
	#[command(subcommand)]
	pub command: Option<ConfigCommands>,
}

/// Config subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
	/// Show current configuration
	Show,

	/// Edit configuration file
	Edit,

	/// Validate configuration
	Validate,

	/// Get a specific config value
	Get {
		/// Config key (e.g., "llm.model")
		key: String,
	},

	/// Set a specific config value
	Set {
		/// Config key (e.g., "llm.model")
		key: String,

		/// Config value
		value: String,
	},
}

impl Cli {
	/// Get the log level based on verbose count
	pub fn log_level(&self) -> &'static str {
		match self.verbose {
			0 => "info",
			1 => "debug",
			_ => "trace",
		}
	}
}
