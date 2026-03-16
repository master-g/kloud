# TODO — 开发交接文档

> 本文件记录当前进度和下一步计划，方便跨会话恢复上下文。

## 当前位置

**Milestone 1.2 — REPL 交互循环**，M1.1 已完成（非流式 + 流式 API 调用均已验证）。

## 已完成的工作

### Bootstrap 阶段
- CLI 解析 (`src/cli.rs`) — clap derive，支持 run/exec/continue/resume/doctor/mcp/serve/config 子命令
- 配置管理 (`src/config.rs`) — TOML + 环境变量，默认指向 Anthropic API
- 错误体系 (`src/error.rs`) — thiserror 层次结构：Error > {ConfigError, ToolError, AgentError, LlmError}
- 日志 (`src/logging.rs`) — tracing，接受 level 参数
- 工具类型 (`src/tools/`) — Tool trait + ToolCall/ToolResult 类型
- 环境变量 (`src/env.rs`) — dotenvy 加载 `.env`

### M1.1 LLM 对话（已完成）

1. **LLM 模块类型定义** (`src/llm/`)
   - `types.rs` — Role, CacheControl, ContentBlock, InputMessage
   - `request.rs` — ChatRequest, SystemPrompt, Thinking, ToolChoice
   - `response.rs` — ChatResponse, StopReason, Usage, StreamEvent, Delta
   - `error.rs` — ApiError, ClientError
   - `client.rs` — LlmClient trait (chat + chat_stream), ModelInfo

2. **AnthropicClient** (`src/llm/anthropic.rs`)
   - Builder 模式构建，构建时 fail-fast 校验（api_key、base_url、model 必填）
   - `reqwest::Client` 非 Option，`build()` 时兜底创建，复用连接池
   - URL 用 `url::Url` 安全拼接（`pop_if_empty` + `extend`）
   - Builder 方法使用 `impl Into<String>` 签名

3. **非流式 `chat()`**
   - 正确的 Anthropic header：`x-api-key` + `anthropic-version: 2023-06-01`
   - 使用 `.json(&request)` 自动序列化 + 设置 Content-Type
   - HTTP 状态码映射：401 → AuthFailed, 429 → RateLimited, 其他 → RequestFailed

4. **流式 `chat_stream()`**
   - SSE 帧解码器 (`src/llm/sse.rs`) — `tokio_util::codec::Decoder`，以 `\n\n` 切分帧
   - 转换链：`bytes_stream → StreamReader → FramedRead → filter_map`
   - 解决了 TCP chunk 与 SSE 帧不对齐的问题

5. **Serde 序列化修复**
   - `SystemPrompt` 加 `#[serde(untagged)]`（API 期望裸字符串或裸数组）
   - `ChatRequest` 的 Option 字段加 `#[serde(skip_serializing_if = "Option::is_none")]`

6. **错误层级完善**
   - `ClientError` — 构建时错误（BadArgument, Url）
   - `LlmError` 新增 — Serde, Reqwest, Url, StreamError（`#[from]` 自动转换）

7. **Examples**
   - `examples/echo.rs` — 非流式调用示例
   - `examples/streaming.rs` — 流式调用示例
   - 已通过真实 API（minimax 兼容端点）验证

### 关键设计决策记录
- **ContentBlock 用 `#[serde(tag = "type")]`** — 内部标记模式
- **SystemPrompt 用 `#[serde(untagged)]`** — API 期望无 tag 的多态
- **纯数据类型用 `#[allow(missing_docs)]`** — trait 保留文档要求
- **SSE 解析用 Codec 模式** — `Decoder` trait 比 BufReader + LinesStream 更贴合帧协议
- **`ApiError` 在 `llm/error.rs`**（协议层），`LlmError` 在 `src/error.rs`（客户端层）

## 下一步：M1.2 REPL 交互循环

### 要做什么
- 实现 stdin 读取用户输入的循环
- 将用户输入发送给 LLM，流式打印回复
- 维护 `Vec<InputMessage>` 对话历史，每次调用发送完整历史
- 处理退出命令 (`/exit`, `Ctrl+C`)
- 集成到 `main.rs` 的 `Run` 命令

### 要学的 Rust 知识
- `tokio::io::stdin` 或 `std::io::stdin` + `spawn_blocking`
- `tokio::signal::ctrl_c()` 信号处理
- 生命周期：LLM client 在循环中被借用

### 要学的 Agent 知识
- REPL 模式 vs 单次执行（对应 CLI 的 `run` 和 `exec`）
- 对话历史：每次调用必须发送完整 `messages[]`

### 验收标准
- `cargo run` 能进入交互模式，输入问题，LLM 流式回复，Ctrl+C 退出

## 之后的路

M1.2 REPL → M2 工具框架 → M3 Agent Loop

完整路线图见 `docs/ROADMAP.md`。

## 文件结构快照

```
src/
├── main.rs          # 入口，命令分发（handler 是 tracing stub）
├── lib.rs           # 模块声明
├── cli.rs           # clap derive CLI
├── config.rs        # TOML + env 配置（默认 Anthropic API）
├── env.rs           # dotenvy 环境变量加载
├── error.rs         # 错误层次: Error > {ConfigError, ToolError, AgentError, LlmError}
├── logging.rs       # tracing init(level)
├── llm/
│   ├── mod.rs       # 模块声明
│   ├── types.rs     # 共享类型 (Role, ContentBlock, InputMessage)
│   ├── request.rs   # 请求类型 (ChatRequest, SystemPrompt, ToolChoice, Thinking)
│   ├── response.rs  # 响应类型 (ChatResponse, StopReason, Usage, StreamEvent, Delta)
│   ├── error.rs     # API/Client 错误 (ApiError, ClientError)
│   ├── client.rs    # LlmClient trait + ModelInfo
│   ├── anthropic.rs # AnthropicClient 实现（Builder + chat + chat_stream）
│   └── sse.rs       # SSE 帧解码器 (SseDecoder: tokio_util::codec::Decoder)
└── tools/
    ├── mod.rs       # Re-exports
    ├── traits.rs    # Tool trait
    └── call.rs      # ToolCall, ToolResult

examples/
├── echo.rs          # 非流式调用示例
└── streaming.rs     # 流式调用示例
```
