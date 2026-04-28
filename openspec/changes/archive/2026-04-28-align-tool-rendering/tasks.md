## 1. ToolResult 状态枚举

- [x] 1.1 在 `src/tools/call.rs` 新增 `ToolResultKind` 枚举（Success / Error / Canceled / Rejected），修改 `ToolResult` 结构体：`name: String` 保留，新增 `kind: ToolResultKind`，`output: String`（扁平化，去掉 `Result` 包裹）
- [x] 1.2 更新 `src/tools/builtin/echo.rs` 的 `execute()` 返回新格式：`Ok(ToolResult { name, kind: Success, output: msg })` / `Ok(ToolResult { name, kind: Error, output: "missing..." })`
- [x] 1.3 更新 `src/tools/builtin/read.rs` 的 `execute()` 所有返回点适配新 `ToolResult`
- [x] 1.4 更新 `src/tools/builtin/write.rs` 的 `execute()` 所有返回点适配新 `ToolResult`
- [x] 1.5 更新 `src/app/session/tools.rs` 的 `execute_tool_call()`：解构 `result.kind` 和 `result.output`，构建 `ContentBlock::ToolResult` 时映射 `ToolResultKind::Error → is_error: true`，其余 `→ is_error: false`
- [x] 1.6 更新 `src/tools/registry.rs` 测试中 `assert_eq!(result.output, Ok(...))` 为 `assert_eq!(result.kind, ToolResultKind::Success)` + `assert_eq!(result.output, "...")`

## 2. Tool trait 渲染方法

- [x] 2.1 在 `src/tools/traits.rs` 的 `Tool` trait 添加三个带默认实现的方法：
  - `fn user_facing_name(&self) -> String` — 默认返回 `self.name().to_string()`
  - `fn render_tool_use_message(&self, input: &serde_json::Value, theme: &Theme) -> Vec<Line<'static>>` — 默认显示 name + truncated JSON
  - `fn render_tool_result_message(&self, output: &str, kind: ToolResultKind, theme: &Theme) -> Vec<Line<'static>>` — 默认 `⎿` + output
  需要在 `traits.rs` 引入 `ratatui::text::Line` 和 `crate::ui::tui::theme::Theme`
- [x] 2.2 为 `EchoTool` override `user_facing_name()` 返回 `"Echo"`，override `render_tool_use_message()` 显示 `message` 字段内容
- [x] 2.3 为 `ReadTool` override `user_facing_name()` 返回 `"Read"`，override `render_tool_use_message()` 显示 `path` 字段（file 样式），override `render_tool_result_message()` 保持行号格式输出
- [x] 2.4 为 `WriteTool` override `user_facing_name()` 返回 `"Write"`，override `render_tool_use_message()` 显示 `path` 字段（file 样式）

## 3. DisplayBlock 预渲染字段

- [x] 3.1 在 `src/agent/message.rs` 的 `DisplayBlock::ToolUse` 新增 `rendered_use: Vec<ratatui::text::Line<'static>>` 字段，`DisplayBlock::ToolResult` 新增 `rendered_result: Vec<ratatui::text::Line<'static>>` 和 `kind: ToolResultKind` 字段（替换 `is_error: bool`）
- [x] 3.2 更新 `DisplayBlock` 的 `PartialEq`/`Eq` derive：`Line<'static>` 不 impl `PartialEq`，改用手动 impl 或 `#[derive(Debug, Clone)]` + 去掉 `PartialEq`
- [x] 3.3 更新 `build_message_lookups()` 中 errored 判断：`matches!(kind, ToolResultKind::Error)` 替代 `is_error`
- [x] 3.4 更新 `src/agent/session_event.rs` 的 `ToolExecutionFinished`：新增 `result_kind: ToolResultKind` 和 `rendered_result: Vec<Line<'static>>` 字段，保留 `output: String` 供 LLM 使用
- [x] 3.5 更新 `src/agent/session_event.rs` 的 `AssistantToolUseStarted`：新增 `rendered_use: Vec<Line<'static>>` 字段

## 4. Store 层预渲染集成

