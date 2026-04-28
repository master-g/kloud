## 1. 控制流修正

- [x] 1.1 在 `src/ui/tui/widgets/tool_block.rs` 的 `render_tool_use_line()` 中，把 `for line in iter` 循环移入 `if let Some(first) = iter.next()` 块内部
- [x] 1.2 给 `rendered.is_empty()` 分支加注释：说明 fallback 使用 raw name 是有意行为（无 tool render method 产出内容时的兜底）

## 2. 验证

- [x] 2.1 `cargo check` 通过
- [x] 2.2 `cargo clippy -- -W warnings` 通过
- [x] 2.3 `cargo test` 全部通过
- [x] 2.4 `cargo fmt --all` 格式化
