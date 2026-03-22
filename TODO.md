# TODO — 开发交接文档

> 本文件记录当前进度和下一步计划，方便跨会话恢复上下文。

## 当前位置

**Milestone 2.2 — 文件工具（ReadTool）已完成第一版**。

- M1 已全部完成（LLM 对话 + REPL 交互循环）
- M2.1 工具框架已基本完成
- M2.2 当前已完成第一个真实文件工具：`ReadTool`
- 下一步建议：先把 builtin registry 接入应用初始化或 Session，再继续实现 `WriteTool`

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

### M2.1 工具框架（基本完成）

1. **`Tool` trait 完善**
   - `src/tools/traits.rs` 新增 `input_schema() -> serde_json::Value`

2. **`ToolRegistry` 基础能力完成** (`src/tools/registry.rs`)
   - `new()` / `register()` / `get()` / `list_names()`
   - `dispatch(&ToolCall)`：按工具名查找并执行
   - 未知工具返回 `ToolError::NotFound`

3. **路径安全 helper 完成** (`src/tools/path.rs`)
   - `resolve_existing_path(root, user_path)`
   - 仅接受相对路径
   - `canonicalize()` + `starts_with(root)` 防目录穿越
   - 绝对路径和越界路径返回 `ToolError::PathSecurity`

4. **builtin 工具注册入口** (`src/tools/builtin/mod.rs`)
   - `create_builtin_tools_registry(root)` 会注册 `EchoTool` 和 `ReadTool`
   - 设计上由上层决定 root，再传给 builtin registry 构造逻辑

5. **最小 builtin tool**
   - `EchoTool` (`src/tools/builtin/echo.rs`) 已实现并带基础测试

### M2.2 文件工具（进行中）

1. **`ReadTool` 第一版完成** (`src/tools/builtin/read.rs`)
   - 构造时持有 `root: PathBuf`
   - 输入 schema：`path` + 可选 `offset` / `limit`
   - `offset` 为 0-based，输出行号为 1-based
   - 通过 `resolve_existing_path()` 限制访问范围
   - 输出为带行号的纯文本字符串（而非 JSON）
   - `limit` 缺省或传 0 时，默认最多读取 2000 行
   - 使用 `FramedRead + LinesCodec` 逐行读取，并限制单行最大长度 1024 字节

2. **`ReadTool` 已覆盖的测试**
   - 正常读取带 offset/limit 的文件内容
   - 缺失 `path` 参数
   - 非存在文件
   - 路径逃逸（`../outside.txt`）

### 当前关键设计决策
- `ToolRegistry::new()` 保持纯净，不隐式读取 `pwd`
- builtin tool 的组装逻辑放在 `src/tools/builtin/mod.rs`
- root 由上层（应用入口 / session 初始化）决定，再传给 builtin registry
- tool discovery / dispatch 失败走 `crate::Error`
- tool 内部业务失败（路径不合法、文件打不开、参数缺失）走 `ToolResult.output = Err(String)`
- `ReadTool` 当前是 **text-first tool**，输出面向 LLM/用户可读文本，不做结构化 JSON 输出

## 下一步建议

### 优先建议
1. **把 builtin registry 接入应用初始化**
   - 在 `main.rs` 或 Session 初始化处获取工作目录 root
   - 调用 `create_builtin_tools_registry(root)`
   - 让已实现的 builtin tools 真正进入运行时

2. **决定工具声明如何进入 LLM 请求**
   - 当前 `src/app/session.rs` 里 `tool_choice: None`、`tools: None`
   - 下一步需要把 registry 里的工具 schema 转成请求中的 `tools`

3. **再继续实现 `WriteTool`**
   - 可以复用路径安全边界
   - 与 `ReadTool` 对称，是 M2.2 的自然下一步

### 暂未做的事情
- builtin registry 还没接入 `main.rs` / `Session`
- Session 还不会真正执行工具闭环（目前只展示 tool_use 相关 UI 事件）
- `WriteTool` / `EditTool` 还未开始
- `ToolRegistry` 还没有导出“列出所有工具 schema”的辅助接口

### 已知技术债 / 观察
- `messages.clone()` 每次请求克隆完整历史 — M4 上下文管理解决
- `CancelTurn` 协议中存在但未实现流中断 — 需要 `CancellationToken`
- `BlockComplete` 始终报告 `BlockType::Text` — 需要从 `ContentBlockStart` 跟踪类型
- `UiError` 变体已定义但未使用 — 等有实际 use case 再接入
- 输入不支持多行 — 未来可加 Shift+Enter
- Slash command 解析在 stdio 和 tui 中重复 — 可提取公共函数
- `ReadTool` 的测试目前够用，但还没有 registry 层 dispatch `read` 的集成测试

## 之后的路

M2.2（接入 builtin registry / 实现更多文件工具） → M3 Agent Loop → M4 上下文管理

完整路线图见 `docs/ROADMAP.md`。
