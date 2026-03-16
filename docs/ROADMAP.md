# kloud 学习路线图

> 通过构建一个 Agentic Coding CLI，掌握 Agent 工程原理和 Rust 编程。

## 路线图总览

```
里程碑 1: 能跑起来
  M1.1 LLM 对话 (API 调用 + 流式响应)  ✅
  M1.2 REPL 交互循环                    ← 你在这里

里程碑 2: 能用工具
  M2.1 工具框架 (registry + dispatch)
  M2.2 文件工具 (read/write/edit)
  M2.3 命令工具 (bash)
  M2.4 搜索工具 (glob/grep)

里程碑 3: 能思考
  M3.1 Agent 状态机
  M3.2 Agent Loop (双循环核心)
  M3.3 事件系统

里程碑 4: 能记忆
  M4.1 上下文管理
  M4.2 对话压缩
  M4.3 会话持久化

里程碑 5: 能协调
  M5.1 子 Agent 与委派
  M5.2 任务 DAG
  M5.3 技能系统

里程碑 6: 能并发
  M6.1 异步任务队列
  M6.2 后台执行器
  M6.3 结果收集

里程碑 7: 能协作
  M7.1 Agent 团队通信
  M7.2 团队生命周期
  M7.3 Worktree 隔离
```

---

## 里程碑 1: 能跑起来

> **目标**: 让 kloud 能连接 LLM，发送消息，接收流式回复，并在终端交互。
>
> 这是最小可用原型——你输入一句话，LLM 回复一段话。

### M1.1 LLM 对话

**你要做的事**:
- 创建 `src/llm/` 模块
- 定义 `LlmClient` trait（核心抽象）
- 实现 `OpenAiClient`（调用 OpenAI 兼容 API）
- 实现 `MockClient`（返回预设回复，用于测试）

**涉及文件**: `src/llm/mod.rs`, `src/llm/client.rs`, `src/llm/models.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `async trait` | LLM 调用是异步的，trait 中的 async 方法需要特殊处理 | 了解 `async-trait` crate 的作用，以及为什么标准 Rust trait 不直接支持 async |
| `reqwest` 异步 HTTP | 发送 HTTP 请求到 API | 关注 `Client::post().json().send().await` 的链式调用 |
| `serde` 反序列化 | API 返回 JSON 需要反序列化为 Rust 结构体 | `#[serde(rename_all = "snake_case")]` 和 `#[serde(default)]` 很常用 |
| `Stream` + 流式响应 | SSE (Server-Sent Events) 实现流式输出 | 看 `reqwest` 的 `bytes_stream()` 或逐行读取 |
| 错误处理链 | HTTP 错误 → API 错误 → 应用错误 | 用 `#[from]` 自动转换，用 `map_err` 手动转换 |

**要学的 Agent 知识**:
- **Chat Completion API 的结构**: messages 数组、role (system/user/assistant)、tool_calls
- **流式 vs 非流式**: 为什么 Agent 需要流式响应（用户体验 + 可以边生成边执行）
- **Mock 的价值**: 为什么 Agent 开发中 Mock LLM 是必须的（省钱、可测试、可离线开发）

**验收**: `cargo test` 中 MockClient 能返回预设回复；用真实 API key 能完成一次对话

**提交**: `feat(llm): add LLM client trait and OpenAI implementation`

---

### M1.2 REPL 交互循环

**你要做的事**:
- 实现 stdin 读取用户输入的循环
- 将用户输入发送给 LLM，打印回复
- 处理退出命令 (`/exit`, `Ctrl+C`)
- 集成到 `main.rs` 的 `Run` 命令

**涉及文件**: `src/main.rs`, `src/ui.rs`（可选，先简单 println 也行）

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `tokio::io::stdin` | 异步读取用户输入 | 或者用 `std::io::stdin` + `tokio::task::spawn_blocking` |
| 生命周期 `'_` | LLM client 在循环中被借用 | 理解为什么 `&self` 在 loop 中需要注意生命周期 |
| `Ctrl+C` 信号处理 | 优雅退出 | `tokio::signal::ctrl_c()` |

**要学的 Agent 知识**:
- **REPL 模式 vs 单次执行**: 对应 CLI 的 `run` 和 `exec` 命令
- **对话历史**: 为什么要维护 `Vec<Message>`，每次调用都要发送完整历史

**验收**: `cargo run` 能进入交互模式，你输入问题，LLM 回复，Ctrl+C 退出

