## Context

kloud 的工具渲染集中在 `tool_block.rs` 的两个通用函数：`render_tool_use_line()` 和 `render_tool_result_block()`。它们通过 `classify_tool(name)` 做粗粒度样式分类（Shell/File/Web/Other），无法按工具语义渲染。

CC 的做法：每个 Tool 实现 `userFacingName()`、`renderToolUseMessage()`、`renderToolResultMessage()` 等方法。渲染组件只负责调度（找 tool → 调方法 → 包布局），具体绘制委托给工具自身。

当前 kloud Tool trait (`tools/traits.rs`) 只有 `name()`, `description()`, `input_schema()`, `execute()` 四个方法。`ToolResult` 是 `Result<String, String>`，无法区分 cancel/reject/error。

**关键架构约束**：TUI 层只接收 `SessionView`（纯数据快照），不持有 `ToolRegistry`。渲染发生在 `SessionStore.apply()` reducer 中（有 registry 访问权）。因此 Tool trait 的渲染方法在 store 层调用，预计算结果存入 `DisplayBlock`，TUI 直接使用。

## Goals / Non-Goals

**Goals:**
- Tool trait 增加渲染方法，每个 builtin tool 自定义渲染
- ToolResult 支持 Success / Error / Canceled / Rejected 四种状态
- 工具输入预览从预生成字符串改为结构化渲染
- 工具执行期间 emit progress 消息，TUI 实时显示
- tool_block.rs 从渲染实现者变为纯布局层（点 + 折叠 + 前缀）

**Non-Goals:**
- 分组渲染（同名工具合并显示）— 留到 M3
- Transparent wrapper 工具 — 留到 M3
- Tool JSX 交互式组件 — 留到 M5
- Permission 渲染（renderToolUseRejectedMessage）— 随 permission 系统一起做
- outputSchema 验证 — 留到工具系统更成熟时

## Decisions

### D1: 渲染方法放在 Tool trait 上，带默认实现

**选择**：在 `Tool` trait 添加渲染方法，带默认实现（返回通用渲染）。Builtin tool 按需 override。

**替代方案**：
- (A) 独立 `ToolRenderer` trait — 增加一层间接，每工具需两个 impl，碎片化
- (B) 函数注册表 `HashMap<String, RenderFn>` — 失去类型安全，与 trait 系统不一致

**理由**：与 CC 架构一致，默认实现保证旧工具不需要改代码。一个 trait 管所有行为，简单。

### D2: ToolResultKind 枚举

**选择**：新增枚举，`ToolResult` 的 `output` 字段从 `Result<String, String>` 改为 `(ToolResultKind, String)`。

```rust
pub enum ToolResultKind {
    Success,
    Error,
    Canceled,
    Rejected,
}

pub struct ToolResult {
    pub name: String,
    pub kind: ToolResultKind,
    pub output: String,
}
```

**替代方案**：
- (A) 保持 `Result<String, String>` + 额外 `canceled: bool` 字段 — 语义不清
- (B) 枚举值直接放 Result 里 — 不自然

**理由**：四态枚举精确建模 CC 的路由逻辑。`Result<String, String>` 只有 ok/err 两态，不够。

### D3: Progress callback 引用闭包

**选择**：`execute()` 签名增加 `on_progress: Option<&dyn Fn(&str)>` 参数。

**替代方案**：
- (A) `Box<dyn Fn(&str)>` — 需要 'static 生命周期，增加分配
- (B) mpsc channel — 需要管理 channel 生命周期，execute 签名变复杂
- (C) 事件发射器 trait — 过度抽象

**理由**：引用闭包最轻量。`execute(&self, call, on_progress)` 的调用者（`execute_tool_call`）在栈上持有闭包，`execute` 是短暂异步调用，引用生命周期自然满足。不需要 Box 分配。

### D4: 渲染输出类型

**选择**：渲染方法返回 `Vec<Line<'static>>`（ratatui Line）。

**替代方案**：
- (A) 返回 `String` — 无法表达样式
- (B) 返回自定义 enum 再转 Line — 增加转换层

**理由**：直接返回 ratatui 类型，最简单。`tools` 模块是主 crate 的一部分（不是独立 crate），ratatui 已在依赖中。

### D5: 预渲染策略（store 层渲染，TUI 层展示）

**选择**：Tool 渲染方法在 `SessionStore.apply()` 中调用。预计算的 `Vec<Line<'static>>` 存入 `DisplayBlock`。TUI 层（`tool_block.rs`、`messages.rs`）只负责布局（状态点 + 折叠 + 前缀），不做内容渲染。

**替代方案**：
- (A) TUI 层持有 `Arc<ToolRegistry>` — 打破纯数据快照架构，`SessionView` 变重
- (B) 渲染方法做成静态函数 `fn render_use(name, input) -> Vec<Line>` — 失去 per-tool 多态
- (C) 注册时预编译渲染器 — 过度工程

**理由**：`SessionStore` 已持有 `Arc<ToolRegistry>`（通过 Session 间接访问），在 `apply()` 中调用 Tool 渲染方法自然且零额外耦合。TUI 保持纯展示层。

**数据流**：
```
SessionEvent::ToolExecutionStarted
  → store.apply()
  → registry.get(&name)
  → tool.render_tool_use_message(&input)
  → DisplayBlock::ToolUse { rendered: Vec<Line>, ... }

SessionEvent::ToolExecutionFinished
  → store.apply()
  → registry.get(&name)
  → tool.render_tool_result_message(&output, kind)
  → DisplayBlock::ToolResult { rendered: Vec<Line>, kind, ... }

TUI (messages.rs)
  → DisplayBlock::ToolUse { rendered, status, ... }
  → prepend status dot + rendered lines
  → DisplayBlock::ToolResult { rendered, kind, ... }
  → apply collapse + prefix ⎿ + rendered lines
```

## Risks / Trade-offs

- **[Risk] Tool trait 变大** → 默认实现控制膨胀。渲染方法都有默认 fallback，不实现也能工作。
- **[Risk] ToolResult 破坏现有代码** → `ToolResultKind` 替换 `Result<String, String>` 是 breaking change，需一次改完 `call.rs` + 所有 `execute()` impl + `execute_tool_call()` 解构 + 全部测试断言。
- **[Risk] Progress 引用闭包生命周期** → `execute()` 是 `&self` 方法，调用者在栈上持有闭包并传入 `&dyn Fn`，生命周期自然满足。但如果未来 execute 变成 `'static` task 则需改为 `Box`。当前架构无此风险。
- **[Risk] Store 层需要 ToolRegistry** → 当前 `SessionStore` 不直接持有 registry，`Session` 持有。需在 store 上新增 `render_with()` 方法或在 apply 事件时由 Session 传入渲染结果。方案：`SessionEvent` 新增可选的 `rendered` 字段，由 Session（持有 registry）在 emit 前预渲染。
- **[Trade-off] 渲染方法耦合 ratatui** → 接受。kloud 只有一个 TUI 后端，不需要抽象。
- **[Trade-off] DisplayBlock 变大** → 预渲染的 `Vec<Line<'static>>` 增加 clone 开销。可接受——消息数量有限（<1000），且 ratatui Line 的 clone 是 Vec clone（shared content via Arc）。
