# TODO — 开发交接文档

> 本文件记录当前进度和下一步计划，方便跨会话恢复上下文。

## 当前位置

**Milestone 2.1 — 工具框架**，M1 已全部完成（LLM 对话 + REPL 交互循环）。

## 已完成的工作

### Bootstrap 阶段
- CLI 解析 (`src/cli.rs`) — clap derive，支持 run/exec/continue/resume/doctor/mcp/serve/config 子命令
- 配置管理 (`src/config.rs`) — TOML + 环境变量，默认指向 Anthropic API
- 错误体系 (`src/error.rs`) — thiserror 层次结构：Error > {ConfigError, ToolError, AgentError, LlmError, UiError}
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

### M1.2 REPL 交互循环（已完成）

1. **UI 抽象层** (`src/ui/`)
   - `events.rs` — AppEvent（Session→UI）、UiAction（UI→Session）消息协议
   - `backend.rs` — UiBackend trait（单 `run` 方法）、UiChannels、UiHandle、create_ui_channels()
   - 三 Actor 模型：UI Task ↔ mpsc channels ↔ Session

2. **StdioBackend** (`src/ui/stdio.rs`)
   - println-based fallback REPL，用于开发调试
   - 异步 stdin 读取 + 事件显示循环

3. **RatatuiBackend** (`src/ui/tui/`)
   - `mod.rs` — `tokio::select!` 事件循环（键盘 + app 事件 + 渲染定时器 20 FPS）
   - `state.rs` — TuiState, DisplayMessage, DisplayBlock（UI 领域数据类型）
   - `widgets.rs` — 三面板布局（messages + input + status bar）
   - `input.rs` — crossterm 键盘事件 → UiAction 映射，Unicode-safe 光标移动
   - TerminalGuard — RAII Drop-based 终端恢复（panic-safe）

4. **Session** (`src/app/session.rs`)
   - 持有 `Box<dyn LlmClient>` + `Vec<InputMessage>` + `UiHandle`
   - 核心循环：等 UiAction → 追加 user message → chat_stream() → 转发 StreamEvent 为 AppEvent → 追加 assistant message
   - `/help`、`/exit` slash commands
   - `let _ = send()` fire-and-forget 策略

5. **main.rs 接入**
   - `run_interactive()` 函数：构建 client → create_ui_channels → Session + RatatuiBackend → `tokio::join!`
   - 无子命令时默认进入交互模式
   - API key 优先级：config > ANTHROPIC_API_KEY env var

### 关键设计决策记录
- **ContentBlock 用 `#[serde(tag = "type")]`** — 内部标记模式
- **SystemPrompt 用 `#[serde(untagged)]`** — API 期望无 tag 的多态
- **纯数据类型用 `#[allow(missing_docs)]`** — trait 保留文档要求
- **SSE 解析用 Codec 模式** — `Decoder` trait 比 BufReader + LinesStream 更贴合帧协议
- **`ApiError` 在 `llm/error.rs`**（协议层），`LlmError` 在 `src/error.rs`（客户端层）
- **UiBackend 只有单个 `run` 方法** — channel 契约就是抽象，不强拆 render/input
- **Channel 容量不对称** — event 256（高频流式 delta）vs action 32（低频用户操作）
- **TerminalGuard RAII** — Drop 恢复终端，即使 panic 也不会卡 raw mode

## 下一步：M2.1 工具框架

### 要做什么
- 完善 `Tool` trait：加入 `input_schema()` 方法（返回 JSON Schema）
- 实现 `ToolRegistry`：注册、查找、列举工具
- 实现 `ToolDispatcher`：根据工具名分发调用
- 实现路径安全检查（防止目录穿越）

### 要学的 Rust 知识
- `dyn Trait` + `Box` — Registry 存储不同类型的 Tool 实现
- trait object vs generics — 运行时多态（工具类型在运行时确定）
- `serde_json::Value` — 工具参数是动态 JSON
- `Path` 安全处理 — canonicalize() + 前缀检查

### 要学的 Agent 知识
- **Tool Use 协议**: LLM 返回 `tool_calls`，Agent 执行后把结果作为 `tool` role 消息发回
- **JSON Schema**: 工具需要 schema 让 LLM 知道参数格式
- **工具权限模型**: 路径安全检查防止 LLM 被注入恶意指令

### 已知技术债（M1.2 Code Review）
- `messages.clone()` 每次请求克隆完整历史 — M4 上下文管理解决
- `CancelTurn` 协议中存在但未实现流中断 — 需要 `CancellationToken`
- `BlockComplete` 始终报告 `BlockType::Text` — 需要从 `ContentBlockStart` 跟踪类型
- `UiError` 变体已定义但未使用 — 等有实际 use case 再接入
- 输入不支持多行 — 未来可加 Shift+Enter
- Slash command 解析在 stdio 和 tui 中重复 — 可提取公共函数

### 验收标准
- `cargo test` 能注册工具、按名查找、dispatch 调用

## 之后的路

M2 工具框架 → M3 Agent Loop → M4 上下文管理

完整路线图见 `docs/ROADMAP.md`。

## 文件结构快照

```
src/
├── main.rs          # 入口，命令分发，run_interactive() 接入 Session + UI
├── lib.rs           # 模块声明
├── cli.rs           # clap derive CLI
├── config.rs        # TOML + env 配置（默认 Anthropic API）
├── env.rs           # dotenvy 环境变量加载
├── error.rs         # 错误层次: Error > {ConfigError, ToolError, AgentError, LlmError, UiError}
├── logging.rs       # tracing init(level)
├── app/
│   ├── mod.rs       # re-exports
│   └── session.rs   # Session: 对话循环驱动器 (LlmClient + 历史 + UiHandle)
├── llm/
│   ├── mod.rs       # 模块声明
│   ├── types.rs     # 共享类型 (Role, ContentBlock, InputMessage)
│   ├── request.rs   # 请求类型 (ChatRequest, SystemPrompt, ToolChoice, Thinking)
│   ├── response.rs  # 响应类型 (ChatResponse, StopReason, Usage, StreamEvent, Delta)
│   ├── error.rs     # API/Client 错误 (ApiError, ClientError)
│   ├── client.rs    # LlmClient trait + ModelInfo
│   ├── anthropic.rs # AnthropicClient 实现（Builder + chat + chat_stream）
│   └── sse.rs       # SSE 帧解码器 (SseDecoder: tokio_util::codec::Decoder)
├── ui/
│   ├── mod.rs       # re-exports, create_ui_channels()
│   ├── events.rs    # AppEvent, UiAction 消息协议
│   ├── backend.rs   # UiBackend trait, UiChannels, UiHandle
│   ├── stdio.rs     # StdioBackend (println fallback)
│   └── tui/
│       ├── mod.rs   # RatatuiBackend (tokio::select! 事件循环)
│       ├── state.rs # TuiState, DisplayMessage, DisplayBlock
│       ├── widgets.rs # render 函数 (messages, input, status bar)
│       └── input.rs # crossterm 键盘事件 → UiAction 映射
└── tools/
    ├── mod.rs       # Re-exports
    ├── traits.rs    # Tool trait
    └── call.rs      # ToolCall, ToolResult

examples/
├── echo.rs          # 非流式调用示例
└── streaming.rs     # 流式调用示例
```
