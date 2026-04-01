# Claude Code Spinner 系统源码分析

本文档详细分析 Claude Code 的 Spinner 组件实现，用于指导 kloud TUI 的类似功能开发。

---

## 1. 组件架构概览

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SpinnerAnimationRow                              │
│  ┌─────────┐  ┌──────────────────────────────────────────────┐ │
│  │ Spinner │  │              GlimmerMessage                    │ │
│  │  Glyph  │  │  [verb]  [shimmer message]  (status parts)    │ │
│  └─────────┘  └──────────────────────────────────────────────┘ │
│      ↓                    ↓                        ↓               │
│   旋转字符动画          shimmer 效果         计时器/Token/思考     │
└─────────────────────────────────────────────────────────────────────┘
```

### 核心文件位置

| 文件 | 功能 |
|------|------|
| `components/Spinner/SpinnerAnimationRow.tsx` | 主容器，50ms 动画循环 |
| `components/Spinner/SpinnerGlyph.tsx` | 旋转字符渲染 |
| `components/Spinner/GlimmerMessage.tsx` | shimmer 消息渲染 |
| `components/Spinner/ShimmerChar.tsx` | 单字符 shimmer 效果 |
| `components/Spinner/useShimmerAnimation.ts` | shimmer 动画 hook |
| `components/Spinner/useStalledAnimation.ts` | 卡顿检测 hook |
| `components/Spinner/utils.ts` | 工具函数 |

---

## 2. SpinnerMode 类型定义

```typescript
// Claude Code 使用字符串字面量类型，而非枚举
type SpinnerMode = 'requesting' | 'responding' | 'tool-input' | 'tool-use' | 'thinking';
```

### 模式切换时机

| 模式 | 触发条件 | shimmer 速度 |
|------|----------|-------------|
| `requesting` | 等待用户输入或 API 响应 | **50ms** (快) |
| `thinking` | content_block_start + thinking | 200ms |
| `responding` | content_block_start + text / message_delta | 200ms |
| `tool-input` | content_block_start + tool_use | 200ms |
| `tool-use` | message_stop (工具执行完成) | 200ms |

---

## 3. SpinnerGlyph 组件

### 关键常量

```typescript
// 旋转字符帧（包含正向和反向，形成连续旋转）
const DEFAULT_CHARACTERS = ['·', '✢', '✳', '✶', '✻', '✽'];
const SPINNER_FRAMES = [...DEFAULT_CHARACTERS, ...[...DEFAULT_CHARACTERS].reverse()];

// 减少动画选项
const REDUCED_MOTION_DOT = '●';
const REDUCED_MOTION_CYCLE_MS = 2000; // 2秒周期：1秒可见，1秒暗淡

// 卡顿时过渡到红色
const ERROR_RED = { r: 171, g: 43, b: 63 };
```

### 渲染逻辑

```typescript
function SpinnerGlyph({ frame, messageColor, stalledIntensity, reducedMotion, time }) {
  // 减少动画模式：使用静态点代替旋转
  if (reducedMotion) {
    const isDim = Math.floor(time / (REDUCED_MOTION_CYCLE_MS / 2)) % 2 === 1;
    return <Text dimColor={isDim}>{REDUCED_MOTION_DOT}</Text>;
  }

  const spinnerChar = SPINNER_FRAMES[frame % SPINNER_FRAMES.length];

  // 卡顿渐变到红色
  if (stalledIntensity > 0) {
    const baseRGB = parseRGB(theme[messageColor]);
    const interpolated = interpolateColor(baseRGB, ERROR_RED, stalledIntensity);
    return <Text color={toRGBColor(interpolated)}>{spinnerChar}</Text>;
  }

  return <Text color={messageColor}>{spinnerChar}</Text>;
}
```

### 帧率

```typescript
const frame = Math.floor(time / 120); // 120ms 切换一帧
```

---

## 4. GlimmerMessage 组件

### shimmer 动画核心算法

```typescript
// 计算 shimmer 位置
const glimmerSpeed = mode === 'requesting' ? 50 : 200;
const glimmerMessageWidth = stringWidth(message);
const cycleLength = glimmerMessageWidth + 20;
const cyclePosition = Math.floor(time / glimmerSpeed);

// requesting 模式：从左到右
const glimmerIndex = mode === 'requesting'
  ? (cyclePosition % cycleLength) - 10
  // 其他模式：从右到左
  : glimmerMessageWidth + 10 - (cyclePosition % cycleLength);
