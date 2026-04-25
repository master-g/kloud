## Why

TUI 消息区内容超出一屏后，空闲时始终显示第一页，仅在 streaming 短暂滚底，处理完又弹回。原版 CC 的 `stickyScroll` 机制与 streaming 状态无关——只要用户没主动上滚，就始终跟随最新内容。

## What Changes

- 移除 auto-scroll 对 `is_streaming`（`AssistantStatus::Streaming`）的依赖，改为仅看 `auto_scroll_paused` 标志
- auto-scroll 激活时同步更新 `state.scroll`，使滚动位置在 streaming→Idle 切换后持久化
- 新消息到达时若 `auto_scroll_paused == false`，保持滚到底部

## Capabilities

### New Capabilities

- `sticky-scroll`: 消息区 stickyScroll 行为——默认跟随底部，用户上滚暂停，回到底部恢复

### Modified Capabilities

## Impact

- `src/ui/tui/widgets/messages.rs`: auto-scroll 条件判断逻辑
- `src/ui/tui/state.rs`: `scroll` 字段持久化、`auto_scroll_paused` 重置时机
