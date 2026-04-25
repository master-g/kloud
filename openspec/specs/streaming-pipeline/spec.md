# Streaming Pipeline（流式管线）

## 它是什么

从 Anthropic API 的 SSE 流到终端像素的全链路——kloud 的数据经络。
理解这条管线等于理解"用户按回车后发生了什么"。

## 关键类型

| 类型 | 位置 | 一句话 |
|------|------|--------|
| `SseDecoder` | `llm/sse.rs` | `tokio_util::codec::Decoder`，以 `\n\n` 切帧 |
| `StreamEvent` | `llm/response.rs` | SSE 事件的 Rust 表示（MessageStart, ContentBlockDelta, ...） |
| `Delta` | `llm/response.rs` | 增量内容：TextDelta, InputJsonDelta, ThinkingDelta, SignatureDelta |
| `PendingBlock` | `app/session/types.rs` | 正在流式接收中的未完成 block |
| `TurnState::Streaming` | `app/session/types.rs` | 持有 stream + pending_blocks + stop_reason |
| `SessionEvent` | `agent/session_event.rs` | Store 的 reducer 输入 |
| `AppEvent::View` | `ui/events.rs` | 推给 UI 的快照 |

## 全链路数据流

```
Anthropic API (HTTPS SSE)
  │
  │  reqwest::Response::bytes_stream()
  │
  ▼
StreamReader (bytes → BytesMut)
  │
  │  FramedRead<SseDecoder>  ── 以 \n\n 切帧
  │
  ▼
原始 SSE 帧 ("event: content_block_delta\ndata: {...}\n")
  │
  │  serde_json::from_str() ── 解析 "data:" 行
  │
  ▼
StreamEvent (Rust enum)
  │
  │  Session::handle_stream_event()
  │
  ├─ ContentBlockStart ── 创建 PendingBlock (Text/Thinking/ToolUse)
  │     │
  │     └─ emit: AssistantTextDelta / AssistantThinkingDelta / AssistantToolUseStarted
  │
  ├─ ContentBlockDelta ── 追加到 PendingBlock
  │     │
  │     ├─ TextDelta     → PendingBlock::Text.text.push_str()
  │     ├─ ThinkingDelta → PendingBlock::Thinking.thinking.push_str()
  │     ├─ InputJsonDelta → PendingBlock::ToolUse.input_json.push_str()
  │     └─ SignatureDelta → PendingBlock::Thinking.signature = Some()
  │     │
  │     └─ 同时 emit 对应 SessionEvent
  │
  ├─ ContentBlockStop ── finalize PendingBlock → completed_assistant_blocks
  │
  ├─ MessageDelta ── 更新 stop_reason + usage
  │     └─ emit: UsageUpdated
  │
  ├─ MessageStart ── 忽略（初始元数据）
  ├─ MessageStop  ── 流结束信号
  ├─ Ping         ── 忽略
  └─ Error        ── emit: SystemMessageAdded(error)
  │
  │  每个 emit → Session::apply_event()
  │
  ▼
SessionStore::apply(SessionEvent)
  │
  │  store.view() ── 重新计算 view
  │
  ▼
AppEvent::View(Box<SessionView>) ── mpsc channel
  │
  │  event_rx.recv()
  │
  ▼
RatatuiBackend
  │
  │  state.apply_view(view)
  │
  ▼
TuiState ── 更新消息列表 + 动画状态
  │
  │  terminal.draw() ── 20 FPS render tick
  │
  ▼
Terminal pixels (ratatui + crossterm)
```

## SSE 帧格式

```
event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

```

**注意**：帧以 `\n\n` 结尾。`SseDecoder` 找到第一个 `\n\n` 的位置，split 出一帧。TCP chunk 边界不保证对齐 `\n\n`——这是为什么需要 `Decoder` trait 而不是简单的 line reader。

## 流式内容的"双重缓冲"