```

### 字符级别离散颜色切换

```typescript
// ShimmerChar.tsx - 精确还原 Claude Code 逻辑
function ShimmerChar({ char, index, glimmerIndex, messageColor, shimmerColor }) {
  const isHighlighted = index === glimmerIndex;
  const isNearHighlight = Math.abs(index - glimmerIndex) === 1;
  const shouldUseShimmer = isHighlighted || isNearHighlight;

  return <Text color={shouldUseShimmer ? shimmerColor : messageColor}>{char}</Text>;
}
```

**关键发现**：Claude Code 使用**离散颜色切换**（非 RGB 插值）：
- 当前字符位置：`shimmerColor`
- 相邻字符位置：`shimmerColor`（距离 ≤ 1）
- 其他位置：`messageColor`

### tool-use 模式的闪烁效果

```typescript
// tool-use 模式：整个消息背景色闪烁
const flashOpacity = mode === 'tool-use'
  ? (Math.sin(time / 1000 * Math.PI) + 1) / 2  // 正弦波，周期 2 秒
  : 0;
```

### 卡顿时的红色渐变

```typescript
if (stalledIntensity > 0) {
  const baseRGB = parseRGB(theme[messageColor]);
  const interpolated = interpolateColor(baseRGB, ERROR_RED, stalledIntensity);
  return <Text color={toRGBColor(interpolated)}>{message}</Text>;
}
```

---

## 5. useStalledAnimation Hook

### 卡顿检测逻辑

```typescript
function useStalledAnimation(time, currentResponseLength, hasActiveTools, reducedMotion) {
  // 1. 新 token 到达时重置计时器
  if (currentResponseLength > lastResponseLength.current) {
    lastTokenTime.current = time;
    stalledIntensityRef.current = 0;
  }

  // 2. 计算距上次 token 的时间
  if (hasActiveTools) {
    timeSinceLastToken = 0; // 有活跃工具时不检测卡顿
  }

  // 3. 3秒无响应触发卡顿
  const isStalled = timeSinceLastToken > 3000 && !hasActiveTools;
  const intensity = isStalled
    ? Math.min((timeSinceLastToken - 3000) / 2000, 1)  // 2秒内渐变到1
    : 0;

  // 4. 平滑过渡
  if (!reducedMotion && (intensity > 0 || stalledIntensityRef.current > 0)) {
    stalledIntensityRef.current += (intensity - stalledIntensityRef.current) * 0.1;
  }

  return { isStalled, stalledIntensity: effectiveIntensity };
}
```

### 关键常量

| 常量 | 值 | 说明 |
|------|-----|------|
| `STALL_THRESHOLD_MS` | 3000 | 3秒无响应触发卡顿 |
| `STALL_FADE_DURATION` | 2000 | 渐变到红色满值的时间 |
| `SMOOTHING_FACTOR` | 0.1 | 每步平滑系数 |

---

## 6. Token 计数动画

### 平滑递增算法

```typescript
const gap = currentResponseLength - tokenCounterRef.current;
if (gap > 0) {
  let increment;
  if (gap < 70) {
    increment = 3;
  } else if (gap < 200) {
    increment = Math.max(8, Math.ceil(gap * 0.15));
  } else {
    increment = 50;
  }
  tokenCounterRef.current = Math.min(
    tokenCounterRef.current + increment,
    currentResponseLength
  );
}
```

### 显示条件

```typescript
const SHOW_TOKENS_AFTER_MS = 30_000; // 30秒后才显示 token 计数
const wantsTimerAndTokens = verbose || hasRunningTeammates || effectiveElapsedMs > SHOW_TOKENS_AFTER_MS;
```

---

## 7. Thinking Shimmer 呼吸效果

### 颜色常量

```typescript
const THINKING_INACTIVE = { r: 153, g: 153, b: 153 };
const THINKING_INACTIVE_SHIMMER = { r: 185, g: 185, b: 185 };
const THINKING_DELAY_MS = 3000;      // 延迟3秒启动
const THINKING_GLOW_PERIOD_S = 2;   // 周期2秒
```

### 动画实现

```typescript
const thinkingElapsedSec = (time - THINKING_DELAY_MS) / 1000;
const thinkingOpacity = time < THINKING_DELAY_MS
  ? 0
  : (Math.sin(thinkingElapsedSec * Math.PI * 2 / THINKING_GLOW_PERIOD_S) + 1) / 2;

