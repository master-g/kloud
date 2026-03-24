# TODO — 开发交接文档

> 本文件记录当前进度和下一步计划，方便跨会话恢复上下文。

## 当前位置

**Milestone 2.2 已明显推进，且已经打通了最小可用的 tool-use 闭环；整体处于 M2.2 向 M3 过渡的阶段。**

- M1 已全部完成（LLM 对话 + REPL 交互循环）
- M2.1 工具框架已完成并接入运行时
- M2.2 已完成 `ReadTool` 第一版
- `Session` 已经可以：
  - 向模型声明可用工具
  - 流式收集 `tool_use`
  - 执行工具并把 `tool_result` 回灌给模型
  - 在 `stop_reason == tool_use` 时继续下一轮模型请求
- 当前下一步建议：继续补文件工具（优先 `WriteTool`），并补更高层的 session/tool-loop 集成测试

## 已完成的工作

### Bootstrap 阶段
- CLI 解析 (`src/cli.rs`) — clap derive，支持 run/exec/continue/resume/doctor/mcp/serve/config 子命令
- 配置管理 (`src/config.rs`) — TOML + 环境变量，默认指向 Anthropic API
- 错误体系 (`src/error.rs`) — thiserror 层次结构：`Error > {ConfigError, ToolError, AgentError, LlmError, UiError}`
- 日志 (`src/logging.rs`) — tracing，接受 level 参数
- 环境变量 (`src/env.rs`) — dotenvy 加载 `.env`

### M1.1 LLM 对话（已完成）

1. **LLM 模块类型定义** (`src/llm/`)
   - `types.rs` — `Role`, `CacheControl`, `ContentBlock`, `InputMessage`
   - `request.rs` — `ChatRequest`, `SystemPrompt`, `Thinking`, `ToolChoice`
   - `response.rs` — `ChatResponse`, `StopReason`, `Usage`, `StreamEvent`, `Delta`
   - `error.rs` — `ApiError`, `ClientError`
   - `client.rs` — `LlmClient` trait (`chat` + `chat_stream`), `ModelInfo`

2. **AnthropicClient** (`src/llm/anthropic.rs`)
   - Builder 模式构建，构建时 fail-fast 校验（`api_key`、`base_url`、`model` 必填）
   - `reqwest::Client` 非 `Option`，`build()` 时兜底创建，复用连接池
   - URL 用 `url::Url` 安全拼接（`pop_if_empty` + `extend`）
   - Builder 方法使用 `impl Into<String>` 签名

3. **非流式 `chat()`**
   - 正确的 Anthropic header：`x-api-key` + `anthropic-version: 2023-06-01`
   - 使用 `.json(&request)` 自动序列化 + 设置 Content-Type
   - HTTP 状态码映射：`401 -> AuthFailed`, `429 -> RateLimited`, 其他 -> `RequestFailed`

4. **流式 `chat_stream()`**
   - SSE 帧解码器 (`src/llm/sse.rs`) — `tokio_util::codec::Decoder`，以 `\n\n` 切分帧
   - 转换链：`bytes_stream -> StreamReader -> FramedRead -> filter_map`
   - 已解决 TCP chunk 与 SSE 帧不对齐的问题

5. **Serde 序列化修复**
   - `SystemPrompt` 加 `#[serde(untagged)]`
   - `ChatRequest` 的 `Option` 字段加 `#[serde(skip_serializing_if = "Option::is_none")]`

6. **错误层级完善**
   - `ClientError` — 构建时错误（`BadArgument`, `Url`）
   - `LlmError` — `Serde`, `Reqwest`, `Url`, `StreamError`

7. **Examples**
   - `examples/echo.rs` — 非流式调用示例
   - `examples/streaming.rs` — 流式调用示例

### M1.2 REPL 交互循环（已完成）

1. **UI 抽象层** (`src/ui/`)
   - `events.rs` — `AppEvent`（Session→UI）、`UiAction`（UI→Session）消息协议
   - `backend.rs` — `UiBackend` trait、`UiChannels`、`UiHandle`、`create_ui_channels()`
   - 三 Actor 模型：UI Task ↔ mpsc channels ↔ Session

2. **StdioBackend** (`src/ui/stdio.rs`)
   - println-based fallback REPL
   - 异步 stdin 读取 + 事件显示循环

3. **RatatuiBackend** (`src/ui/tui/`)
   - `mod.rs` — `tokio::select!` 事件循环（键盘 + app 事件 + 渲染定时器 20 FPS）
   - `state.rs` — `TuiState`, `DisplayMessage`, `DisplayBlock`
   - `widgets.rs` — 三面板布局（messages + input + status bar）
   - `input.rs` — crossterm 键盘事件 → `UiAction` 映射，Unicode-safe 光标移动
   - `TerminalGuard` — RAII Drop-based 终端恢复（panic-safe）

4. **Session 主循环**
   - `src/app/session/run.rs` — 外层 `run()` 循环 + slash command
   - `src/app/session/stream.rs` — 流式 poll/action/event 处理
   - `src/app/session/tools.rs` — tool-use continuation / dispatch
   - `src/app/session/types.rs` — `TurnState` 与 pending block 类型

5. **main.rs 接入**
   - `run_interactive()` 已构建 client / tool registry / channels / Session / TUI
   - 无子命令时默认进入交互模式
   - API key 优先级：config > `ANTHROPIC_API_KEY` env var

### M2.1 工具框架（已完成）

1. **`Tool` trait**
   - `src/tools/traits.rs` 提供 `name()` / `description()` / `input_schema()` / `execute()`

2. **`ToolRegistry`** (`src/tools/registry.rs`)
   - `new()` / `register()` / `get()` / `list_names()`
   - `dispatch(&ToolCall)`：按工具名查找并执行
   - `definitions()`：导出给 LLM 的工具声明（`name` / `description` / `input_schema`）
   - 未知工具返回 `ToolError::NotFound`

