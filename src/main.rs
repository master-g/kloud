//! Kloud - Main entry point

use clap::Parser;
use kloud::{Result, cli, config, env::load_env, logging};

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
	let _config = config::Config::load()?;
	tracing::debug!("Configuration loaded successfully");

	// Handle subcommands
	match cli.command {
		Some(cmd) => match cmd {
			cli::Commands::Run(args) => {
				tracing::info!("Running interactive mode...");
				tracing::debug!("Run args: {:?}", args);
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
			// No subcommand, run interactive mode by default
			tracing::info!("Starting interactive mode...");
		}
	}

	tracing::info!("Kloud exited");
	Ok(())
}
