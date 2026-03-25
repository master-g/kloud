# kloud 下一阶段实现清单

> 目的：把当前 `M2.2 -> M3` 过渡阶段的工作拆成一份可执行的实现清单。
>
> 这不是长期路线图。长期目标看 `docs/ROADMAP.md`，当前上下文看 `TODO.md`。

---

## 当前评估

按当前代码而不是按旧路线图判断，项目位置更接近：

- `M1` 已完成
- `M2.1` 已完成
- `M2.2` 已完成一半以上：`ReadTool` 和最小 tool-use 闭环都已打通
- `M3` 还未正式命名开始，但已经有明显前奏

当前最重要的事实不是“功能还少”，而是：

1. `Session` 已经具备了最小 Agent Loop 的核心语义
2. 工具能力还不完整，尤其缺少写文件能力
3. 高层集成测试仍然不足，当前测试更多是 helper 层和局部状态转移
4. `stream.rs` / `tools.rs` 体量已经开始提示状态模型需要收束

因此，下一阶段的重点应该是：

1. 先把 `M2.2` 补到一个更完整的“读 + 写 + tool loop”闭环
2. 再正式整理 `M3` 的状态机与 Agent Loop 语义

不建议的顺序：

- 现在继续投入 TUI 打磨
- 现在就做 `BashTool`
- 现在就做子代理、并发、持久化

---

## 目标顺序

建议严格按下面顺序推进：

1. 为写操作补路径安全 helper
2. 实现 `WriteTool` v1
3. 增加 `Session::run()` 高层集成测试
4. 收束 `Session` 当前状态机和模块边界
5. 正式进入 `M3.1 / M3.2`

推荐按 5 个 commit 推进：

1. `refactor(tools): add writable path resolution helper`
2. `feat(tools): implement write tool`
3. `test(session): add end-to-end tool loop coverage`
4. `refactor(session): clarify streaming and tool-loop transitions`
5. `feat(agent): formalize agent state model`

---

## 阶段一：补写路径安全边界

### 目标

当前 `resolve_existing_path()` 的语义偏向“读现有文件”，不适合直接复用到“写新文件”。

写工具需要的不是“目标必须存在”，而是：

- 用户传入的路径必须是相对路径
- 目标必须位于 workspace root 内
- 目标文件可以尚不存在
- 父目录可能需要创建

### 建议新增内容

优先在 `src/tools/path.rs` 中增加一个新的 helper，而不是修改现有 helper 到面目模糊。

建议方向：

- 保留现有 `resolve_existing_path(root, user_path)` 给 `ReadTool`
- 新增一个更适合写操作的 helper

可以考虑的命名：

- `resolve_writable_path`
- `resolve_path_within_root`
- `resolve_target_path`

其中我更推荐 `resolve_writable_path`，因为它直接表达用途。

### 建议语义

输入：

- `root: &Path`
- `user_path: &str`

输出：

- 返回一个已经确认位于 `root` 内的目标 `PathBuf`

核心约束：

- 拒绝绝对路径
- 拒绝空路径
- 拒绝 `..` 导致的目录逃逸
- 允许文件本身不存在

### 实现提示

关键点在于：不能直接对“目标文件本身”做 `canonicalize()`，因为它可能还不存在。

更稳妥的办法是：

1. 先 `canonicalize(root)`
2. 把 `user_path` 拼接到 `root`
3. 如果目标存在，可以直接 `canonicalize(target)`
4. 如果目标不存在，则对“最近存在的父目录”做 `canonicalize()`，再检查它是否仍在 root 内
5. 最后返回拼好的目标路径

### 你要特别想清楚的问题

这里有一个很好的 Rust + Agent 工程练习题：

> 为什么“读路径安全”和“写路径安全”不能完全复用同一个 helper？

你应该能回答：

- 因为读操作依赖目标已经存在
- 写操作的目标常常还不存在
- 写操作真正要保护的是“目标落点是否仍在 root 内”

### 相关文件

- `src/tools/path.rs`
- `src/error.rs`

### 验收标准

- 相对路径可以成功解析
- 绝对路径被拒绝
- `../outside.txt` 被拒绝
- `nested/new/file.txt` 可以解析，即使文件尚不存在
- 如果父目录在 root 外，也必须拒绝

### 推荐测试

至少补这些测试：

- `resolve_writable_path_allows_new_file_under_root`
- `resolve_writable_path_rejects_absolute_path`
- `resolve_writable_path_rejects_escape`
- `resolve_writable_path_allows_existing_file`

---

