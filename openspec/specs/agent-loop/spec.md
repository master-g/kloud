# Agent Loop（Agent 循环）

## 它是什么

Agent 的"大脑循环"——决定什么时候该问模型、什么时候该执行工具、什么时候该交给用户。
kloud 当前是**单轮工具循环**（用户 → 模型 → 工具 → 模型 → … → 结束），未来要演化为 Claude Code 风格的双循环架构。

## 关键类型

| 类型 | 位置 | 一句话 |
|------|------|--------|
| `TurnState` | `app/session/types.rs` | 当前轮次的运行时状态：Idle 或 Streaming |
| `Session` | `app/session/mod.rs` | 外循环所有者，持有 messages、store、tool_registry |
| `SessionStore` | `agent/store.rs` | 纯数据 store，通过 `apply(SessionEvent)` 更新 |
| `SessionEvent` | `agent/session_event.rs` | 所有可能的状态变更事件（20+ variants） |
| `StopReason` | `llm/response.rs` | 模型停下来的原因——Agent Loop 的分支依据 |

## 当前数据流

```
用户输入
  │
  ▼
Session::run() ─── 外循环 (tokio::select!)
  │
  ├─ TurnState::Idle ─── 等待 UiAction
  │     │
  │     └─ UiAction::SendMessage(text)
  │           │
  │           ▼
  │     start_streaming_turn(text)
  │           │
  │           ▼
  │     start_model_stream() ── 构建 ChatRequest, 调 LLM
  │           │
  │           ▼
  │     TurnState::Streaming { stream, pending_blocks, ... }
  │
  ├─ TurnState::Streaming ─── 内循环 (tokio::select!)
  │     │
  │     ├─ stream.next() ── SSE 事件到达
  │     │     │
  │     │     ├─ ContentBlockStart/Delta/Stop ── 累积 pending blocks
  │     │     ├─ MessageDelta ── 更新 stop_reason + usage
  │     │     └─ None (流结束) ── finish turn
  │     │
  │     └─ UiAction ── 用户中断/退出
  │
  └─ 流结束后
        │
        ▼
  continue_after_assistant_turn()
        │
        ├─ StopReason::ToolUse ── 提取工具调用 → 执行 → 追加 tool_result → start_model_stream()
        │
        └─ StopReason::EndTurn ── 回到 Idle，等用户
```

## 与 Claude Code 的差距

| 能力 | Claude Code | kloud 当前 |
|------|------------|-----------|
| 并行工具调用 | 支持，多工具同时执行 | `disable_parallel_tool_use: true` |
| 工具权限确认 | 每次工具调用前需用户确认 | 无权限系统 |
| 后续队列（外循环） | 用户消息 + 工具结果进同一队列 | 无队列，同步循环 |
| 引导队列（内循环） | 工具调用完成后可直接再调工具 | 已实现基本版 |
| 上下文压缩 | 自动压缩过长对话 | 无 |
| 子代理委派 | 可 spawn 子 agent | 无 |

## 未决问题

1. **并行工具执行**：当前 `tool_calls` 是顺序执行的（`for dispatch in tool_calls`），需要改为 `futures::join_all` 或类似并发模式
2. **权限确认中间态**：`PendingPermissionView` 已在 store 里定义，但 `Session` 还没有在工具执行前暂停等用户确认的逻辑
3. **错误恢复策略**：工具失败时是继续给模型（当前做法）、重试、还是直接中止？需要策略化
4. **最大循环次数**：防止 agent 死循环（模型调工具 → 工具结果 → 模型再调工具 → …）的硬上限在哪？

## 参考

- ROADMAP 里程碑 3：Agent 状态机 + 双循环核心
- 源文件：`src/app/session/`（全部 5 个文件）
- `src/agent/store.rs` — store/reducer 模式
- `src/agent/session_event.rs` — 完整事件列表
