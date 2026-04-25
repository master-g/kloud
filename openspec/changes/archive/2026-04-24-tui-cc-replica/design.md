## Context

kloud 当前 TUI 基于 ratatui + crossterm，有基础的消息显示、单行输入、20fps 渲染循环。与 Claude Code（Ink/React TUI）对比缺少：虚拟滚动、多行输入、主题系统、流式动画、工具折叠、搜索等。

Claude Code 源码在 `docs/cc/`，作为复刻参考。kloud 用 Rust，不能直接移植 Ink 组件，需要用 ratatui 的 widget/layout 系统重新实现相同视觉效果。

当前 `src/ui/` 结构：
```
src/ui/
├── mod.rs, backend.rs, events.rs, stdio.rs
└── tui/
    ├── mod.rs      # 事件循环 (tokio::select!)
    ├── state.rs    # TuiState, DisplayMessage, DisplayBlock
    ├── widgets.rs  # 渲染函数
    └── input.rs    # 按键映射
```

## Goals / Non-Goals

**Goals:**

- 像素级复刻 Claude Code 的视觉风格（颜色、布局、动画）
- 虚拟滚动支持数千条消息无卡顿
- 多行输入框支持历史、Vim 模式、斜杠命令补全
- 6 种主题（dark/light × 3 变体）
- 流式渲染动画（shimmer、stall、token 计数）
- 工具调用可折叠/展开、批量分组
- 消息搜索和高亮

**Non-Goals:**

- 不实现鼠标支持（Claude Code 有但复杂度高）
- 不实现图片显示（终端限制）
- 不实现命令面板（fuzzy finder 需要额外依赖）
- 不实现 voice 模式
- 不实现 plugin/extension 系统
- 不重写 session/store 层——只扩展事件协议

## Decisions

### D1: 组件化 widget 架构

**选择**: 拆分 `widgets.rs` 为独立组件模块（`components/` 目录）

**替代方案**: 保持单文件 + 函数。被否决——当前 widgets.rs 已 300+ 行，继续增长不可维护。

**结构**:
```
src/ui/tui/components/
├── mod.rs           # 组件注册和重导出
├── messages.rs      # 虚拟滚动消息列表
├── input.rs         # 多行输入框
├── status_bar.rs    # 底部状态栏
├── tool_display.rs  # 工具调用渲染
├── activity.rs      # 活动行动画
└── search.rs        # 搜索栏和匹配高亮
```

### D2: 主题系统设计

**选择**: 编译时主题 + 运行时切换（通过配置文件）

**结构**:
```rust
struct Theme {
    name: &'static str,
    colors: ThemeColors,
}

struct ThemeColors {
    claude: Color,        // 品牌 orange
    suggestion: Color,    // 蓝色链接
    text: Color,
    text_bold: Color,
    subtle: Color,
    inactive: Color,
    success: Color,
    error: Color,
    warning: Color,
    tool: Color,
    // ... 20+ 语义颜色
}
```

**替代方案**: 使用 ratatui 的 Style 直接硬编码。被否决——无法支持多主题。

### D3: 虚拟滚动策略

**选择**: 基于可见行数计算 + 行高缓存

```rust
struct VirtualScroll {
    messages: Vec<DisplayMessage>,
    visible_range: (usize, usize),  // 当前可见消息索引范围
    line_cache: Vec<usize>,         // 每条消息的行高缓存
    scroll_offset: u16,             // 垂直偏移
}
```

**理由**: ratatui 没有 virtual list 原语，需要自建。行高缓存避免每帧重算。

### D4: 动画引擎

**选择**: 50ms tick (20fps) + 时间线驱动

当前已有 50ms activity tick。扩展为统一的 `AnimationEngine`:

```rust
struct AnimationEngine {
    tick: Instant,
    shimmer_phase: f32,      // shimmer 左右扫描
    breathing_phase: f32,     // thinking 呼吸
    fade_alpha: f32,          // 活动行淡出
    stalled: bool,            // 3s 无 token
}
```

**理由**: 集中管理所有动画状态，避免散落在各 widget 中。20fps 够用——Claude Code 的 Ink 也是类似刷新率。

### D5: 多行输入

**选择**: 自建 `TextArea` widget，不用外部 crate

```rust
struct TextArea {
    lines: Vec<String>,
    cursor: (usize, usize),  // (row, col)
    history: Vec<String>,
    history_pos: usize,
    mode: InputMode,         // Normal | Insert | Vim
}
```

**替代方案**: 用 `tui-textarea` crate。被否决——需要精确控制渲染以匹配 Claude Code 风格，外部 crate 难以定制。

### D6: 工具调用折叠

**选择**: 基于 DisplayBlock 的 collapsible state

```rust
struct ToolGroup {
    tool_uses: Vec<DisplayBlock>,
    results: Vec<DisplayBlock>,
    collapsed: bool,
    elapsed: Duration,
}
```

同一批并发工具调用自动分组。用户按 Tab 展开/折叠。

## Risks / Trade-offs

- **[虚拟滚动复杂度]** → 先实现简单的 offset-based 分页，后续优化为真正的虚拟化。消息数 <1000 时全量渲染即可。
- **[动画性能]** → 50ms tick 在大量消息时可能掉帧。缓解：只重绘脏区域（ratatui 的 diff 机制）。
- **[多行输入边界]** → 终端输入处理（crossterm 事件）在不同终端模拟器上行为不一致。缓解：优先支持 iTerm2/Terminal.app/Alacritty，记录已知问题。
- **[主题颜色精度]** → 256 色终端无法显示 Claude Code 的 RGB 颜色。缓解：提供 ANSI fallback 配色。
- **[Vim 模式范围]** → 完整 Vim 模式工作量巨大。缓解：MVP 只支持 h/j/k/l/i/Esc/:wq 基本操作。

## Migration Plan

分阶段实现，每阶段可独立测试：

1. **Phase 1**: 主题系统 + 颜色迁移（不改布局）
2. **Phase 2**: 组件拆分 + 多行输入
3. **Phase 3**: 虚拟滚动 + 搜索
4. **Phase 4**: 动画引擎 + shimmer/stall
5. **Phase 5**: 工具调用折叠/分组
6. **Phase 6**: 状态栏完整实现

每阶段完成后 `cargo test` + 手动验证。

## Open Questions

- 是否需要 `syntect` 做代码语法高亮？增加编译时间和二进制大小。
- Vim 模式应该用 `modular-bitfield` 的 bitflags 还是简单 enum？
- 搜索是否需要正则支持？先用 substring 匹配。
