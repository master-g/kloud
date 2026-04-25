## 1. 修改 auto-scroll 条件

- [x] 1.1 在 `src/ui/tui/widgets/messages.rs` 中，将 `messages.rs:113` 的 auto-scroll 条件从 `is_streaming && !state.auto_scroll_paused` 改为 `!state.auto_scroll_paused`（保留 `is_streaming` 变量仅用于 indicator 显示判断）
- [x] 1.2 在 auto-scroll 激活时（`scroll = max_scroll` 分支），同步写回 `state.scroll = max_scroll as u16`，确保位置持久化

## 2. 扩展 scroll-up 暂停范围

- [x] 2.1 在 `src/ui/tui/state.rs` 的 `scroll_messages_up` 方法中，去掉 `matches!(self.status, Streaming | Cancelling)` 条件，任何状态下上滚都设置 `auto_scroll_paused = true`

## 3. 验证

- [x] 3.1 `cargo check` 确认编译通过
- [ ] 3.2 运行 `cargo run --example tui_showcase`，验证内容超一屏后空闲状态停留在底部、不弹回第一页
