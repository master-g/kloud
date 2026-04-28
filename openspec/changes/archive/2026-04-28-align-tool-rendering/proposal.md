## Why

kloud 的工具渲染完全由通用函数 `tool_block.rs` 处理，无法按工具类型差异化。CC 中每个 Tool 自带 `userFacingName`、`renderToolUseMessage`、`renderToolResultMessage` 等渲染方法，使不同工具（bash、文件编辑、搜索）呈现各自风格。当前 kloud 缺少：(1) per-tool 渲染接口；(2) 工具结果的多级路由（cancel/reject/error/success）；(3) 工具执行进度消息；(4) input 结构化预览。对齐后用户体验将显著接近 CC。

## What Changes

- **Tool trait 扩展**：新增 `user_facing_name()`、`render_tool_use_message()`、`render_tool_result_message()` 方法，由每个 builtin tool 实现自定义渲染
- **ToolResult 状态丰富化**：`ToolResult` 增加 `ToolResultKind` 枚举（Success / Error / Canceled / Rejected），取代当前 `Result<String, String>`
- **错误路由**：`tool_block.rs` 根据 `ToolResultKind` 分发到不同渲染路径（cancel 静默、reject 说明原因、error 红色高亮）
- **Progress 消息管线**：Tool execute 通过 callback emit `ToolProgress`，TUI 在工具执行期间实时显示
- **input 结构化预览**：`render_tool_use_message()` 接收解析后的 input，生成工具特定的预览（bash 显示命令、read 显示路径、write 显示文件名）
- **预渲染架构**：Tool 渲染方法在 store 层调用（有 ToolRegistry 访问权），预计算 `Vec<Line<'static>>` 存入 `DisplayBlock`。TUI 层只做布局（状态点 + 折叠 + 前缀），不接触 ToolRegistry

## Capabilities

### New Capabilities
- `tool-rendering`: per-tool 渲染接口与实现，覆盖 tool_use 行、tool_result 块、progress 消息的差异化渲染

### Modified Capabilities
- `tool-protocol`: Tool trait 新增渲染方法签名，ToolResult 引入状态枚举
- `tui-tool-rendering`: tool_block.rs 从渲染实现者变为纯布局层，消费预渲染内容

## Impact

- `src/tools/traits.rs` — Tool trait 扩展（新增渲染方法带默认实现）
- `src/tools/call.rs` — ToolResult / ToolResultKind 类型变更
- `src/tools/builtin/*.rs` — 每个 builtin tool 实现新渲染方法
- `src/agent/message.rs` — DisplayBlock 新增预渲染字段 + ToolResultKind，去掉 PartialEq derive
- `src/agent/session_event.rs` — 新增 ToolProgress 事件，事件携带预渲染内容
- `src/agent/store.rs` — apply() 中存入预渲染内容，处理 ToolProgress
- `src/app/session/tools.rs` — emit 事件前调用 Tool 渲染方法，传递 progress callback
- `src/ui/tui/widgets/tool_block.rs` — 接收预渲染 Lines，只做布局
- `src/ui/tui/widgets/messages.rs` — 消息渲染适配新 DisplayBlock 字段
