## Why

kloud 的 tool call 和代码块渲染与原版 CC 严重偏离。核心问题：kloud 对所有工具内容使用 `╭╮╰╯` 全包围圆角边框，而 CC 使用 `⎿` (U+23BF) 左缩进指示器。这导致视觉密度过高，失去了 CC 简洁干净的终端 UI 风格。需要严格对齐 CC 的渲染模式。

## What Changes

- **Tool Use 渲染**：移除 `╭─ ⏺ name ───╮` 全包围边框，改为 `⏺ name` 单行渲染
- **Tool Result 渲染**：移除 `╭───╮ │ content │ ╰───╯` 全包围边框，改为 `⎿ content` 左缩进模式
- **代码块渲染**：移除 `┌─ lang` / `└─` 上下边框，改为行号 gutter（CC 风格右对齐行号）
- **Bash 工具**：移除粉色边框特殊处理，与其他工具使用相同渲染风格
- **边框辅助函数**：移除 `wrap_in_border()` / `wrap_in_border_styled()`，替换为缩进辅助函数

## Capabilities

### New Capabilities
- `cc-tool-rendering`: Tool use 和 tool result 的 CC 风格渲染（`⏺ name` 单行 + `⎿` 左缩进结果）
- `cc-code-gutter`: 消息中代码块的行号 gutter 渲染（右对齐行号 + 语法高亮，无边框）

### Modified Capabilities

## Impact

- `src/ui/tui/widgets/tool_block.rs` — 核心重写，移除所有边框逻辑
- `src/ui/tui/widgets/messages.rs` — 修改代码块渲染 + 调用方参数更新
- `src/ui/tui/theme.rs` — 移除 `bash_border` 字段
- `src/ui/tui/constants.rs` — 添加 `⎿` 常量
- 现有测试需更新（边框断言改为缩进断言）
