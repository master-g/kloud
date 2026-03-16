# kloud

A minimal Claude Code implementation in Rust, built as a learning project to understand Agent internals by building one from scratch.

## What is this?

kloud is an Agentic Coding CLI that follows the same architecture as [Claude Code](https://docs.anthropic.com/en/docs/claude-code): an LLM-powered agent that can read files, write code, execute commands, and coordinate sub-agents — all through a terminal interface.

This project is structured as a 7-milestone learning roadmap, progressively building from a simple LLM API client to a full agent system with tool use, context management, and multi-agent collaboration.

## Current Status

**Milestone 1.1 — LLM Conversation** is complete. The project can make both non-streaming and streaming API calls to Anthropic-compatible endpoints.

| Milestone | Description | Status |
|-----------|-------------|--------|
| M1 | LLM conversation + REPL | In progress |
| M2 | Tool framework + implementations | Not started |
| M3 | Agent Loop (dual-loop core) | Not started |
| M4 | Context management + compression | Not started |
| M5 | Sub-agents + task DAG + skills | Not started |
| M6 | Async execution + concurrency | Not started |
| M7 | Team collaboration + worktree | Not started |

## Quick Start

### Prerequisites

- Rust 1.85.0+ (edition 2024)
- An Anthropic-compatible API key

### Setup

```bash
git clone https://github.com/master-g/kloud.git
cd kloud

# Set your API key
echo 'KLOUD_API_KEY=your-api-key-here' > .env

# Or export directly
export KLOUD_API_KEY=your-api-key-here
```

### Run Examples

```bash
# Non-streaming chat
RUST_LOG=info cargo run --example echo

# Streaming chat (SSE)
RUST_LOG=info cargo run --example streaming
```

### Build & Test

```bash
cargo build
cargo test
cargo fmt --all && cargo clippy -- -W warnings
```

## Project Structure

```
src/
├── main.rs          # Entry point, command dispatch
├── cli.rs           # CLI definition (clap derive)
├── config.rs        # TOML + env config management
├── error.rs         # Error type hierarchy (thiserror)
├── llm/
│   ├── anthropic.rs # AnthropicClient (Builder + chat + chat_stream)
│   ├── client.rs    # LlmClient trait abstraction
│   ├── sse.rs       # SSE frame decoder (tokio codec)
│   ├── types.rs     # Shared types (Role, ContentBlock, InputMessage)
│   ├── request.rs   # Request types (ChatRequest, SystemPrompt)
│   └── response.rs  # Response types (ChatResponse, StreamEvent)
└── tools/
    ├── traits.rs    # Tool trait definition
    └── call.rs      # ToolCall, ToolResult types
```

## Architecture

- **Anthropic Messages API** as the LLM backend
- **Trait-based abstractions** (`LlmClient`, `Tool`) for testability and extensibility
- **Dual-loop agent design** (planned): outer loop for follow-ups, inner loop for tool calls
- **Codec-based SSE parsing** using `tokio_util::codec::Decoder` for correct frame splitting

## Learning Roadmap

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for the full roadmap with Rust knowledge points and Agent engineering concepts at each milestone.

## License

MIT OR Apache-2.0
