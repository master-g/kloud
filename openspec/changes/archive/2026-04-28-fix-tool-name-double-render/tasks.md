## 1. 布局逻辑重写

- [x] 1.1 修改 `src/ui/tui/widgets/tool_block.rs` 的 `render_tool_use_line()`：去掉独立 header 行构建逻辑，改为把 dot 前缀合并到 rendered 第一行 spans 前面
- [x] 1.2 在 `render_tool_use_line()` 中添加空 rendered 回退路径：`rendered.is_empty()` 时用 `dot + display_name`（保留 server_name 拼接）
- [x] 1.3 在 `render_tool_use_line()` 的 rendered 非空路径中，当 `server_name.is_some()` 时，在第一行 rendered spans 前插入 `server_name - ` 前缀

## 2. 验证

- [x] 2.1 `cargo check` 通过
- [x] 2.2 `cargo clippy -- -W warnings` 通过
- [x] 2.3 `cargo test` 全部通过
- [x] 2.4 `cargo fmt --all` 格式化
