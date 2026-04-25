## Context

kloud TUI 消息区使用 ratatui `Paragraph::scroll((offset, 0))` 渲染。滚动位置由 `TuiState::scroll: u16` 控制。当前 auto-scroll 逻辑在 `messages.rs:113` 将条件绑定为 `is_streaming && !auto_scroll_paused`，导致 Idle 时 scroll 始终为 0，内容超一屏后弹回顶部。

原版 CC 使用 Ink `ScrollBox` 组件的 `stickyScroll` 属性：默认 true，内容增长即滚底，与 streaming 状态无关。用户上滚通过 `lastUserScrollTsRef` 暂停 follow。

当前代码结构：
- `state.scroll: u16` — 手动滚动偏移（PageUp/Down 更新）
- `state.auto_scroll_paused: bool` — 用户上滚时设 true
- `state.virtual_scroll: VirtualScroll` — 虚拟滚动（>100 消息时启用，独立 auto_scroll 标志）

两条滚动路径并存：简单 Paragraph.scroll 路径和 VirtualScroll 路径。

## Goals / Non-Goals

**Goals:**
- Idle + 非暂停状态下始终显示最新内容（滚到底部）
- 用户主动上滚后暂停 auto-scroll
- 用户滚回底部后恢复 auto-scroll
- streaming→Idle 切换后滚动位置不丢失

**Non-Goals:**
- 不实现 CC 的 3 秒 repin 窗口（kloud 输入场景不同）
- 不修改 VirtualScroll 机制（>100 消息场景）
- 不改变 `u16` 滚动类型（后续可单独优化）

## Decisions

### D1: 去掉 is_streaming 条件

将 `messages.rs` 的 auto-scroll 条件从 `is_streaming && !auto_scroll_paused` 改为 `!auto_scroll_paused`。

- `is_streaming` 变量仍保留用于 "auto-scroll paused" 指示器显示（仅 streaming 时显示）
- Transcript 模式的特殊处理保持不变（已有 `screen != Transcript` 检查）

**替代方案**: 保持 is_streaming 但同步写回 state.scroll。— 语义不清晰，且未解决"空闲时应显示最新内容"的期望。

### D2: auto-scroll 时写回 state.scroll

auto-scroll 激活时，将 `max_scroll` 写回 `state.scroll`。确保：
- streaming→Idle 切换后位置持久
- 用户上滚时从当前位置开始计算

在渲染函数中赋值：`state.scroll = max_scroll`。ratatui 渲染不影响 &self（需要 &mut），所以需要在渲染后回调或在 `sync_from_active_view` 中处理。

实际更简单的方式：不写回 state.scroll，而是在 `else` 分支中，当 `!auto_scroll_paused` 时也使用 `max_scroll`。这样 state.scroll 仅在 auto_scroll_paused 时生效。

### D3: scroll_messages_up 在任何状态下都暂停

当前 `scroll_messages_up` 仅在 streaming 时设置 `auto_scroll_paused = true`。改为任何状态下都设置，确保用户在 Idle 时上滚也能暂停 auto-follow。

## Risks / Trade-offs

- **去掉 is_streaming 后用户无法在 Idle 时浏览历史**: 已通过 auto_scroll_paused 覆盖——用户上滚即暂停。
- **state.scroll u16 溢出**: >65535 行时截断。虚拟滚动路径（>100 消息）使用 usize 不受影响。短期能接受。
- **渲染函数中修改 state**: 需确保 borrow checker 允许。消息渲染函数签名接收 `&mut TuiState`，可直接修改。
