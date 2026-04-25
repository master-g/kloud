## Why

tui_showcase 示例存在 5 个 bug 导致演示内容缺失、阶段失效、用户感知为"TUI 提前退出"。具体：stream_plan 缺少 match 分支、时序重叠导致消息被静默丢弃、多行输入不提交导致残留文本污染后续阶段。

## What Changes

- 在 `DemoClient::stream_plan` 的 match 中补上 `diff_demo`、`pause_turn_demo`、`stop_sequence_demo` 三个分支
- 调整 `run_showcase` 中 stream_demos 的延迟，消除时序重叠（batch tools → mcp、stop sequence → help）
- 修复多行输入阶段：改用 `Ctrl+J` 或在 Enter 前清空/提交文本，确保 text_area 在阶段结束时为空
- 在多行输入阶段结束后显式清空 text_area（通过连续 Backspace 注入），防止残留文本污染后续 autocomplete/search 阶段
- 在 autocomplete 阶段开始前确保 text_area 为空

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tui-showcase-extended`: 修复缺失的 demo 分支、时序重叠、多行输入残留文本问题

## Impact

- `examples/tui_showcase.rs` — 主要修改文件（stream_plan match、run_showcase 时序和 key injection 序列）
- 无 API/依赖变更
- 无 breaking change
