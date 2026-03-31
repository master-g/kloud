# kloud TUI 下一步开发计划

基于对 Claude Code 源代码的深入分析，本文档记录关键发现和下一步实施计划。

## 🔍 关键发现

### Shimmer 实现差异

**Claude Code 实际实现**（离散颜色切换）：
```typescript
// ShimmerChar.tsx
const isHighlighted = index === glimmerIndex;
const isNearHighlight = Math.abs(index - glimmerIndex) === 1;
const shouldUseShimmer = isHighlighted || isNearHighlight;
const color = shouldUseShimmer ? shimmerColor : messageColor;
// 只有两种颜色状态，非此即彼
```

**kloud 当前实现**（离散颜色切换 ✅ 已完成）：
```rust
// widgets.rs:shimmer_word_spans
let shimmer_color = Color::Rgb(gradient.hot.0, gradient.hot.1, gradient.hot.2);
let message_color = Color::Rgb(gradient.base.0, gradient.base.1, gradient.base.2);
let is_near = distance <= 1;
let color = if is_near { shimmer_color } else { message_color };
```

**结论**：已与 Claude Code 一致，使用离散颜色切换。

---

### Activity Line 完整架构（Claude Code）

Claude Code 的 `SpinnerAnimationRow.tsx` 是一个高度优化的 50ms 动画循环组件：

```
┌─────────────────────────────────────────────────────────────┐
│  [Glyph]  [GlimmerMessage]  (status parts...)                 │
│    ↑         ↑                    ↑                          │
│   120ms      50/200ms      timer | tokens | thinking        │
│   帧动画     shimmer速度    动态显示/隐藏                     │
└─────────────────────────────────────────────────────────────┘
```

**核心功能模块**：

1. **SpinnerGlyph** (120ms 帧率)
   - 6 帧旋转图标: `["·", "✻", "✽", "✶", "✳", "✢"]`
   - 模式感知: `requesting` 显示 ↑，其他显示 ↓

2. **GlimmerMessage** (50ms/200ms 可调)
   - `requesting` 模式: 50ms 快速 shimmer，从左到右
   - 其他模式: 200ms 慢速 shimmer，从右到左

3. **状态区域**（响应式布局）
   - 经过时间计时器
   - Token 计数（平滑动画递增）
   - Thinking 状态（呼吸效果）
   - 根据终端宽度动态显示/隐藏

4. **Stalled 检测**
   - 3 秒无响应输出变红
   - 使用 `responseLengthRef` 跟踪变化

---

## 📋 下一步实施计划

### Phase 1: Activity Line 完善

#### 1.1 Spinner Glyph 动画系统

**参考文件**: `SpinnerGlyph.tsx`, `useAnimationFrame.ts`

**当前状态**: ✅ 基础帧动画已实现（`widgets.rs:active_glyph`），但缺少模式感知

**功能需求**:
- [x] 扩展 `ACTIVITY_FRAMES` 到 6 帧图标
- [ ] 添加 `SpinnerMode` 枚举（Requesting/Responding/ToolUse/Thinking）
- [ ] 根据模式显示不同图标（↑↓箭头或旋转glyph）
- [ ] 120ms 帧切换间隔

**关键代码位置**:
- `src/ui/constants.rs` - ✅ `ACTIVITY_FRAMES` 已定义
- `src/ui/tui/state.rs` - 🔄 需添加 `SpinnerMode` 到 `LiveActivity`
- `src/ui/tui/widgets.rs` - 🔄 需修改 `active_glyph` 函数

#### 1.2 Token 计数显示

**参考文件**: `SpinnerAnimationRow.tsx` lines 142-158

**当前状态**: ❌ 未实现

**功能需求**:
- [ ] 跟踪响应 token 数量
- [ ] 平滑递增动画（根据差距调整步长）
- [ ] 显示格式: "↓ 1,234 tokens"
- [ ] 30 秒后才显示

**递增算法**:
```rust
let increment = match gap {
    0..70 => 3,
    70..200 => max(8, gap * 0.15),
    _ => 50,
};
```

#### 1.3 Stalled 状态检测

**参考文件**: `useStalledAnimation.ts`

**当前状态**: ❌ 未实现

**功能需求**:
- [ ] 跟踪最后响应长度和变化时间
- [ ] 3 秒无变化触发 stalled 状态
- [ ] stalled 时应用红色 tint
- [ ] 有活跃工具时重置计时器

**数据结构**:
```rust
struct StalledState {
    last_response_length: usize,
    last_change_at: Instant,
    is_stalled: bool,
}
```

#### 1.4 Thinking Shimmer 呼吸效果

**参考文件**: `SpinnerAnimationRow.tsx` lines 196-200

**当前状态**: ❌ 未实现

**功能需求**:
- [ ] 正弦波动画（周期 2 秒）
- [ ] 延迟 3 秒后启动
- [ ] 颜色在 inactive 和 shimmer 之间插值
- [ ] 颜色值: `(153,153,153)` → `(185,185,185)`

---

### Phase 2: 消息系统

#### 2.1 消息组件架构

**参考文件**: `UserTextMessage.tsx`, `AssistantTextMessage.tsx`, `MessageResponse.tsx`

**当前状态**: ❌ 未实现

**功能需求**:
- [ ] 创建 `MessageResponse` 包装器（⎿ 前缀）
- [ ] 消息类型路由（Plain/Plan/SlashCommand/BashOutput）
- [ ] 防止嵌套包装

**组件层次**:
```
MessageResponse
├── UserTextMessage
│   ├── UserPlanMessage
│   ├── SlashCommandUserMessage
│   └── BashOutputUserMessage
└── AssistantTextMessage
    ├── ThinkingBlock
    ├── ToolUseBlock
    └── TextContent
```