3. **路径安全 helper** (`src/tools/path.rs`)
   - `resolve_existing_path(root, user_path)`
   - 仅接受相对路径
   - `canonicalize()` + `starts_with(root)` 防目录穿越
   - 绝对路径和越界路径返回 `ToolError::PathSecurity`

4. **builtin 工具注册入口** (`src/tools/builtin/mod.rs`)
   - `create_builtin_tools_registry(root)` 注册 `EchoTool` 和 `ReadTool`

5. **最小 builtin tool**
   - `EchoTool` 已实现并带基础测试

### M2.2 文件工具 + 最小 tool loop（进行中）

1. **`ReadTool` 第一版完成** (`src/tools/builtin/read.rs`)
   - 构造时持有 `root: PathBuf`
   - 输入 schema：`path` + 可选 `offset` / `limit`
   - `offset` 为 0-based，输出行号为 1-based
   - 通过 `resolve_existing_path()` 限制访问范围
   - 输出为带行号的纯文本字符串（非结构化 JSON）
   - `limit` 缺省或传 0 时默认最多读取 2000 行
   - 使用 `FramedRead + LinesCodec` 逐行读取，并限制单行最大长度 1024 字节

2. **`ReadTool` 已覆盖的测试**
   - 正常读取带 `offset/limit` 的文件内容
   - 缺失 `path` 参数
   - 非存在文件
   - 路径逃逸（`../outside.txt`）

3. **工具声明已接入模型请求**
   - `Session` 会通过 `ToolRegistry::definitions()` 构建 `ChatRequest.tools`
   - 当前默认 `ToolChoice::Auto { disable_parallel_tool_use: true }`

4. **最小 tool-use 闭环已接通**
   - assistant `tool_use` block 会在流式阶段被重建
   - `Session` 在 `StopReason::ToolUse` 时会：
     - 把 assistant message 写回历史
     - 提取 tool call
     - 执行工具
     - 把 `ToolResult` 作为后续消息写回历史
     - 再次调用 `start_model_stream()`

5. **`Session` 状态机重构**
   - 旧的单文件 `src/app/session.rs` 已拆为目录模块：
     - `src/app/session/mod.rs`
     - `src/app/session/types.rs`
     - `src/app/session/run.rs`
     - `src/app/session/stream.rs`
     - `src/app/session/tools.rs`
   - 当前文件体量：
     - `mod.rs`: 76 行
     - `types.rs`: 145 行
     - `run.rs`: 127 行
     - `stream.rs`: 714 行
     - `tools.rs`: 494 行

6. **当前 session/tool-loop 测试覆盖**
   - `execute_tool_call_wraps_dispatch_error_as_tool_result`
   - `continue_after_tool_use_dispatches_tool_and_starts_next_stream`
   - `continue_after_tool_use_wraps_dispatch_error_and_starts_next_stream`
   - `continue_after_end_turn_returns_idle_without_starting_new_stream`
   - `finalize_pending_block_tool_use_falls_back_to_start_input`
   - `finalize_pending_block_tool_use_reports_parse_error`
   - `handle_stream_event_text_block_lifecycle`
   - `handle_stream_event_tool_use_block_lifecycle_with_input_json_delta`
   - `handle_stream_event_thinking_block_lifecycle_with_signature`

## 当前关键设计决策

- `ToolRegistry::new()` 保持纯净，不隐式读取 `pwd`
- builtin tool 的组装逻辑放在 `src/tools/builtin/mod.rs`
- root 由上层（应用入口 / session 初始化）决定，再传给 builtin registry
- `ToolRegistry::definitions()` 是 LLM 声明工具的唯一出口
- tool discovery / dispatch 失败走 `crate::Error`
- tool 执行失败时，`Session::execute_tool_call()` 不会直接中断 loop，而是包装成 `ContentBlock::ToolResult { is_error: Some(true) }`
- `ReadTool` 当前是 **text-first tool**，输出面向 LLM/用户可读文本，不做结构化 JSON 输出
- tool loop 当前默认禁用 parallel tool use，先把单工具闭环做稳

## 下一步建议

### 优先建议
1. **实现 `WriteTool`**
   - 复用路径安全边界
   - 与 `ReadTool` 对称
   - 是当前 M2.2 最自然的下一步

2. **补更高层的 `Session::run()` / UI 通道集成测试**
   - 当前已有 stream/tool helper 层测试
   - 还缺一条真正从 `UiAction::SendMessage` 进入 `run()` 的高层测试

3. **实现真正的 turn cancellation**
   - 当前 `CancelTurn` 只是状态机层面返回 Idle
   - 还没有实际中断底层 stream
   - 需要 `CancellationToken`

### 暂未做的事情
- `WriteTool` / `EditTool` 还未开始
- `continue` / `resume` 仍然只是 CLI 占位，没有会话持久化
- 没有真正的 tool parallelism
- 没有 stream-level cancellation
- 还没有把 tool-use 闭环放到完整 e2e REPL 路径里验证

### 已知技术债 / 观察
- `messages.clone()` 每次请求克隆完整历史 — M4 上下文管理阶段再解决
- `stream.rs` 和 `tools.rs` 虽然已拆出，但仍然偏大，后续还可继续细分
- `UiError` 变体已定义但未实际接入
- 输入仍不支持多行（未来可考虑 `Shift+Enter`）
- Slash command 解析在 stdio / tui 中仍有一定重复

## 之后的路

M2.2（补更多文件工具） → 巩固最小 tool loop → M3（更完整的 Agent Loop / 状态机语义） → M4（上下文管理）

完整路线图见 `docs/ROADMAP.md`。