## 阶段二：实现 `WriteTool` v1

### 目标

先实现最小、稳固、可测试的写文件工具，不要一开始就做成复杂文件编辑器。

v1 的职责应该非常窄：

- 创建文件
- 覆盖文件
- 必要时创建父目录

不要在 v1 做的事：

- append
- patch
- diff
- 行级编辑
- 多文件批量写入

### 建议接口

文件：

- 新增 `src/tools/builtin/write.rs`
- 更新 `src/tools/builtin/mod.rs`

建议输入 schema：

- `path: string`
- `content: string`

建议输出：

- 保持 text-first
- 返回类似：
  - `Wrote 128 bytes to path/to/file.rs`
  - 或 `Created path/to/file.rs (3 lines)`

不建议现在输出结构化 JSON。当前 repo 的工具输出策略已经明显偏向 text-first，和 `ReadTool` 保持一致更重要。

### 实现提示

建议实现流程：

1. 从 `ToolCall.args` 取出 `path`
2. 从 `ToolCall.args` 取出 `content`
3. 调用新的 writable path helper
4. `create_dir_all(parent)`，如果父目录不存在
5. 写入内容
6. 返回一条紧凑的人类可读结果

### 设计问题

这里值得思考一个问题：

> `WriteTool` 应该默认覆盖，还是遇到已有文件时报错？

我建议 v1 默认覆盖，原因：

- 语义简单
- 更接近真实 agent coding 工具的最小可用行为
- 避免把“冲突确认”复杂度过早引入

但你应该在文档或注释里承认它的 tradeoff：

- 未来如果接入更强的编辑能力，可能需要区分 create / overwrite / patch

### 相关文件

- `src/tools/builtin/write.rs`
- `src/tools/builtin/mod.rs`
- `src/tools/traits.rs`
- `src/tools/path.rs`

### 验收标准

- 能创建新文件
- 能覆盖已有文件
- 父目录不存在时可自动创建
- 缺失 `path` 参数时报错
- 缺失 `content` 参数时报错
- 路径逃逸时报错

### 推荐测试

至少补这些测试：

- `test_write_tool_creates_file`
- `test_write_tool_overwrites_existing_file`
- `test_write_tool_creates_parent_dirs`
- `test_write_tool_missing_path`
- `test_write_tool_missing_content`
- `test_write_tool_rejects_path_escape`

### 完成后立刻检查

执行：

```bash
cargo fmt --all
cargo clippy -- -W warnings
cargo test write_tool -- --nocapture
cargo test --lib
```

---

## 阶段三：补高层 `Session::run()` 集成测试

### 目标

当前你已经有不少局部测试，但还缺最关键的一层：

> 从 UI action 出发，完整经过 Session 外循环、stream 处理、tool dispatch、再次请求模型、最终回到 assistant 文本输出。

这一步很重要，因为它会验证真正的 Agent Loop 契约，而不是某个 helper 函数的局部正确性。

### 为什么这一步值得优先做

如果现在先继续堆工具，而不补这层测试，你后面会遇到两类问题：

1. helper 层都对，但拼起来不对
2. 重构状态机时，没有高层安全网

### 建议测试形状

建议写一个专用 stub client，而不是复用太多已有 test helper。

理想测试流程：

1. 启动 `Session`
2. 通过 `action_tx` 发送 `UiAction::SendMessage`
3. stub client 第一次返回一个 `tool_use`
4. session 执行真实注册的 `EchoTool` 或 `WriteTool`
5. stub client 第二次收到 `tool_result` 后，返回最终文本
6. 断言 UI event 序列和最终 history

### 推荐优先覆盖的路径

第一条高层测试只测 happy path：

- user text
- model tool_use
- tool dispatch success
- second model response
- final assistant turn end

第二条再补 error path：

- model tool_use
- tool dispatch failure
- tool_result `is_error = true`
- second model still continues and gives final reply

### 可以断言什么

至少断言这些：

- `AssistantTurnStart` 被发送
- 出现 `ToolUseStart`
- 出现 `ToolResult`
- 最终出现 `TextDelta`
- 最终出现 `AssistantTurnEnd`

如果你想更严格：

- 校验 event 顺序
- 校验第二轮请求确实携带了 `tool_result` message
- 校验 `Session.messages` 中 assistant/tool_result 的持久化顺序

### 相关文件

- `src/app/session/run.rs`
- `src/app/session/stream.rs`
- `src/app/session/tools.rs`
- `src/app/session/types.rs`
- `src/ui/events.rs`

### 验收标准