#### 2.2 Markdown 渲染

**参考文件**: `Markdown.tsx`

**当前状态**: ❌ 未实现

**功能需求**:
- [ ] 集成 `pulldown-cmark` crate
- [ ] Token LRU 缓存（最大 500）
- [ ] 快速路径：无 markdown 语法时直接渲染
- [ ] 支持代码块、列表、强调

#### 2.3 代码块高亮

**功能需求**:
- [ ] 使用 `syntect` 进行语法高亮
- [ ] 添加语言标识显示
- [ ] 可选：复制按钮（OSC 52）

---

### Phase 3: 任务系统

#### 3.1 BackgroundTask 组件

**参考文件**: `BackgroundTask.tsx`, `ShellProgress.tsx`

**当前状态**: ❌ 未实现

**功能需求**:
- [ ] 后台任务列表显示
- [ ] 支持任务类型：local_bash, remote_agent
- [ ] 状态映射: running/completed/failed/killed → 颜色
- [ ] 侧边栏或底部面板显示

#### 3.2 任务状态颜色

```rust
match status {
    "completed" => Color::Green,  // success
    "failed" => Color::Red,       // error
    "killed" => Color::Yellow,    // warning
    "running" => Color::Default,  // default
}
```

---

### Phase 4: 主题系统增强

#### 4.1 ThemeProvider

**参考文件**: `ThemeProvider.tsx`, `ThemedText.tsx`, `ThemedBox.tsx`

**当前状态**: 🔄 基础主题系统已实现

**功能需求**:
- [ ] 支持 'auto', 'dark', 'light' 模式切换
- [ ] 监听系统主题变化（OSC 11）
- [ ] 主题预览功能

#### 4.2 Ratchet 高度锁定

**参考文件**: `Ratchet.tsx`

**当前状态**: ❌ 未实现

**功能需求**:
- [ ] 测量内容最大高度并锁定
- [ ] 防止布局抖动
- [ ] 支持 'always' 和 'when-visible' 模式

---

## 🔧 关键常量参考

```rust
// Animation timing
pub const TUI_FPS: u64 = 20;
pub const ACTIVITY_TICK_DIVISOR: u64 = 3;  // 当前值（已调整）
pub const GLYPH_FRAME_INTERVAL_MS: u64 = 120;
pub const SHIMMER_SPEED_REQUESTING_MS: u64 = 50;
pub const SHIMMER_SPEED_OTHER_MS: u64 = 200;

// Glyph frames
pub const ACTIVITY_FRAMES: [&str; 6] = ["·", "✻", "✽", "✶", "✳", "✢"];

// Stall detection
pub const STALL_DETECTION_MS: u128 = 3000;
pub const ACTIVITY_FADE_WINDOW_MS: u128 = 9500;
pub const ACTIVITY_MIN_RETAIN: f32 = 0.24;

// Thinking shimmer
pub const THINKING_DELAY_MS: u64 = 3000;
pub const THINKING_GLOW_PERIOD_S: f32 = 2.0;
pub const THINKING_INACTIVE: (u8, u8, u8) = (153, 153, 153);
pub const THINKING_INACTIVE_SHIMMER: (u8, u8, u8) = (185, 185, 185);

// Token display
pub const SHOW_TOKENS_AFTER_MS: u128 = 30_000;

// Width gating
pub const SEP_WIDTH: usize = 3;  // " · "
```

---

## 📁 文件映射

| Claude Code | kloud 目标位置 | 状态 |
|------------|---------------|------|
| `Spinner/ShimmerChar.tsx` | `widgets.rs:shimmer_word_spans` | ✅ 已完成 |
| `Spinner/SpinnerAnimationRow.tsx` | `widgets.rs:animated_verb_spans` | 🔄 需扩展 |
| `Spinner/SpinnerGlyph.tsx` | `widgets.rs:active_glyph` | 🔄 需增加模式感知 |
| `Spinner/useShimmerAnimation.ts` | `constants.rs` | ✅ 已实现 |
| `Spinner/useStalledAnimation.ts` | `state.rs:StalledState` | ⏳ 待实现 |
| `messages/UserTextMessage.tsx` | `messages.rs` | ⏳ 待创建 |
| `messages/AssistantTextMessage.tsx` | `messages.rs` | ⏳ 待创建 |
| `MessageResponse.tsx` | `widgets.rs:MessageResponse` | ⏳ 待实现 |
| `tasks/BackgroundTask.tsx` | `tasks.rs` | ⏳ 待创建 |
| `design-system/ThemeProvider.tsx` | `theme.rs` | 🔄 基础已实现 |
| `design-system/Ratchet.tsx` | `widgets.rs:Ratchet` | ⏳ 待实现 |

---

## 🎯 推荐实施顺序

### 立即开始（M2.2 延续）
1. **Spinner Glyph 模式感知** - 1-2 小时
   - 添加 `SpinnerMode` 枚举
   - 实现 ↑↓ 箭头切换

2. **Token 计数显示** - 1-2 小时
   - 添加 token 计数跟踪
   - 实现平滑递增动画

### 下周（M3 准备）
3. **Stalled 状态检测** - 1-2 小时
4. **Thinking Shimmer** - 2 小时
5. **消息组件架构** - 4-6 小时

### 后续
6. Markdown 渲染
7. 任务系统
8. 主题系统增强

---

## 📚 参考文档

- Claude Code 源码分析: `/Users/mg/.claude/plans/kloud-tui-roadmap.md`
- 详细规划: `/Users/mg/.claude/plans/structured-stargazing-dongarra.md`