const thinkingShimmerColor = toRGBColor(
  interpolateColor(THINKING_INACTIVE, THINKING_INACTIVE_SHIMMER, thinkingOpacity)
);
```

---

## 8. SpinnerModeGlyph - 状态区域的箭头图标

```typescript
function SpinnerModeGlyph({ mode }) {
  switch (mode) {
    case 'tool-input':
    case 'tool-use':
    case 'responding':
    case 'thinking':
      return <Text dimColor>{figures.arrowDown}</Text>; // ↓
    case 'requesting':
      return <Text dimColor>{figures.arrowUp}</Text>;   // ↑
  }
}
```

**重要发现**：箭头图标**只出现在状态区域**（token 计数旁边），**不是主 glyph**。

---

## 9. 动画时钟系统

### useAnimationFrame Hook

```typescript
function useAnimationFrame(intervalMs: number | null = 16) {
  const clock = useContext(ClockContext);
  const [viewportRef, { isVisible }] = useTerminalViewport();
  const [time, setTime] = useState(() => clock?.now() ?? 0);

  // 视口不可见或传入 null 时暂停
  const active = isVisible && intervalMs !== null;

  useEffect(() => {
    if (!clock || !active) return;

    const onChange = (): void => {
      const now = clock.now();
      if (now - lastUpdate >= intervalMs) {
        lastUpdate = now;
        setTime(now);
      }
    };

    return clock.subscribe(onChange, true); // keepAlive: true
  }, [clock, intervalMs, active]);

  return [viewportRef, time];
}
```

### 关键特性

1. **共享时钟**：所有动画实例共享同一个 ClockContext
2. **视口暂停**：当终端不可见时自动暂停
3. **可暂停**：`intervalMs = null` 时完全停止
4. **防抖**：只有当 `now - lastUpdate >= intervalMs` 时才更新

---

## 10. 渐进式宽度控制

### 显示优先级

```
message > thinking > timer > tokens
```

### 空间计算

```typescript
const SEP_WIDTH = stringWidth(' · ');
const THINKING_BARE_WIDTH = stringWidth('thinking');
const availableSpace = columns - messageWidth - 5;

let showThinking = wantsThinking && availableSpace > thinkingWidthValue;
let showTimer = wantsTimerAndTokens && availableSpace > usedAfterThinking + timerWidth;
let showTokens = wantsTimerAndTokens && totalTokens > 0 && availableSpace > usedAfterTimer + tokensWidth;
```

---

## 11. kloud 现有实现 vs Claude Code

### 已完成

| 功能 | Claude Code | kloud | 状态 |
|------|------------|-------|------|
| 旋转 glyph | SpinnerGlyph | `active_glyph` | ✅ 已实现 |
| 离散 shimmer | ShimmerChar | `shimmer_word_spans` | ✅ 已实现 |
| SpinnerMode | 5 种模式 | 4 种模式 | ✅ 已实现 |
| 帧率 | 120ms | ~167ms (TICK_DIVISOR=3) | 🔄 可优化 |
| shimmer 速度 | 50/200ms | ~167ms | 🔄 可优化 |

### 待实现

| 功能 | Claude Code | kloud 状态 |
|------|------------|----------|
| 卡顿检测 | `useStalledAnimation` | ❌ 未实现 |
| Token 计数 | 平滑递增 | ❌ 未实现 |
| Thinking 呼吸 | 正弦波 opacity | ❌ 未实现 |
| 箭头状态图标 | SpinnerModeGlyph | ❌ 未实现 |
| 减少动画 | 静态点 | ❌ 未实现 |
| 工具闪烁 | flashOpacity | ❌ 未实现 |

---

## 12. 关键代码位置映射

```
Claude Code                              kloud 目标
─────────────────────────────────────────────────────────
SpinnerAnimationRow.tsx                   widgets.rs:render_live_assistant_header
  ├── SpinnerGlyph.tsx                  widgets.rs:active_glyph
  ├── GlimmerMessage.tsx                widgets.rs:shimmer_word_spans
  │   └── ShimmerChar.tsx              (内联在 shimmer_word_spans 中)
  ├── useShimmerAnimation.ts            constants.rs:shimmer 相关常量
  ├── useStalledAnimation.ts            state.rs:StalledState (待实现)
  └── utils.ts                         constants.rs:颜色和动画常量

types.ts (SpinnerMode 定义)              state.rs:SpinnerMode 枚举
```

---

## 13. Verb 与动作描述的切换逻辑

### 消息内容优先级

```typescript
// SpinnerWithVerbInner.tsx (Claude Code)
const leaderVerb = overrideMessage       // 1. 优先级最高：overrideMessage
  ?? currentTodo?.activeForm            // 2. 当前任务的动作形式
  ?? currentTodo?.subject               // 3. 当前任务的主题
  ?? randomVerb;                        // 4. 默认：从 SPINNER_VERBS 随机选择