**提交**: `feat(repl): add interactive REPL loop`

---

## 里程碑 2: 能用工具

> **目标**: 让 LLM 能调用工具（读文件、写文件、执行命令），这是 Agent 区别于 Chatbot 的核心。
>
> 完成后，LLM 不再只是"说"，而是能"做"。

### M2.1 工具框架

**你要做的事**:
- 完善 `Tool` trait：加入 `input_schema()` 方法（返回 JSON Schema）
- 实现 `ToolRegistry`：注册、查找、列举工具
- 实现 `ToolDispatcher`：根据工具名分发调用
- 实现路径安全检查（防止目录穿越）

**涉及文件**: `src/tools/mod.rs`, `src/tools/traits.rs`, `src/tools/registry.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `dyn Trait` + `Box` | Registry 需要存储不同类型的 Tool 实现 | `HashMap<String, Box<dyn Tool>>` |
| trait object vs generics | 运行时多态 vs 编译时多态 | 这里选 trait object 因为工具类型在运行时确定 |
| `serde_json::Value` | 工具参数是动态的 JSON | 理解为什么不用强类型——因为每个工具的参数不同 |
| `Path` 安全处理 | 防止 `../../etc/passwd` 攻击 | `canonicalize()` + 前缀检查 |

**要学的 Agent 知识**:
- **Tool Use 协议**: LLM 返回 `tool_calls`，Agent 执行后把结果作为 `tool` role 消息发回
- **JSON Schema**: 为什么工具需要 schema（LLM 需要知道参数格式才能正确调用）
- **工具权限模型**: 为什么需要路径安全检查（LLM 可能被注入恶意指令）

**验收**: `cargo test` 能注册工具、按名查找、dispatch 调用

**提交**: `feat(tools): add tool registry and dispatch framework`

---

### M2.2 文件工具

**你要做的事**:
- `ReadTool`: 读文件内容，支持行号范围 (offset + limit)
- `WriteTool`: 创建/覆盖文件
- `EditTool`: 字符串替换编辑（old_string → new_string）

**涉及文件**: `src/tools/read.rs`, `src/tools/write.rs`, `src/tools/edit.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `std::fs` 文件操作 | 读写文件是核心功能 | `read_to_string`, `write`, `create_dir_all` |
| 迭代器 `.skip().take()` | 实现行号范围读取 | `lines().enumerate().skip(offset).take(limit)` |
| `String` vs `&str` | 文件内容处理涉及大量字符串操作 | 理解所有权和借用在字符串操作中的体现 |
| 单元测试 + `tempfile` | 文件工具需要真实的文件系统测试 | `tempfile::TempDir` 自动清理 |

**要学的 Agent 知识**:
- **Edit 工具的设计哲学**: 为什么用"查找替换"而不是"按行号修改"（LLM 经常数错行号）
- **工具输出格式**: 为什么 read 要带行号输出（帮助 LLM 定位代码位置）

**验收**: 每个工具有 3+ 个单元测试，覆盖正常和异常场景

**提交**: `feat(tools): implement read/write/edit file tools`

---

### M2.3 命令工具

**你要做的事**:
- `BashTool`: 执行 shell 命令，捕获 stdout/stderr
- 实现命令超时（用 `tokio::time::timeout`）
- 实现命令白名单（可选）

**涉及文件**: `src/tools/bash.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `tokio::process::Command` | 异步执行子进程 | 对比 `std::process::Command` |
| `timeout` | 防止命令无限执行 | `tokio::time::timeout(Duration::from_secs(30), ...)` |
| `Output` 处理 | stdout 和 stderr 是 `Vec<u8>` | `String::from_utf8_lossy` 处理非 UTF-8 输出 |

**要学的 Agent 知识**:
- **Bash 工具是双刃剑**: 最强大也最危险的工具，需要安全边界
- **输出截断**: 为什么需要限制输出长度（避免撑爆上下文窗口）

**验收**: 能执行 `echo hello` 并返回 "hello"；超时命令会被终止

**提交**: `feat(tools): implement bash command tool`

---

### M2.4 搜索工具

**你要做的事**:
- `GlobTool`: 文件名模式匹配搜索
- `GrepTool`（可选）: 文件内容正则搜索

**涉及文件**: `src/tools/glob.rs`, `src/tools/grep.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `walkdir` crate | 递归遍历目录 | 比手写递归更健壮 |
| `regex` crate | 正则表达式匹配 | `Regex::new()` 可能失败，需要处理 |
| 迭代器组合 | 过滤 + 映射 + 收集 | `.filter().map().collect()` 链式调用 |

