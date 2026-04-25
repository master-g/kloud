# Session Lifecycle（会话生命周期）

## 它是什么

Session 是 kloud 的中枢神经——它连接 LLM、UI、ToolRegistry，管理对话历史，驱动整个交互循环。
理解 Session 的生命周期就是理解整个系统的运行方式。

## 关键类型

| 类型 | 位置 | 一句话 |
|------|------|--------|
| `Session` | `app/session/mod.rs` | 长生命周期所有者：client + messages + store + handle + registry |
| `SessionStore` | `agent/store.rs` | 纯数据 store，reducer 模式 (apply → view) |
| `SessionView` | `agent/view.rs` | store 的快照，UI 据此渲染 |
| `SessionEvent` | `agent/session_event.rs` | 20+ variants，覆盖所有状态变更 |
| `TurnState` | `app/session/types.rs` | 短生命周期：当前轮次是 Idle 还是 Streaming |
| `UiHandle` / `UiChannels` | `ui/backend.rs` | Session 和 UI 之间的双向通道 |

## Session 的生命周期

```
创建
  │  Session::new(client, registry, prompt, max_tokens, handle)
  │
  ▼
初始化
  │  publish_view() ── 发送初始空 view 给 UI
  │
  ▼
运行 ─── Session::run()
  │
  │  ┌──────────────────────────────────────────┐
  │  │  外循环 (Idle)                            │
  │  │    ├─ SendMessage → start_streaming_turn │
  │  │    ├─ SlashCommand → 处理或退出           │
  │  │    ├─ CancelTurn → 忽略（本来就没在跑）    │
  │  │    ├─ SetScreen → 更新 store              │
  │  │    └─ Exit → break                        │
  │  │                                          │
  │  │  内循环 (Streaming)                       │
  │  │    ├─ SSE 事件 → handle_stream_event     │
  │  │    ├─ CancelTurn → 截断，记中断，回 Idle  │
  │  │    └─ Exit → 截断，记完成，退出           │
  │  │                                          │
  │  │  流结束                                   │
  │  │    ├─ ToolUse → 执行工具 → 重新内循环     │
  │  │    └─ EndTurn → 回外循环                  │
  │  └──────────────────────────────────────────┘
  │
  ▼
退出
   Ok(()) 或 Err
```

## 职责边界

### Session 管（做什么）

- 持有完整对话历史 (`messages: Vec<InputMessage>`)
- 驱动 LLM 调用（构建 request、解析 response）
- 协调工具执行（dispatch → tool_result → 再调 LLM）
- 通过 `apply_event → publish_view` 推送状态到 UI
- 处理用户中断（CancelTurn）

### Session 不管（不做什么）

- **不渲染**：渲染是 `RatatuiBackend` 的事，Session 只发 `SessionView`
- **不做持久化**：对话历史没有保存到磁盘（M4 计划）
- **不管理 token 预算**：不做上下文压缩或截断（M4 计划）
- **不做权限确认**：`PendingPermissionView` 存在但未接入流程

## Store → View 管线

```
SessionStore (source of truth)
  │
  │  store.view()
  │
  ├─ normalize_messages()        ── 每个 block 独立成行
  ├─ build_message_lookups()     ── O(1) 查 tool_use↔tool_result 关系
  ├─ reorder_messages_in_ui()    ── tool_result 紧跟 tool_use
  ├─ group_messages_for_display()── 合并回 DisplayMessage
  │
  ▼
SessionView (snapshot)
  │
  │  AppEvent::View(Box<SessionView>)
  │
  ▼
RatatuiBackend.apply_view()
  │
  ▼
TuiState ── 持有 view 副本 + 本地动画状态
  │
  ▼
widgets::render() ── 画到 terminal
```

**设计亮点**：每次状态变更都重新计算 view。因为 `apply_event` 后会立即 `publish_view`，UI 始终看到最新状态。revision 号单调递增，可用于脏检查。

## 消息的两次表示

kloud 中同一条消息有两个平行的表示：

```
LlmClient 世界                   SessionStore 世界
─────────────                    ────────────────
InputMessage                     TranscriptMessage
  role: Role                       message_type: MessageType
  content: Vec<ContentBlock>       blocks: Vec<DisplayBlock>

ContentBlock                    DisplayBlock
  Text { text, cache_control }     Text(String)
  Thinking { thinking, sig }       Thinking(String)
  ToolUse { id, name, input }      ToolUse { id, name, status, input_preview, ... }
  ToolResult { ... }               ToolResult { ... }
```

**为什么两套**：`ContentBlock` 是 API 序列化格式，`DisplayBlock` 是 UI 渲染格式。后者多了 `ToolStatus`、`input_preview`、`server_name` 等 UI 专属字段。转换函数在 `stream.rs` 的 `display_blocks_from_content_blocks()`。

## 未决问题

1. **Session 膨胀**：`Session` 已经有 6 个字段 + 5 个 impl 文件，随着 M3-M7 会继续增长——何时拆分？
2. **TurnState 解构爆炸**：每个方法都有 6-8 个参数（从 TurnState::Streaming 解构出来的字段），用结构体化减少参数数量？
3. **错误恢复粒度**：LLM 调用失败 → 记日志回 Idle；工具失败 → 包装为 ToolResult 继续。两种错误处理策略不一致
4. **多轮对话 history 无限增长**：没有 token 预算管理，大项目长会话迟早爆 context window

## 参考

- 源文件：`src/app/session/`（mod, run, stream, tools, types）
- Store/View：`src/agent/`（store, view, message, session_event）
- UI 通道：`src/ui/backend.rs`, `src/ui/events.rs`
- ROADMAP 里程碑 3-4

## Additional Requirements (from tui-cc-replica)

### Requirement: SessionView supports TUI mode state
SessionView SHALL include the current UI mode (Normal, Search, Transcript) and mode-specific data (search query, match positions, transcript scroll offset).

#### Scenario: Switch to search mode
- **WHEN** user activates search mode
- **THEN** the SessionView SHALL include `screen: Screen::Search` with an empty query
- **THEN** subsequent keystrokes SHALL update the query in the view

#### Scenario: Switch to transcript mode
- **WHEN** user presses `Ctrl+O`
- **THEN** the SessionView SHALL include `screen: Screen::Transcript` with full message history
- **WHEN** user presses `Ctrl+O` again
- **THEN** the SessionView SHALL revert to `screen: Screen::Prompt`

### Requirement: AppEvent and UiAction extended for new UI features
AppEvent SHALL include new variants: `ThemeChanged(String)`, `SearchQuery(String)`, `SearchNavigateNext`, `SearchNavigatePrev`, `ToggleToolCollapse(usize)`. UiAction SHALL include corresponding user action variants.

#### Scenario: Theme change event
- **WHEN** user issues `/theme dark`
- **THEN** a `UiAction::ChangeTheme("dark")` SHALL be sent to Session
- **THEN** Session SHALL emit `AppEvent::ThemeChanged("dark")` back to UI
- **THEN** the UI SHALL re-render with new theme colors

#### Scenario: Tool collapse toggle
- **WHEN** user presses `Tab` on a focused tool block
- **THEN** a `UiAction::ToggleToolCollapse(block_index)` SHALL be sent
- **THEN** the SessionView SHALL update the collapse state for that block