```
StreamEvent 世界              SessionStore 世界
───────────────               ────────────────
PendingBlock (in-memory)      current_assistant (in store)
  ├─ Text { text }              ├─ DisplayBlock::Text (追加)
  ├─ Thinking { thinking }      ├─ DisplayBlock::Thinking (追加)
  └─ ToolUse { input_json }     └─ DisplayBlock::ToolUse (input_json 增量更新)

两条线并行工作：
1. PendingBlock 累积最终要写入 messages[] 的 ContentBlock
2. Store 的 current_assistant 累积 UI 实时看到的 DisplayBlock
```

**为什么两套**：PendingBlock 精确跟踪 SSE index，保证顺序；Store 的 current_assistant 通过 apply_event 增量更新，保证 UI 实时刷新。

## 流式期间的事件优先级

```rust
tokio::select! {
    maybe_event = stream.next() => { ... }    // 优先处理 SSE
    maybe_action = action_rx.recv() => { ... } // 用户中断
}
```

SSE 和 UI action 平等竞争——用户可以在任何时候中断流。中断时：
1. 收集已完成的 blocks
2. `persist_assistant_turn`（保存已有内容）
3. emit `InterruptRecorded`
4. 回到 Idle

## 性能特征

| 环节 | 机制 | 备注 |
|------|------|------|
| 网络层 | reqwest bytes_stream | TCP chunk 可能不齐 |
| 帧解析 | tokio_util Decoder | 内部缓冲，不足则返回 None |
| JSON 反序列化 | serde_json | 每个 delta 都反序列化一次 |
| Store 更新 | 每次 apply_event 都重新 view() | O(messages * blocks) |
| UI 推送 | mpsc channel 256 容量 | 背压保护 |
| 渲染 | 20 FPS tick + event-driven | 不是每次 delta 都重绘 |

## 未决问题

1. **view() 性能**：每次 delta 都触发完整的 normalize → lookup → reorder → group 管线。消息多了会慢。需要增量更新？
2. **Delta 合并**：连续的 TextDelta 可以合并再推 UI（减少 channel 消息量），当前逐条推送
3. **流式错误不中断**：`StreamEvent::Error` 只记日志不停止流，可能导致后续事件处于不一致状态
4. **Thinking block 的 signature**：流式时 signature 单独通过 `SignatureDelta` 发送，finalize 时拼回——这个时序假设是否可靠？
5. **channel 容量**：event channel 256 够用吗？如果 LLM 暴发大量 delta 而 UI 来不及消费，背压会传导到 Session

## 参考

- SSE 解码：`src/llm/sse.rs`
- 流式事件处理：`src/app/session/stream.rs`
- Store 管线：`src/agent/store.rs`（view() 方法）
- TUI 渲染循环：`src/ui/tui/mod.rs`
- LLM 类型定义：`src/llm/response.rs`, `src/llm/types.rs`

## Additional Requirements (from tui-cc-replica)

### Requirement: Stream events carry UI rendering metadata
StreamEvent handling SHALL emit additional metadata for TUI animations: per-delta token count increment, stall detection signal, and thinking duration.

#### Scenario: Token count delta in stream event
- **WHEN** a ContentBlockDelta(TextDelta) event is processed
- **THEN** the emitted SessionEvent SHALL include a cumulative token count for the current turn
- **THEN** the TUI SHALL use this count for animated token display

#### Scenario: Stall detection signal
- **WHEN** no ContentBlockDelta arrives for 3000ms during streaming
- **THEN** the Session SHALL emit a `StreamStalled` event
- **WHEN** a new delta arrives after stall
- **THEN** the Session SHALL emit a `StreamResumed` event

#### Scenario: Thinking duration tracking
- **WHEN** a ContentBlockStart(Thinking) event is received
- **THEN** the Session SHALL record the start timestamp
- **WHEN** the thinking block completes (ContentBlockStop)
- **THEN** the SessionEvent SHALL include the thinking duration in milliseconds
