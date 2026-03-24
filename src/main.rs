//! Kloud - Main entry point

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use kloud::llm::client::LlmClient;
use kloud::tools::builtin::create_builtin_tools_registry;
use kloud::ui::UiBackend;
use kloud::{Result, cli, config, env::load_env, logging};
use tracing::trace;

#[tokio::main]
async fn main() -> Result<()> {
	load_env();

	// Parse command-line arguments
	let cli = cli::Cli::parse();

	// Initialize logging with appropriate level
	let log_level = cli.log_level();
	logging::init(log_level);

	tracing::info!("Starting kloud...");
	tracing::debug!("CLI arguments: {:?}", cli);

	// Load configuration
	let config = config::Config::load()?;
	tracing::debug!("Configuration loaded successfully");

	// Handle subcommands
	match cli.command {
		Some(cmd) => match cmd {
			cli::Commands::Run(args) => {
				run_interactive(config, args).await?;
			}
			cli::Commands::Exec(args) => {
				let task = args.task.join(" ");
				tracing::info!("Executing task: {}", task);
				tracing::debug!("Exec args: {:?}", args);
			}
			cli::Commands::Continue(args) => {
				tracing::info!("Continuing most recent conversation...");
				tracing::debug!("Continue args: {:?}", args);
			}
			cli::Commands::Resume(args) => {
				tracing::info!("Resuming conversation...");
				tracing::debug!("Resume args: {:?}", args);
			}
			cli::Commands::Doctor => {
				tracing::info!("Running health check...");
			}
			cli::Commands::Mcp(args) => match args.command {
				Some(cli::McpCommands::Add {
					name,
					command_or_url,
					args: _,
					transport,
					env: _,
				}) => {
					tracing::info!("Adding MCP server: {}", name);
					tracing::debug!("Command: {}, transport: {:?}", command_or_url, transport);
				}
				Some(cli::McpCommands::Remove {
					name,
				}) => {
					tracing::info!("Removing MCP server: {}", name);
				}
				Some(cli::McpCommands::List) => {
					tracing::info!("Listing MCP servers...");
				}
				Some(cli::McpCommands::Get {
					name,
				}) => {
					tracing::info!("Getting MCP server: {}", name);
				}
				Some(cli::McpCommands::Serve) => {
					tracing::info!("Starting MCP server...");
				}
				None => {
					tracing::info!("MCP command:");
				}
			},
			cli::Commands::Serve(args) => {
				tracing::info!("Starting server on {}:{}", args.address, args.port);
			}
			cli::Commands::Config(args) => match args.command {
				Some(cli::ConfigCommands::Show) => {
					tracing::info!("Showing configuration...");
				}
				Some(cli::ConfigCommands::Edit) => {
					tracing::info!("Editing configuration...");
				}
				Some(cli::ConfigCommands::Validate) => {
					tracing::info!("Validating configuration...");
				}
				Some(cli::ConfigCommands::Get {
					key,
				}) => {
					tracing::info!("Getting config: {}", key);
				}
				Some(cli::ConfigCommands::Set {
					key,
					value,
				}) => {
					tracing::info!("Setting config: {} = {}", key, value);
				}
				None => {
					tracing::info!("Config command:");
				}
			},
		},
		None => {
			// No subcommand — run interactive mode by default
			run_interactive(config, cli::RunArgs::default()).await?;
		}
	}

	tracing::info!("Kloud exited");
	Ok(())
}

/// Launch the interactive REPL session.
async fn run_interactive(config: config::Config, args: cli::RunArgs) -> Result<()> {
	// Resolve API key
	let api_key =
		config.llm.api_key.or_else(|| std::env::var("ANTHROPIC_API_KEY").ok()).ok_or_else(
			|| {
				kloud::error::ConfigError::MissingField(
				"API key not set. Set KLOUD_API_KEY, ANTHROPIC_API_KEY, or configure llm.api_key"
					.into(),
			)
			},
		)?;

	// Build LLM client
	let model = args.model.unwrap_or(config.llm.model);
	let client = kloud::llm::anthropic::AnthropicClient::new_builder()
		.with_api_key(api_key)
		.with_base_url(&config.llm.api_base_url)
		.with_model(&model)
		.with_max_response_tokens(config.llm.max_tokens)
		.build()
		.map_err(|e| kloud::error::LlmError::RequestFailed(e.to_string()))?;
	let model_info = client.model_info();
	let max_context_tokens = model_info.max_context_length;

	let system_prompt = args.system_prompt.unwrap_or_else(|| "You are a helpful assistant.".into());

	// Create tool registry
	let pwd = std::env::current_dir().map_err(kloud::error::Error::Io)?;
	trace!("Current working directory: {:?}", pwd);
	let tool_registry = create_builtin_tools_registry(&pwd);
	let tool_count = tool_registry.list_names().len();
	let workspace = pwd.display().to_string();
	let repo_root = detect_repo_root(&pwd);
	let branch = detect_branch(&pwd);
	let instruction_files = detect_instruction_files(&repo_root);
	let hook_count = count_active_hooks(&pwd);
	let effort = args.effort.clone().unwrap_or_else(|| "default effort".to_string());

	// Create channels and session
	let (ui_channels, ui_handle) = kloud::ui::create_ui_channels();
	let session = kloud::app::Session::new(
		Box::new(client),
		tool_registry,
		system_prompt,
		config.llm.max_tokens,
		ui_handle,
	);

	// Choose backend
	let backend = kloud::ui::tui::RatatuiBackend {
		model,
		max_context_tokens,
		workspace,
		branch,
		effort,
		tool_count,
		instruction_files,
		hook_count,
	};

	// Run session and UI concurrently
	let (session_result, ui_result) = tokio::join!(session.run(), backend.run(ui_channels));

	session_result?;
	ui_result?;

	Ok(())
}

fn run_git_output(workdir: &Path, args: &[&str]) -> Option<String> {
	let output = Command::new("git").args(args).current_dir(workdir).output().ok()?;
	if !output.status.success() {
		return None;
	}

	let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
	if text.is_empty() {
		None
	} else {
		Some(text)
	}
}

fn detect_repo_root(workdir: &Path) -> PathBuf {
	run_git_output(workdir, &["rev-parse", "--show-toplevel"])
		.map(PathBuf::from)
		.unwrap_or_else(|| workdir.to_path_buf())
}

fn detect_branch(workdir: &Path) -> String {
	run_git_output(workdir, &["branch", "--show-current"]).unwrap_or_else(|| "no git".to_string())
}

fn detect_instruction_files(repo_root: &Path) -> Vec<String> {
	["CLAUDE.md", "AGENTS.md"]
		.into_iter()
		.filter(|name| repo_root.join(name).is_file())
		.map(str::to_string)
		.collect()
}

fn count_active_hooks(workdir: &Path) -> usize {
	let Some(hooks_path) = run_git_output(workdir, &["rev-parse", "--git-path", "hooks"]) else {
		return 0;
	};

	let Ok(entries) = std::fs::read_dir(hooks_path) else {
		return 0;
	};

	entries
		.filter_map(std::result::Result::ok)
		.filter(|entry| entry.path().is_file())
		.filter(|entry| {
			entry
				.file_name()
				.to_str()
				.is_some_and(|name| !name.ends_with(".sample") && !name.starts_with('.'))
		})
		.count()
}