- [x] 4.1 在 `src/app/session/tools.rs` 的 `continue_after_assistant_turn()` 中：`ToolExecutionStarted` emit 前调用 `registry.get(&name) → tool.render_tool_use_message(&input, &theme)` 生成 `rendered_use`，传入事件
- [x] 4.2 在 `src/app/session/tools.rs` 的 `execute_tool_call()` 返回后：调用 `registry.get(&name) → tool.render_tool_result_message(&output, kind, &theme)` 生成 `rendered_result`，传入 `ToolExecutionFinished` 事件
- [x] 4.3 Session 需要持有 `Theme` 引用或默认 Theme：在 `Session::new()` 中创建 `Theme::default()`，供渲染调用使用
- [x] 4.4 更新 `src/agent/store.rs` 的 `apply()` 中 `ToolExecutionStarted` 分支：将 `rendered_use` 存入 `DisplayBlock::ToolUse`
- [x] 4.5 更新 `src/agent/store.rs` 的 `apply()` 中 `ToolExecutionFinished` 分支：将 `rendered_result` 和 `kind` 存入 `DisplayBlock::ToolResult`
- [x] 4.6 更新 `src/agent/store.rs` 的 `append_tool_result()`：接收 `ToolResultKind` 替代 `is_error: bool`
- [x] 4.7 更新 `src/agent/store.rs` 测试中 `ToolExecutionFinished` 和 `DisplayBlock::ToolResult` 构造适配新字段

## 5. tool_block.rs 渲染重构

- [x] 5.1 修改 `render_tool_use_line()` 接收 `rendered: &[Line<'static>]` 替代 `input_preview: &str`，直接 append rendered lines 到输出（保留状态点渲染不变）
- [x] 5.2 修改 `render_tool_result_block()` 接收 `rendered: &[Line<'static>]` 和 `kind: ToolResultKind` 替代 `output: &str` + `is_error: bool`：
  - `Success` → 直接用 rendered lines，加 `⎿` 前缀和折叠
  - `Error` → 用 `theme.error` 样式渲染 rendered lines
  - `Canceled` → 渲染一行灰色 "Canceled" 指示
  - `Rejected` → 渲染一行灰色 "Rejected" 指示
- [x] 5.3 删除 `classify_tool()` 和 `ToolKind` 枚举（per-tool 样式已由 Tool trait 渲染方法处理）
- [x] 5.4 更新 `messages.rs` 中 `render_assistant_message()` 的 `DisplayBlock::ToolUse` 匹配：从 `tool_block` 提取 `rendered` 和 `status`，传入新签名
- [x] 5.5 更新 `messages.rs` 中 `render_assistant_message()` 的 `DisplayBlock::ToolResult` 匹配：从 `tool_block` 提取 `rendered` 和 `kind`，传入新签名

## 6. Progress 消息管线

- [x] 6.1 在 `src/agent/session_event.rs` 新增 `ToolProgress { id: String, text: String }` 事件
- [x] 6.2 修改 `Tool` trait 的 `execute()` 签名增加 `on_progress: Option<&dyn Fn(&str)>` 参数（现有调用点传 `None`）
- [x] 6.3 在 `src/app/session/tools.rs` 的 `execute_tool_call()` 中构造 progress 闭包，闭包内部通过 channel emit `SessionEvent::ToolProgress`（需 `ui_handle` 或 store sender）
- [x] 6.4 在 `src/agent/message.rs` 的 `DisplayBlock::ToolUse` 新增 `progress_text: Option<String>` 字段
- [x] 6.5 在 `src/agent/store.rs` 的 `apply()` 中处理 `ToolProgress` 事件：更新匹配 `DisplayBlock::ToolUse` 的 `progress_text`
- [x] 6.6 在 `messages.rs` 渲染 running 状态的 `ToolUse` 时：如果有 `progress_text`，在 rendered lines 后追加进度行

## 7. 测试更新

- [x] 7.1 更新 `src/tools/registry.rs` 测试适配 `ToolResult` 新格式
- [x] 7.2 更新 `src/tools/builtin/echo.rs` 测试适配 `ToolResult` 新格式
- [x] 7.3 更新 `src/tools/builtin/read.rs` 测试适配 `ToolResult` 新格式
- [x] 7.4 更新 `src/tools/builtin/write.rs` 测试适配 `ToolResult` 新格式
- [x] 7.5 更新 `src/app/session/tools.rs` 测试适配新事件字段和 `DisplayBlock` 结构

## 8. 编译验证

- [x] 8.1 `cargo check` 通过
- [x] 8.2 `cargo clippy -- -W warnings` 通过
- [x] 8.3 `cargo fmt --all` 格式化
- [x] 8.4 `cargo test` 全部通过
