## 1. Tool Use 渲染重写

- [x] 1.1 移除 `render_tool_use_line()` 中的 `╭─ ⏺ name ───╮` 顶部边框，改为单行：`⏺ ToolName(input_summary)`
- [x] 1.2 渲染 input summary：取 `render_tool_use_message()` 返回的首行内容，以 `(summary)` 格式跟在 tool name 后（匹配 CC `AssistantToolUseMessage` 行为）
- [x] 1.3 移除 `╰───╯` 底部边框，移除 `wrap_in_border()` 对 tool use 内容的包裹
- [x] 1.4 将 tool use 内容（超出首行的部分）改为 2 空格缩进输出（无 `│` 左右包裹）
- [x] 1.5 移除 `border_style_for_tool()` 函数和 `server_name` 拼接逻辑
- [x] 1.6 保留状态 dot 颜色和 blink 动画逻辑不变

## 2. Tool Result 渲染重写

- [x] 2.1 移除 `render_tool_result_block()` 中的 `╭───╮` / `╰───╯` 全包围边框
- [x] 2.2 实现 `⎿` (U+23BF) 左缩进指示器，使用 `theme.subtle` dim 样式
- [x] 2.3 重写 Canceled / Rejected 渲染：`⎿ Canceled` / `⎿ Rejected`（无边框）
- [x] 2.4 保留 5 行折叠逻辑，折叠提示改为 `⎿ ... N more lines (Tab to expand)`
- [x] 2.5 diff 高亮输出改为 `⎿` 左缩进包裹（替代边框包裹）
- [x] 2.6 移除 `is_bash` 参数和粉色边框逻辑
- [x] 2.7 移除 `wrap_in_border()` / `wrap_in_border_styled()` 函数

## 3. 代码块渲染重写

- [x] 3.1 移除 `┌─ code` / `┌─ lang` 顶部标记和 `└─` 底部标记
- [x] 3.2 实现行号 gutter：计算总行数确定 gutter 宽度（1/2/3 位）
- [x] 3.3 每行代码前缀：右对齐行号 + 2 空格，使用 `theme.subtle` 样式
- [x] 3.4 语言标签渲染为 dim 单独行（代码块内容前）
- [x] 3.5 保留 syntect `highlight_line()` 调用和 `is_language_supported()` 逻辑
- [x] 3.6 无语言代码块：只有行号 gutter，无语言标签行

## 4. 主题和常量清理

- [x] 4.1 从 `Theme` struct 移除 `bash_border` 字段
- [x] 4.2 移除所有主题方案中的 `bash_border` 值定义
- [x] 4.3 在 `tool_block.rs` 或 `constants.rs` 添加 `⎿` 常量

## 5. 调用方更新和测试

- [x] 5.1 更新 `messages.rs` 中 `render_tool_result_block()` 调用：移除 `is_bash` 参数
- [x] 5.2 移除 `render_assistant_message()` 中的 `last_tool_name` 变量（仅用于 bash 判断）
- [x] 5.3 更新 `syntax_highlight_rust_code_block` 测试：移除 `┌─` / `└─` 断言
- [x] 5.4 更新 `syntax_highlight_no_language` 测试：移除 `code` 边框断言
- [x] 5.5 新增行号 gutter 测试：断言行号前缀和 gutter 宽度
- [x] 5.6 `cargo fmt --all && cargo clippy -- -W warnings` — 干净
- [x] 5.7 `cargo run --example tui_showcase` — 视觉验证：无边框、有行号、`⎿` 缩进
