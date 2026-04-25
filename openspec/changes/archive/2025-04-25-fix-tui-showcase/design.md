## Context

tui_showcase 是一个无需真实 LLM 的确定性演示，通过 `DemoClient::stream_plan` 返回预设的流事件序列。`run_showcase` 驱动器按固定时间间隔发送 prompt 和 key injection。

当前有 5 个 bug：
1. `stream_plan` match 缺少 `diff_demo`、`pause_turn_demo`、`stop_sequence_demo` 分支
2. batch tools demo 结束前 mcp demo prompt 已到达（重叠 1360ms）
3. stop sequence 默认响应结束前 help 命令已到达（重叠 ~940ms）
4. 多行输入阶段 Enter 不提交（多行模式下 Enter 插入换行）
5. 残留文本污染后续 autocomplete/search 阶段

## Goals / Non-Goals

**Goals:**
- 所有 19 个 streaming demo 正确展示其专属内容
- 所有 post-demo 阶段（slash commands、cancel、transcript、search、multi-line、autocomplete、permission、notification）正确执行
- 无时序重叠、无消息丢弃
- text_area 在每个阶段开始和结束时状态干净

**Non-Goals:**
- 不改变 Session/Backend 架构（消息丢弃是设计选择，本次只修 showcase 时序）
- 不改变 pace multiplier 默认值
- 不改变 input.rs 的 Enter 行为（多行 Enter=换行是正确行为）

## Decisions

### D1: 修复 stream_plan match

在 `stream_plan` 的 match 中添加三个分支：
```rust
Some(text) if text.contains("diff demo") => self.diff_demo(),
Some(text) if text.contains("pause turn demo") => self.pause_turn_demo(),
Some(text) if text.contains("stop sequence demo") => self.stop_sequence_demo(),
```

放在 `context demo` 之后、`_` 之前。这样 "show diff demo" 等不再落入默认分支。

**替代方案**: 无。这是唯一正确做法。

### D2: 消除时序重叠 — 增大延迟

两个重叠点：
- batch tools demo（#10）延迟 8000ms → mcp demo 在 batch tools 仍在 streaming 时到达
  - 方案：将 batch tools 延迟从 8000ms 增大到 10000ms（batch tools 总耗时 7360ms，原间隔仅 8000ms）
- stop sequence demo（#18）默认响应 ~3940ms → help 命令在 3000ms 后到达
  - 方案：将 slash command phase 的初始延迟从 3000ms 增大到 5000ms

增加的总时长：~4000ms（3分钟 → 约3分钟4秒），可接受。

**替代方案**: 改为等待响应结束后再发送下一条。但 `run_showcase` 当前架构是固定时间驱动，改动大。增大延迟更简单可靠。

### D3: 修复多行输入阶段

问题根因：Shift+Enter 使 text_area 进入多行模式，后续 Enter 插入换行而非提交。

方案：多行输入阶段结束后，注入足够多的 Backspace 清空 text_area，再注入 Escape 确保退出 Vim Normal 模式（如果进入了的话）。

具体步骤（在 Enter 后追加）：
```
// 清空 text_area（"line one\nline two\n" 共 20 字符 + 2 换行 ≈ 25 次 Backspace）
for _ in 0..30 {
    inject Backspace with 30ms delay
}
inject Escape (确保 Vim Normal 退出)
```

**替代方案**: 改用单行输入演示（删掉 Shift+Enter），但这样就失去了多行输入的展示意义。保留多行演示 + 清空是更好的选择。

### D4: Autocomplete 阶段前置清空

在 autocomplete 阶段开始前（输入 '/' 前），先注入 Escape + Ctrl+U 清空输入行：
```
inject Escape (退出 Vim Normal)
inject Ctrl+U (kill-to-beginning，清空输入)
```

Ctrl+U 在输入为空时被映射为 `scroll_messages_up`，但在非空时会执行 `kill_to_beginning`。所以需要确保 text_area 有内容时才发送 Ctrl+U。

实际上，结合 D3 的清空步骤，text_area 在 autocomplete 阶段时应该已经是空的。只需在 autocomplete 开始前加一个安全检查：注入 Escape。

### D5: Search 阶段前置清空

同 D4，search 阶段开始前（输入 '/' 前）确保 text_area 为空。由于 D3 已清空，只需注入 Escape 防止 Vim Normal 模式干扰。

## Risks / Trade-offs

- **增大延迟使展示变长**：增加约 4 秒。3分钟 vs 3分4秒，用户无感知。→ 可接受。
- **Backspace 清空依赖字符计数**：如果多行文本长度变化，清空可能不彻底。→ 用 30 次 Backspace（远超实际字符数），并在后续阶段前加 Escape 保险。
- **Ctrl+U 空输入时触发 scroll**：`input.rs:247` 中 `Ctrl+U when empty` 映射为 scroll。→ 确保只在 text_area 有内容时才发送 Ctrl+U，或直接用 Backspace 序列清空。
