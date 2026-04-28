## Context

`render_tool_use_line()` (tool_block.rs:22-72) 在 `fix-tool-name-double-render` 后引入 3 个 review findings：
1. L58-68：`if let Some(first) = iter.next() { ... }` 后紧跟 `for line in iter`，后者实际在 if 块外但始终安全（iter 此时已消耗 first）。控制流误导读者。
2. L64：手动 `for span in &first.spans { spans.push(Span::styled(content.to_string(), style)) }` 重建 spans，与 `prepend_prefix_to_line()` 做相同 clone 工作。首行无法复用 `prepend_prefix_to_line` 因为前缀是 dot span（styled）而非 plain text。
3. L45-55：空 rendered fallback 用 raw name（"read"），缺少注释说明这是有意行为。

## Goals / Non-Goals

**Goals:**
- 修正控制流结构，消除误导
- 减少冗余 clone（首行也用 `prepend_prefix_to_line`，dot span 单独添加）
- 添加 fallback 注释

**Non-Goals:**
- 不改函数签名
- 不改 `prepend_prefix_to_line` 签名
- 不改 Tool trait 或 DisplayBlock

## Decisions

### Decision 1: 控制流合并

**选择：** `if let Some(first) = iter.next()` 块内包含首行渲染 + `for line in iter` 循环。

```rust
let mut iter = rendered.iter();
if let Some(first) = iter.next() {
    // render first line with dot
    ...
    // subsequent lines
    for line in iter {
        ...
    }
}
```

**理由：** 一眼看出 `for` 依赖 `if let` 成功。

### Decision 2: 首行渲染复用 prepend_prefix_to_line

**选择：** 首行也用 `prepend_prefix_to_line(first, "  ", ...)` 得到 `Line`，然后手动把 dot span 插入到该 Line 的 spans 开头（替换 "  " 前缀为 dot + " "）。或者更简单：先推入 dot span，再用 `prepend_prefix_to_line` 处理剩余 spans。

实际最简方案：保持手动构建首行 spans（已有逻辑），但把 clone 限制在必要位置。`prepend_prefix_to_line` 本身也 clone——改首行用它不会减少 clone。所以 L64 不是真正的性能问题，只是风格 nit。

**最终选择：** 保持手动 span 构建不变，只修正控制流 + 加注释。

## Risks / Trade-offs

- 无功能变更风险，纯重构
