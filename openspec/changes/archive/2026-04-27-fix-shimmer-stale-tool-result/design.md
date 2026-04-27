## Context

Two independent bugs in the TUI layer cause incorrect rendering after the first showcase demo:

1. **StalledState carries over between queries** — `StalledState.last_response_length` (state.rs:65) accumulates the character count from previous demos. When a new query starts and `response_char_count` resets to 0 (store.rs:142), the stall detector sees `0 < last_response_length` as "no progress" and triggers false stall detection after 3 seconds, painting the activity line red.

2. **Tool result lookup has no scope boundary** — `latest_tool_result()` in the showcase example iterates ALL user messages in reverse to find tool results. After any tool-using demo completes, the tool result persists in message history. Subsequent non-tool demos match the stale result and produce wrong output.

### CC 源码对照

**Bug 1 — CC `useStalledAnimation.ts`**:
- `mountTime = useRef(time)` (line 17): 组件挂载时初始化，每次新 query 重新挂载
- `currentResponseLength > lastResponseLength.current` 时重置 `stalledIntensityRef.current = 0` (line 25)
- `currentResponseLength == 0` 时用 `time - mountTime.current` 计算 timeSinceLastToken (line 37)，不存在跨 turn 累积
- kloud 缺少 `mountTime` 等价物 → `last_response_length` 跨 turn 累积 → 假 stall

**Bug 2 — Anthropic Messages API 约定**:
- Tool results 始终是当轮最后一个 user message
- `latest_tool_result()` 应只检查最后一条 user message
- 当前实现扫描全部消息，违反 API 约定

**Shimmer stalled 处理** — CC `SpinnerAnimationRow.tsx:138`:
- `isStalled ? -100 : ...` 把 glimmerIndex 推到离屏
- kloud 通过 `stalled_intensity > 0.0` 分支覆盖为红色，效果等价，无需修改

## Goals / Non-Goals

**Goals:**

- Reset stall detector state on every new query (Idle→Streaming transition), matching CC's mount-time reset behavior
- Scope tool result lookup to only the last user message, matching Anthropic API convention
- Preserve all existing shimmer, stall, and tool result behavior within a single query

**Non-Goals:**

- Changes to the shimmer algorithm, glimmer math, or activity line rendering
- Changes to the session message management or tool dispatch protocol
- Changes to the StalledState detection thresholds or EMA smoothing
- Adding a `mountTime` field to StalledState (Idle→Streaming reset is equivalent and simpler)

## Decisions

### D1: Reset StalledState on status transition to Streaming

Add a `reset()` method to `StalledState` and call it in `apply_view()` when `previous_status != Streaming && self.status == Streaming`.

**Why**: The stall detector is per-query state — it tracks token arrival rate within a single streaming turn. Resetting on Idle→Streaming is the correct semantic boundary, equivalent to CC's component mount reset (`mountTime = useRef(time)`). The alternative (resetting `last_response_length` in `update()` when `response_length == 0`) would break stall detection during the initial seconds of a query before any tokens arrive.

**CC reference**: `useStalledAnimation.ts:17` — `const mountTime = useRef(time)` resets on every component mount, which in React maps to each new spinner lifecycle (each query).

### D2: Scope latest_tool_result to last user message only

Change `latest_tool_result()` from scanning all messages to only inspecting the last user message. If it contains no `ToolResult` blocks, return `None`.

**Why**: Tool results in the Anthropic protocol are always the LAST user message in a conversation turn. Any tool results deeper in the history are from earlier turns and should not influence routing. The current behavior (scanning all messages) is a bug — it was likely written before the showcase accumulated multiple turns.

**CC reference**: Anthropic Messages API convention — `tool_result` content blocks are always the last user message in the conversation array.

## Risks / Trade-offs

- **[Low Risk] StalledState reset timing** — If `apply_view()` is called multiple times during a single query, the stall detector could be reset mid-stream. Mitigation: the reset only fires on Idle→Streaming transition, which only happens once per query.
- **[No Risk] Tool result scope change** — This only affects the showcase example, not production code. The fix aligns with the Anthropic API convention where tool results are always the last user message.