**要学的 Agent 知识**:
- **工具组合**: Agent 通常先 glob 找文件，再 read 看内容——理解工具如何协作
- **搜索结果的排序**: 按修改时间排序比字母序更有用（最近改的文件最相关）

**验收**: 能搜索 `**/*.rs` 返回项目中所有 Rust 文件

**提交**: `feat(tools): implement glob and grep search tools`

---

## 里程碑 3: 能思考

> **目标**: 实现 Agent Loop——接收 LLM 响应，如果包含 tool_calls 就执行工具，
> 把结果发回 LLM，循环直到 LLM 给出最终回答。
>
> 这是 Agent 的"大脑"。前面的 REPL 只是一问一答，现在要变成"思考-行动-观察"的循环。

### M3.1 Agent 状态与消息类型

**你要做的事**:
- 重构 `src/state.rs` → `src/agent/types.rs`
- 定义完整的消息类型: `Message { role, content_blocks }`
- 定义 `ContentBlock` enum: Text / ToolUse / ToolResult
- 定义 `AgentState` struct (不是简单 enum，而是包含完整运行时状态)

**涉及文件**: `src/agent/mod.rs`, `src/agent/types.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| 枚举的丰富表达力 | `ContentBlock` 是典型的 tagged union | Rust 的 enum 比其他语言的 union 类型安全得多 |
| `#[serde(tag = "type")]` | 序列化 tagged union 为 JSON | 对齐 API 的 JSON 格式 |
| struct vs enum 的设计选择 | `AgentState` 为什么用 struct 而非 enum | 当状态有"正交维度"（streaming + pending_tools 可独立变化），struct 更合适 |

**要学的 Agent 知识**:
- **多模态内容块**: 一个 assistant message 可以同时包含文字和多个 tool_calls
- **StopReason 的含义**: `end_turn` vs `tool_use` vs `max_tokens`——各代表什么，Agent 应该怎么反应

**验收**: 所有类型支持 `Serialize`/`Deserialize`，`cargo test` 验证 JSON 往返

**提交**: `feat(agent): define agent types and message model`

---

### M3.2 Agent Loop — 核心双循环

**你要做的事**:
- 实现 `Agent` struct：持有 LlmClient + ToolRegistry + 消息历史
- 实现核心循环逻辑:
  ```
  loop {                              // outer: follow-up
      while has_tool_calls {          // inner: tool execution
          execute_tools()
          send_results_to_llm()
          response = call_llm()
          if response.has_tool_calls() { continue }
          else { break }
      }
      if has_follow_up() { continue } // 有后续消息则继续
      break                           // 否则结束
  }
  ```
- 实现 steering 队列（中断当前执行）和 follow-up 队列（追加后续任务）

**涉及文件**: `src/agent/agent.rs`, `src/agent/loop.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| 所有权与可变借用 | Agent 持有 messages，循环中需要同时读和写 | 考虑 `&mut self` 的设计，或者用内部可变性 |
| `tokio::select!` | 同时等待 LLM 响应和用户中断 | `select!` 让你同时监听多个 future |
| `CancellationToken` | 优雅地中止正在执行的工具 | `tokio_util::sync::CancellationToken` |
| 模式匹配 + 控制流 | 根据 StopReason 决定下一步 | `match response.stop_reason { ... }` |

**要学的 Agent 知识**:
- **这是 Agent 的核心**: ReAct 模式 (Reason + Act) 的代码实现
- **Steering vs Follow-up**: steering 是"打断"（如用户按 Esc），follow-up 是"追加"（如自动代码审查）
- **为什么需要双循环**: 内循环处理工具调用链，外循环处理对话延续

**验收**: 用 MockClient 模拟"LLM 要求读文件 → 读取 → LLM 给出回答"的完整流程

**提交**: `feat(agent): implement dual-loop agent core`

---

### M3.3 事件系统

**你要做的事**:
- 定义 `AgentEvent` enum（四层生命周期: Agent/Turn/Message/Tool）
- 实现 `EventEmitter`（用 `tokio::sync::broadcast` 或 callback 列表）
- 在 Agent Loop 的关键节点发射事件
- UI 层订阅事件并渲染输出

**涉及文件**: `src/agent/events.rs`, `src/ui.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `broadcast` channel | 一对多的事件分发 | `tokio::sync::broadcast::channel` |
| `enum` 的类型安全事件 | 16+ 种事件，每种携带不同数据 | 比 string-based 事件系统更安全 |
| 闭包 `Fn` trait | 事件回调是闭包 | `Box<dyn Fn(AgentEvent) + Send>` |

