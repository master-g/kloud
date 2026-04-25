## 1. 补全 stream_plan match 分支

- [x] 1.1 在 `DemoClient::stream_plan` 的 match 中，`context demo` 分支后、`_` 分支前，添加 `diff demo`、`pause turn demo`、`stop sequence demo` 三个 match arm，分别调用 `self.diff_demo()`、`self.pause_turn_demo()`、`self.stop_sequence_demo()`
- [x] 1.2 编译验证：确认 `cargo check --example tui_showcase` 无 dead_code 警告

## 2. 修复时序重叠

- [x] 2.1 将 `run_showcase` 中 batch tools demo 的延迟从 `pace(8_000)` 增大到 `pace(10_000)`，消除与 mcp demo 的 1360ms 重叠
- [x] 2.2 将 slash command phase 初始延迟从 `pace(3_000)` 增大到 `pace(5_000)`，消除 stop sequence 默认响应与 help 命令的重叠
- [ ] 2.3 手动验证所有 19 个 demo + slash commands 的时序：每个 prompt 到达时 session 必须在 Idle 状态

## 3. 修复多行输入阶段残留文本

- [x] 3.1 在多行输入阶段的 Enter 之后，注入 30 次 Backspace（每次 30ms 间隔）清空 text_area（无需 Escape — Backspace 期间保持 Insert 模式）
- [x] 3.2 ~~注入一次 Escape~~ 改为不注入 Escape（Escape 在 Insert+Idle 下会进入 Normal 模式，适得其反）
- [ ] 3.3 验证：清空后 text_area 为空，后续 '/' key injection 能正确触发 SearchActivate 或 autocomplete

## 4. 修复 autocomplete 和 search 阶段前置条件

- [x] 4.1 ~~注入 Escape~~ 改为无需 Escape（多行清理后的 Backspace 保持 Insert 模式，text_area 为空）
- [x] 4.2 ~~注入 Escape~~ 改为无需 Escape（同上）
- [ ] 4.3 验证 autocomplete 演示的 '/' 输入触发 autocomplete filter（text_area 内容以 '/' 开头）
- [ ] 4.4 验证 search 演示的 '/' 输入触发 SearchActivate（text_area 为空）

## 5. 端到端验证

- [ ] 5.1 运行 `cargo run --example tui_showcase`，观察全部 demo 按序执行无跳跃
- [ ] 5.2 验证所有 post-demo 阶段正常：help 命令显示、theme 切换、cancel 流、transcript 模式、search、permission prompt
- [ ] 5.3 确认 TUI 在 showcase 完成后正常退出（约 3 分 5 秒后）
