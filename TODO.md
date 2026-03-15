# TODO — 开发交接文档

> 本文件记录当前进度和下一步计划，方便跨会话恢复上下文。

## 当前位置

**Milestone 1.1 — LLM 对话**，类型层已完成，下一步实现 `AnthropicClient`。

## 已完成的工作

### Bootstrap 阶段（早于本次会话）
- CLI 解析 (`src/cli.rs`) — clap derive
- 配置管理 (`src/config.rs`) — TOML + 环境变量，默认指向 Anthropic API
- 错误体系 (`src/error.rs`) — thiserror 层次结构
- 日志 (`src/logging.rs`) — tracing，接受 level 参数而非 set_var
- 工具类型 (`src/tools/`) — Tool trait + ToolCall/ToolResult 类型

### 本次会话完成
1. **Rust edition 2021 → 2024**
   - `Cargo.toml` 更新 edition + rust-version
   - 修复 `set_var` unsafe 问题（重构 logging::init）
   - 启用 let chains（`if let ... && let ...`）
   - import 排序规则适配

2. **路线图重写** (`docs/plan/ROADMAP.md`)
   - 原 88 个零碎 step 重组为 7 个里程碑、18 个子任务
   - 每个子任务含 Rust 知识点 + Agent 知识点 + 验收标准
   - CLAUDE.md 更新 mentor 角色定位

3. **代码清理**
   - 删除 `src/state.rs`（属于 M3，未被使用，到时用 struct-based 设计重写）
   - `config.rs` 的 `println!` 替换为 `tracing::debug!`
   - 默认值改为 Anthropic（`api.anthropic.com`，`claude-sonnet-4-20250514`，env var `ANTHROPIC_API_KEY`）

4. **LLM 模块类型定义** (`src/llm/`)
   - `types.rs` — Role, CacheControl, ContentBlock (Text/Thinking/RedactedThinking/ToolUse/ToolResult), InputMessage
   - `request.rs` — ChatRequest, SystemPrompt, Prompt, Thinking, ToolChoice
   - `response.rs` — ChatResponse, StopReason, Usage + 流式 StreamEvent, Delta, MessageDelta
   - `error.rs` — ApiError（API 返回的错误体）
   - `client.rs` — LlmClient trait (chat + chat_stream), ModelInfo

### 关键设计决策记录
- **ContentBlock 用 `#[serde(tag = "type")]`**，不手动存 typ 字段
- **纯数据类型用 `#[allow(missing_docs)]`**，trait 保留文档要求
- **`LlmClient` 有两个方法**: `chat`（非流式）和 `chat_stream`（返回 `Pin<Box<dyn Stream>>`）
- **`ApiError` 在 `llm/error.rs`**（API 协议的一部分），与 `src/error.rs` 的 `LlmError`（客户端侧错误）是不同层次

## 下一步：实现 AnthropicClient

### 要做什么
创建 `src/llm/anthropic.rs`，实现 `LlmClient` trait：

1. **struct 定义**: 持有 `reqwest::Client`、API key、base URL、默认 model
2. **`chat()` 实现**:
   - 将 `ChatRequest` 序列化为 JSON
   - POST 到 `{base_url}/v1/messages`
   - 设置 headers: `x-api-key`, `anthropic-version: 2023-06-01`, `content-type: application/json`
   - 反序列化响应为 `ChatResponse`
   - 错误处理: HTTP 错误 → API 错误 → LlmError
3. **`chat_stream()` 实现**（可以先跳过，优先跑通非流式）:
   - 设置 `stream: true`
   - 用 `reqwest` 的 `bytes_stream()` 获取 SSE 流
   - 解析 SSE 格式（考虑 `eventsource-stream` crate）
   - 每行 `data: {...}` 反序列化为 `StreamEvent`

### 要学的 Rust 知识
- `reqwest::Client` 异步 HTTP（复用连接池）
- `serde_json::to_value` / `to_string` 手动序列化（ChatRequest 没有 derive Serialize）
- Header 构建: `reqwest::header::HeaderMap`
- 错误类型转换: `reqwest::Error` → `LlmError`

### 要学的 Agent 知识
- Anthropic API 的认证方式（`x-api-key` header，不是 Bearer token）
- API 版本控制（`anthropic-version` header）
- 流式 SSE 协议的解析逻辑

### 验收标准
- `cargo test` 中用 MockClient 测试消息往返
- 用真实 API key 运行 `cargo run` 能完成一次对话

## 之后的路

完成 `AnthropicClient` 后 → M1.2 REPL 交互循环 → M2 工具框架 → M3 Agent Loop

完整路线图见 `docs/plan/ROADMAP.md`。

## 文件结构快照

```
src/
├── main.rs          # 入口，命令分发（handler 是 tracing stub）
├── lib.rs           # 模块声明
├── cli.rs           # clap derive CLI
├── config.rs        # TOML + env 配置（默认 Anthropic API）
├── error.rs         # 错误层次: Error > {ConfigError, ToolError, AgentError, LlmError}
├── logging.rs       # tracing init(level)
├── llm/
│   ├── mod.rs       # 模块声明
│   ├── types.rs     # 共享类型 (Role, ContentBlock, InputMessage)
│   ├── request.rs   # 请求类型 (ChatRequest, SystemPrompt, ToolChoice, Thinking)
│   ├── response.rs  # 响应类型 (ChatResponse, StopReason, Usage, StreamEvent, Delta)
│   ├── error.rs     # API 错误 (ApiError)
│   └── client.rs    # LlmClient trait + ModelInfo
└── tools/
    ├── mod.rs       # Re-exports
    ├── traits.rs    # Tool trait
    └── call.rs      # ToolCall, ToolResult
```