**要学的 Agent 知识**:
- **事件驱动 UI**: 为什么不在 Agent Loop 里直接 print，而是通过事件解耦
- **可观测性**: 事件系统让你能追踪 Agent 的每一步操作（调试 Agent 的核心手段）

**验收**: 运行 Agent 时能看到事件流输出；UI 能实时展示 LLM 流式回复

**提交**: `feat(agent): add event system for observable agent behavior`

---

## 里程碑 4: 能记忆

> **目标**: 管理对话上下文，在 token 限制内保留最有价值的信息。
>
> 没有记忆管理的 Agent 很快就会撞上上下文窗口限制。

### M4.1 上下文管理

**你要做的事**:
- 实现 token 计数（先用简单估算：1 token ≈ 4 字符，后续可接 tiktoken）
- 实现 `ContextManager`：管理消息历史，跟踪 token 用量
- 实现第一层压缩：旧的 tool result 替换为摘要占位符

**涉及文件**: `src/memory/mod.rs`, `src/memory/tokenizer.rs`, `src/memory/context.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| 策略模式 | 不同的压缩策略可替换 | trait `CompressionStrategy` |
| 迭代器 `.rev()` | 从最旧的消息开始压缩 | 保留最近的，压缩最早的 |

**要学的 Agent 知识**:
- **上下文窗口是 Agent 的"工作记忆"**: 就像人脑一样，容量有限
- **工具结果是最大的 token 消耗者**: 一次 `cat` 大文件就可能占掉一半上下文

**验收**: 当 token 接近阈值时自动压缩旧的工具结果

**提交**: `feat(memory): add context manager with tool result compression`

---

### M4.2 对话压缩

**你要做的事**:
- 实现阈值触发压缩：token 用量 > 80% 时自动压缩
- 实现 LLM 摘要生成：让 LLM 总结对话历史
- 实现 `/compact` 命令：用户主动触发压缩

**涉及文件**: `src/memory/compression.rs`, `src/memory/summarizer.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| 递归/自引用调用 | 压缩器需要调用 LLM，LLM 又受压缩器管理 | 考虑依赖注入，避免循环依赖 |

**要学的 Agent 知识**:
- **摘要即有损压缩**: 压缩后信息不可逆丢失，摘要质量至关重要
- **压缩时机的权衡**: 太早压缩浪费上下文空间，太晚压缩导致 API 报错

**验收**: 长对话中自动触发压缩，Agent 仍能记住之前讨论的要点

**提交**: `feat(memory): add conversation compression with LLM summarization`

---

### M4.3 会话持久化

**你要做的事**:
- 实现会话保存到磁盘（JSON 格式）
- 实现 `continue` 命令：恢复最近会话
- 实现 `resume` 命令：按 session ID 恢复

**涉及文件**: `src/memory/storage.rs`, `src/memory/session.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `directories` crate | 跨平台数据目录 | `ProjectDirs::data_dir()` |
| `uuid` | 会话 ID 生成 | `Uuid::new_v4()` |
| 文件锁 | 防止并发写入同一会话 | `fs2::FileExt` 或简单的 lockfile |

**要学的 Agent 知识**:
- **会话 = 对话状态的快照**: 保存什么（消息历史）、不保存什么（工具实例、LLM 连接）
- **Continue vs Resume**: continue 是"上次聊到哪继续"，resume 是"回到某次特定对话"

**验收**: 退出后重新运行，能继续上次对话

**提交**: `feat(memory): add session persistence and resume`

---

## 里程碑 5: 能协调

> **目标**: 主 Agent 能将任务委派给专门的子 Agent，实现分工协作。
>
> 这是从"一个 Agent"到"Agent 系统"的关键跳跃。

### M5.1 子 Agent 与委派

**你要做的事**:
- 定义 `SubAgent` trait（继承自 Agent 的简化版本）
- 实现几个专用 Agent: Coder, Reviewer, Explorer
- 实现主 Agent 的委派逻辑：识别任务类型 → 选择合适的子 Agent → 发送上下文 → 收集结果

**涉及文件**: `src/agents/mod.rs`, `src/agents/types.rs`, `src/agents/coder.rs`, ...

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| trait 继承 | SubAgent 复用 Agent 的部分能力 | `trait SubAgent: Send + Sync { ... }` |
| 工厂模式 | 根据类型创建不同的 Agent | `match agent_type { ... }` 或 `HashMap<AgentType, Box<dyn SubAgent>>` |
| 泛型约束 | Agent 注册表需要 `Send + Sync + 'static` | 理解线程安全约束 |

