## Why

`align-tool-rendering` 重构后引入了视觉 bug：工具名在 TUI 中渲染了两次。`render_tool_use_line()` 构建独立 header 行（`⏺ read`），而 `render_tool_use_message()` 返回的内容又包含 user-facing name（`Read TODO.md`），导致输出显示两行重复名称信息。CC 的实际行为是 dot + name + args 在同一行。

## What Changes

- 修改 `render_tool_use_line()` 布局逻辑：去掉独立 header 行，把 dot 前缀合并到 `rendered` 第一行
- 保留空 rendered 回退路径（MCP 工具 / fallback）
- 保留 server_name 前缀逻辑

## Capabilities

### New Capabilities

_(无)_

### Modified Capabilities

- `tui-tool-rendering`: 修正 tool use header 布局——dot 合并到 rendered 第一行，消除名称重复

## Impact

- `src/ui/tui/widgets/tool_block.rs` — `render_tool_use_line()` 函数重写
- 视觉影响：所有工具（Read、Write、Echo、未来工具）的 header 行数从 2 行减为 1 行
- 无 API/数据结构变更，纯布局层修改