- 至少 1 条 happy-path e2e 集成测试
- 最好再加 1 条 tool error continuation 测试
- 重构 `stream.rs` / `tools.rs` 时，这些测试能充当高层回归保护

---

## 阶段四：收束当前 Session 状态机

### 目标

这一步先不新增能力，只做语义收束和模块整理。

当前最明显的信号是：

- `src/app/session/stream.rs` 很大
- `src/app/session/tools.rs` 也很大
- `TurnState` 的注释已经在描述“内循环 Agent runtime”

这说明项目已经自然长到了需要更明确状态边界的时候。

### 本阶段不要做什么

- 不要马上把所有东西重命名成 `agent`
- 不要引入并发
- 不要引入子代理
- 不要做持久化

这一步的目标是“让现在已有的模型更清楚”，不是“架构大跃进”。

### 建议拆解任务

1. 先写清楚状态边界
   - `Session` 持有什么长期状态
   - `TurnState` 持有什么短期状态
   - 哪些状态只存在于单个 assistant turn 内

2. 再看 helper 是否该按职责继续拆
   - stream event 翻译
   - pending block finalize
   - tool call extraction
   - tool dispatch / continuation

3. 最后再做小规模重构
   - 让每个函数只表达一个状态转移步骤

### 你要回答的关键问题

> 当前代码里，“outer loop” 和 “inner loop” 的真实边界在哪里？

你应该能给出近似这样的答案：

- `Session::run()` 是 outer loop
- 单次 assistant streaming + tool continuation 是 inner loop
- `TurnState` 是 inner loop 的 runtime snapshot

如果你现在还不能一口气说清楚，那说明 M3 还不该开工，应该先做这一步整理。

### 相关文件

- `src/app/session/mod.rs`
- `src/app/session/run.rs`
- `src/app/session/stream.rs`
- `src/app/session/tools.rs`
- `src/app/session/types.rs`

### 验收标准

- 大函数职责更清晰
- 模块边界更清晰
- 注释从“描述现象”变成“解释状态转移语义”
- 高层集成测试仍然全部通过

---

## 阶段五：正式开始 M3

### 目标

在完成前四步后，再正式把当前设计提升为路线图里的：

- `M3.1 Agent 状态机`
- `M3.2 Agent Loop`

### 这一步的真实任务

不是新增很多功能，而是：

- 给已有设计一个更准确的名字
- 让状态模型成为代码的主结构，而不是隐藏在 helper 之间

### 建议切入方式

优先从类型和状态转移文档开始，再决定是否调整模块名。

建议先做：

1. 重新梳理消息模型
2. 明确 assistant turn 中的 block 生命周期
3. 明确 stop reason 如何驱动下一步动作
4. 明确 tool-use continuation 的状态转移表

再决定是否需要：

- 新建 `src/agent/`
- 把 `Session` 迁移成更贴近 agent 的命名

我的建议是：先把语义定准，再改目录结构。

### 你应该得到的学习收获

这一阶段的重点不是“我又做了一个 feature”，而是：

- 理解为什么 agent runtime 是状态机问题
- 理解为什么 message history 是 append-only log
- 理解为什么 tool call / tool result / stop reason 是 loop control，而不是普通 UI 事件

### 验收标准

- 能清楚画出当前 agent loop 的状态转移
- 关键状态和不变量都能在代码里找到明确归属
- 不用读 700 行文件，也能解释一次完整的 tool-use turn 是怎么流动的

---

## 暂缓事项

这些事现在先不要做，或者至少不要抢在前面：

- `EditTool`
- `GlobTool`
- `BashTool`
- 真正的 turn cancellation
- 会话持久化
- parallel tool use
- 子代理 / DAG / 技能系统

其中 `BashTool` 尤其不建议现在做，因为它会一次性引入：

- 超时
- 取消
- stdout/stderr 截断
- 安全边界
- 更复杂的测试环境

这些都是真的重要，但不是现在最值得学的内容。

---

## 推荐工作流

每完成一个阶段，至少执行：

```bash
cargo fmt --all
cargo clippy -- -W warnings
cargo test --lib
```

在做 session / tool loop 改动时，额外建议：

```bash
cargo test session -- --nocapture
cargo test tools -- --nocapture
```

---

## 最终建议

如果你接下来只做三件事，那就做这三件：

1. `resolve_writable_path`
2. `WriteTool`
3. `Session::run()` 高层集成测试

这是当前阶段性价比最高、学习价值也最高的三步。

做完这三步，再进入 `M3`，你会清楚很多。