**要学的 Agent 知识**:
- **子 Agent 不是"新 AI"**: 它们共享同一个 LLM，只是有不同的 system prompt 和可用工具
- **委派 = system prompt 工程**: Coder 和 Reviewer 的区别主要在 prompt 和工具权限

**验收**: 主 Agent 能识别"写代码"请求并委派给 Coder Agent

**提交**: `feat(agents): add sub-agent system with delegation`

---

### M5.2 任务 DAG

**你要做的事**:
- 定义 `Task` 结构体（id, deps, status, assignee）
- 实现 `TaskGraph`：有向无环图，支持拓扑排序
- 实现 `TaskScheduler`：根据依赖关系调度任务

**涉及文件**: `src/tasks/mod.rs`, `src/tasks/graph.rs`, `src/tasks/scheduler.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| 图数据结构 | DAG 是任务依赖的自然表示 | `HashMap<TaskId, Vec<TaskId>>` 邻接表 |
| 拓扑排序 (Kahn's) | 确定任务执行顺序 | 入度为 0 的节点先执行 |
| 循环检测 | 防止循环依赖 | DFS + 颜色标记 (白/灰/黑) |

**要学的 Agent 知识**:
- **任务分解是 Agent 规划能力的核心**: 大任务 → 子任务 → 依赖关系 → 执行顺序
- **Claude Code 的 TodoWrite 就是这个**: 只不过它的 DAG 比较简单（主要是线性依赖）

**验收**: 能构建任务图，检测循环依赖，输出拓扑排序

**提交**: `feat(tasks): add task DAG with topological scheduling`

---

### M5.3 技能系统

**你要做的事**:
- 定义 `Skill` trait（name, trigger, execute）
- 实现斜杠命令触发 (`/commit`, `/review`)
- 实现启发式自动触发（根据上下文关键词匹配）

**涉及文件**: `src/skills/mod.rs`, `src/skills/registry.rs`, `src/skills/trigger.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| 模式匹配字符串 | 斜杠命令解析 | `if input.starts_with('/') { ... }` |
| 正则匹配 | 启发式触发 | `Regex::is_match()` |

**要学的 Agent 知识**:
- **技能 = 预设的 Agent 工作流**: 不需要 LLM 规划，直接执行确定性流程
- **触发的二元性**: 显式触发（用户命令）vs 隐式触发（Agent 自己识别）

**验收**: `/commit` 能触发提交流程；发现 bug 时自动建议调试技能

**提交**: `feat(skills): add skill system with slash commands and heuristic triggers`

---

## 里程碑 6: 能并发

> **目标**: 让耗时操作（如编译、测试）在后台异步执行，Agent 不被阻塞。
>
> 注意：我们用 tokio 异步模型，不需要手动管理线程。

### M6.1 异步任务队列

**你要做的事**:
- 实现 `BackgroundTask` 结构体
- 实现基于 `tokio::sync::mpsc` 的任务队列
- 实现并发限制（Semaphore）

**涉及文件**: `src/executor/mod.rs`, `src/executor/queue.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `mpsc` channel | 生产者-消费者模式 | `tokio::sync::mpsc::channel` |
| `Semaphore` | 限制并发数 | `tokio::sync::Semaphore` |
| `JoinHandle` | 跟踪异步任务 | `tokio::spawn` 返回 `JoinHandle` |

**要学的 Agent 知识**:
- **Agent 的效率瓶颈**: 不是 LLM 推理，而是工具执行（编译可能要几分钟）
- **并发 != 并行**: Agent 同时发起多个请求，但不一定在多核上同时运行

**验收**: 能同时运行多个后台任务，且不超过配置的并发上限

**提交**: `feat(executor): add async task queue with concurrency control`

---

### M6.2 后台执行器

**你要做的事**:
- 将 `BashTool` 的长时间命令改为后台执行
- 实现进度通知（task started / completed / failed）
- 实现超时和取消

**涉及文件**: `src/executor/runner.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `tokio::select!` | 同时等待任务完成和超时 | 最实用的 tokio 宏 |
| `CancellationToken` | 取消正在运行的任务 | 从 Agent 中止传递到子进程 |

