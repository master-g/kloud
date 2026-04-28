## Why

`fix-tool-name-double-render` 的 code review 发现 `tool_block.rs` 中 3 个可改进点：误导性控制流（`if let` 外残留 `for`）、冗余 span 重建、缺少 fallback 行为注释。

## What Changes

- 修正 `render_tool_use_line()` 的控制流：把 `for line in iter` 移入 `if let Some(first)` 块内，消除误导
- 统一首行渲染路径：用 `prepend_prefix_to_line` + 单独 dot span 替代手动 span 重建，减少 `to_string()` clone
- 给空 rendered fallback 路径加注释，说明是有意使用 raw name

## Capabilities

### New Capabilities

_(无)_

### Modified Capabilities

_(无 — 纯代码质量改进，无 spec 级别行为变更)_

## Impact

- `src/ui/tui/widgets/tool_block.rs` — `render_tool_use_line()` 内部重构，签名不变
