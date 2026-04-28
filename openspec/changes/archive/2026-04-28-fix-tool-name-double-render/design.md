## Context

`align-tool-rendering` 重构引入了 Tool trait 预渲染架构：`render_tool_use_message()` 返回含 `user_facing_name` 的 `Vec<Line>`，存储在 `DisplayBlock::ToolUse.rendered_use`。但 `tool_block.rs` 的 `render_tool_use_line()` 仍保留旧的独立 header 行逻辑——自己构建 `dot + name` 行，然后追加 rendered 行。两者叠加导致工具名显示两次。

当前渲染流程：
```
render_tool_use_line(name="read", rendered=[Line("Read TODO.md")])
  → line 1: "⏺ read"           ← 独立 header，用 raw name
  → line 2: "  Read TODO.md"   ← rendered 内容，含 user_facing_name
```

CC 的实际渲染：
```
  → line 1: "⏺ Read TODO.md"  ← 单行，dot + rendered 内容
```

## Goals / Non-Goals

**Goals:**
- 消除工具名双重渲染
- 使 tool use header 与 CC 的单行布局一致
- 保留 MCP server_name 前缀功能
- 保留空 rendered 的回退路径

**Non-Goals:**
- 不修改 Tool trait 或 render_tool_use_message() 签名
- 不修改 DisplayBlock 数据结构
- 不修改 store 层预渲染逻辑

## Decisions

### Decision 1: 修改 `render_tool_use_line()` 合并 dot 到第一行

**选择：** 去掉独立 header 行，dot 前缀合并到 rendered 第一行的 spans 前面。

**理由：** Tool trait 的 `render_tool_use_message()` 已返回完整显示内容（name + args）。布局层只负责加 dot 前缀——符合 CC 的分层设计。

**替代方案：** 修改 `render_tool_use_message()` 去掉 user_facing_name，让 header 行由布局层构建。
**否决理由：** 违背预渲染架构——布局层不应知道工具的显示名。且 `name` 参数是 raw identifier（"read"），不是 user-facing name（"Read"）。

### Decision 2: 空 rendered 时回退到 dot + raw name

**选择：** `rendered.is_empty()` 时保持当前行为（dot + display_name），确保 MCP 工具和无注册工具的 fallback。

## Risks / Trade-offs

- **server_name 拼接位置变化** → 从 header 行移到 rendered 第一行内部，视觉一致（都在 dot 后面）
- **空 rendered 工具** → 使用 raw name（"read" 而非 "Read"），可接受因为这只出现在未注册工具场景