**验收**: `cargo build` 在后台执行时，Agent 仍然能响应输入

**提交**: `feat(executor): add background task runner with timeout`

---

### M6.3 结果收集

**你要做的事**:
- 实现结果通知队列
- Agent 在每次 LLM 调用前检查已完成的后台任务
- 将完成结果注入对话上下文

**涉及文件**: `src/executor/collector.rs`, `src/agent/agent.rs`

**要学的 Agent 知识**:
- **拉取 vs 推送**: Agent 主动拉取结果（每次 LLM 调用前），而非结果推送给 Agent
- **结果注入时机**: 太早注入浪费上下文，太晚注入 Agent 可能重复执行

**验收**: 后台编译完成后，Agent 的下一次回复中能看到编译结果

**提交**: `feat(executor): add result collection and context injection`

---

## 里程碑 7: 能协作

> **目标**: 多个 Agent 组成团队，通过消息队列协作，每个 Agent 在独立的 git worktree 中工作。
>
> 这是最终形态——一个微型的 Agent 协作系统。

### M7.1 Agent 团队通信

**你要做的事**:
- 实现 Agent 邮箱（每个 Agent 有自己的消息队列）
- 实现消息路由（TeamManager 分发消息）
- 实现广播机制

**涉及文件**: `src/team/mod.rs`, `src/team/mailbox.rs`, `src/team/manager.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `mpsc` per-agent | 每个 Agent 一个 receiver | `HashMap<AgentId, mpsc::Sender>` |
| `Arc<RwLock<>>` | 团队状态需要共享读写 | 读多写少时 `RwLock` 比 `Mutex` 好 |

**要学的 Agent 知识**:
- **Actor 模型**: 每个 Agent 是一个 Actor，通过消息通信，不共享状态
- **这就是 Agent 协作的本质**: 不是共享内存，而是结构化消息传递

**验收**: Coder 发送代码给 Reviewer，Reviewer 返回审查意见

**提交**: `feat(team): add agent mailbox and message routing`

---

### M7.2 团队生命周期

**你要做的事**:
- 实现团队启动/停止
- 实现 Agent 健康检查
- 实现优雅关闭（等待进行中的任务完成）

**涉及文件**: `src/team/lifecycle.rs`, `src/team/factory.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| `tokio::spawn` + `JoinSet` | 管理多个并发 Agent | `JoinSet` 比手动管理 `JoinHandle` 更方便 |
| graceful shutdown | 优雅关闭是分布式系统的基本功 | 先停止接收新任务 → 等待当前任务 → 关闭 |

**验收**: 团队能启动 3 个 Agent，协作完成任务后优雅关闭

**提交**: `feat(team): add team lifecycle management`

---

### M7.3 Worktree 隔离

**你要做的事**:
- 封装 `git worktree` 命令（add/remove/list）
- 实现任务-worktree 绑定（每个任务一个隔离工作目录）
- 实现自动清理（任务完成后删除 worktree）

**涉及文件**: `src/worktree/mod.rs`, `src/worktree/git.rs`, `src/worktree/manager.rs`

**要学的 Rust 知识**:

| 概念 | 为什么在这里学 | 提示 |
|------|--------------|------|
| RAII / `Drop` trait | worktree 创建后需要确保被清理 | 实现 `Drop` 自动清理，或用 guard pattern |
| `Command` 链式调用 | 封装 git CLI | `Command::new("git").args(&["worktree", "add", ...])` |

**要学的 Agent 知识**:
- **隔离是并行协作的前提**: 没有隔离，多个 Agent 同时改文件会冲突
- **Worktree 是 Agent 的"独立工位"**: 用完后合并回主分支

**验收**: 两个 Agent 在不同 worktree 同时编辑代码，最后合并无冲突

**提交**: `feat(worktree): add git worktree isolation for agent tasks`

---

## 附录: 学习资源推荐

### Agent 工程
- [ReAct 论文](https://arxiv.org/abs/2210.03629) — Agent Loop 的理论基础
- [OpenAI Function Calling 文档](https://platform.openai.com/docs/guides/function-calling) — Tool Use 协议
- [Anthropic Tool Use 文档](https://docs.anthropic.com/en/docs/build-with-claude/tool-use/overview) — 另一种视角

### Rust
- [The Rust Book](https://doc.rust-lang.org/book/) — 遇到所有权/生命周期问题时翻阅
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) — 异步编程的最佳入门
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — 快速查阅语法
