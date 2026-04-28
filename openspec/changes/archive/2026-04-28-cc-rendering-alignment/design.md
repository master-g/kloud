## Context

kloud TUI 的 tool call 和代码块渲染使用全包围圆角边框（`╭╮╰╯`），而原版 Claude Code CLI 使用 `⎿` 左缩进指示器。审计 CC 源码发现：

- `AssistantToolUseMessage.tsx`：单行 `⏺ ToolName(input)` 无边框
- `MessageResponse.tsx`：`⎿` (U+23BF) 左缩进，dim 样式
- `HighlightedCode.tsx`：行号 gutter（右对齐行号 + 空格），无 `┌└` 边框
- `BashToolResultMessage.tsx`：同样使用 `MessageResponse` 包装，无特殊边框

kloud 当前 `tool_block.rs` 使用 `wrap_in_border()` / `wrap_in_border_styled()` 将所有内容包裹在 `╭╮╰╯` 中。`messages.rs` 中代码块使用 `┌─ lang` / `└─` 标记。

## Goals / Non-Goals

**Goals:**
- Tool use 渲染严格匹配 CC：`⏺ name` 单行
- Tool result 渲染严格匹配 CC：`⎿` 左缩进
- 代码块渲染严格匹配 CC：行号 gutter，无边框
- bash 工具与其他工具使用相同渲染风格
- 保留现有折叠逻辑和 diff 高亮功能

**Non-Goals:**
- 不改动 markdown 渲染（表格、列表、引用等）
- 不改动 thinking block 渲染
- 不改动 tool 状态 dot 动画逻辑
- 不改动虚拟滚动或布局
- 不实现 CC 的 `MessageResponse` context 嵌套去重（kloud 结构不同）

## Decisions

### 1. Tool Result 左缩进字符

**选择**: `⎿` (U+23BF, HORIZONTAL SCAN LINE-9)

**替代方案**: `│` (box drawing) — 更重，当前 kloud 已在用，但 CC 用 `⎿`
**理由**: 严格对齐 CC。`⎿` 更轻量，dim 样式下视觉干扰更小。

### 2. 代码块行号 gutter

**选择**: 右对齐行号 + 2 空格分隔（CC 的 `HighlightedCode.tsx` 模式）

```
  1  fn main() {
  2      println!("hello");
  3  }
```

**替代方案**: 无行号，只有缩进 — 信息密度低
**理由**: CC 全屏模式下行号 gutter 是标准行为。行号宽度由总行数决定（如 100+ 行用 3 位宽）。

### 3. 代码块语言标签

**选择**: dim 样式显示在代码块第一行前（单独一行）

```
rust
  1  fn main() {
  2      println!("hello");
  3  }
```

**理由**: CC 的 `HighlightedCode` 通过 `ColorFile` 自带语言识别，无需显式标签。kloud 用 dim 单行标签保持信息量。

### 4. 移除 bash_border 主题字段

**选择**: 移除 `Theme.bash_border`，bash 与其他工具相同
**理由**: CC 不区分 bash 边框颜色。工具名 bold + 状态 dot 颜色已足够区分。

### 5. 折叠逻辑保留

**选择**: 保留 5 行折叠阈值和 Tab 切换，但用 `⎿ ... N more lines (Tab to expand)` 替代边框内折叠
**理由**: 功能正确，只改视觉包裹方式。

## Risks / Trade-offs

- [`⎿` 字符兼容性] → 大多数现代终端支持 Unicode，macOS Terminal.app 和 iTerm2 均可。若遇问题可 fallback 到 `┆`
- [行号 gutter 宽度计算] → 需在渲染时预计算行数以确定 gutter 宽度。代码块文本已完整，计算简单。
- [测试更新] → 现有测试断言 `┌─` / `╭─` 等边框字符，需改为缩进断言。
- [视觉回归] → 改动范围大，需通过 `tui_showcase` 示例视觉验证。