const message = effectiveVerb + '…';
```

### 何时显示 Verb，何时显示动作描述

| 场景 | 显示内容 | 示例 |
|------|----------|------|
| 有 `overrideMessage` | `overrideMessage` | 权限请求时的自定义提示 |
| 有正在运行的任务 (`currentTodo.activeForm`) | 任务的 `activeForm` | "Reading files…" |
| 有正在运行的任务 (`currentTodo.subject`) | 任务的 `subject` | "Reading files" |
| 无任务 | 从 `SPINNER_VERBS` 随机选择 | "Processing…" |

### SPINNER_VERBS 列表

包含 **200+** 个趣味动词，分类如下：

| 类别 | 示例 |
|------|------|
| 动作类 | Processing, Executing, Performing |
| 创意类 | Creating, Crafting, Composing |
| 思考类 | Thinking, Considering, Pondering |
| 技术类 | Computing, Calculating, Hashing |
| 有趣类 | Beboppin', Canoodling, Flibbertigibbeting |

---

## 14. Thinking Shimmer 呼吸效果

### 颜色常量

```typescript
const THINKING_INACTIVE = { r: 153, g: 153, b: 153 };        // 暗灰色
const THINKING_INACTIVE_SHIMMER = { r: 185, g: 185, b: 185 }; // 亮灰色
const THINKING_DELAY_MS = 3000;         // 延迟 3 秒启动
const THINKING_GLOW_PERIOD_S = 2;      // 呼吸周期 2 秒
```

### 动画实现

```typescript
// SpinnerAnimationRow.tsx 第 196-200 行
const thinkingElapsedSec = (time - THINKING_DELAY_MS) / 1000;
const thinkingOpacity = time < THINKING_DELAY_MS
  ? 0
  : (Math.sin(thinkingElapsedSec * Math.PI * 2 / THINKING_GLOW_PERIOD_S) + 1) / 2;

const thinkingShimmerColor = toRGBColor(
  interpolateColor(THINKING_INACTIVE, THINKING_INACTIVE_SHIMMER, thinkingOpacity)
);
```

### 动画曲线

```
时间 →  0s      1s      2s      3s      4s      5s
       ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
opacity  0       0       0       1       0       1
         │       │       │       ↑       │       ↑
         └───────┴───────┴───────┴───────┴───────┘
         (thinkingOpacity 从 0 开始，3s 后启动正弦波)
```

### 渲染位置

Thinking shimmer 应用于状态区域的 **"thinking"** 文字，而非主消息：

```typescript
// SpinnerAnimationRow.tsx 第 210-214 行
...(showThinking && thinkingText ? [
  thinkingStatus === 'thinking' && !reducedMotion ? (
    <Text key="thinking" color={thinkingShimmerColor}>
      {thinkingOnly ? `(${thinkingText})` : thinkingText}
    </Text>
  ) : <Text dimColor key="thinking">{thinkingText}</Text>
] : [])
```

---

## 15. 参考常量汇总

```typescript
// 动画帧率
const GLYPH_FRAME_INTERVAL_MS = 120;     // glyph 旋转帧间隔
const SHIMMER_SPEED_REQUESTING_MS = 50;   // requesting 模式 shimmer 速度
const SHIMMER_SPEED_OTHER_MS = 200;      // 其他模式 shimmer 速度

// 卡顿检测
const STALL_DETECTION_MS = 3000;         // 3秒无响应触发
const STALL_FADE_DURATION_MS = 2000;     // 渐变到红色满值

// Thinking shimmer
const THINKING_DELAY_MS = 3000;          // 延迟3秒启动
const THINKING_GLOW_PERIOD_S = 2;       // 周期2秒
const THINKING_INACTIVE = {r:153, g:153, b:153};
const THINKING_INACTIVE_SHIMMER = {r:185, g:185, b:185};

// Token 显示
const SHOW_TOKENS_AFTER_MS = 30_000;      // 30秒后显示

// 宽度控制
const SEP_WIDTH = 3;                      // " · " 宽度
```

---

## 16. 下一步实施建议

### 优先级 1：卡顿检测 (useStalledAnimation)

```rust
// state.rs 新增结构
struct StalledState {
    last_response_length: usize,
    last_change_at: Instant,
    is_stalled: bool,
    stalled_intensity: f32,
}
```

### 优先级 2：Token 计数显示

```rust
// 平滑递增算法
let increment = match gap {
    0..70 => 3,
    70..200 => max(8, (gap * 0.15).ceil()),
    _ => 50,
};
```

### 优先级 3：Thinking 呼吸效果

```rust
// 正弦波 opacity
let elapsed = now - activity.started_at - THINKING_DELAY_MS;
let opacity = (elapsed as f32 / 1000.0 * std::f32::consts::PI / PERIOD).sin();
```
