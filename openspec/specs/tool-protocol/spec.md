# Tool Protocol（工具协议）

## 它是什么

定义 kloud 中"工具"的契约——从 Tool trait 定义、注册发现、调用协议、到路径安全。这是 LLM 从"只会说"到"能做事"的关键桥梁。

## 关键类型

| 类型 | 位置 | 一句话 |
|------|------|--------|
| `Tool` trait | `tools/traits.rs` | 所有工具必须实现的接口：name, description, input_schema, execute |
| `ToolCall` | `tools/call.rs` | 一次工具调用的请求：name + args(JSON) |
| `ToolResult` | `tools/call.rs` | 工具调用的结果：name + output(Ok/Err) |
| `ToolRegistry` | `tools/registry.rs` | BTreeMap<String, Arc<dyn Tool>>，提供注册、查找、definitions、dispatch |
| `ContentBlock::ToolUse` | `llm/types.rs` | LLM 响应中的工具调用块 |
| `ContentBlock::ToolResult` | `llm/types.rs` | 回传给 LLM 的工具结果块 |

## 数据流

```
LLM 响应
  │
  ▼
ContentBlock::ToolUse { id, name, input }
  │
  │  Session 提取为 PendingDispatch
  ▼
ToolCall { name, args }  ───  extract_tool_calls() 转换
  │
  │  ToolRegistry::dispatch()
  ▼
Arc<dyn Tool>::execute(&ToolCall)
  │
  ▼
ToolResult { name, output: Ok(String) | Err(String) }
  │
  │  转换回 LLM 格式
  ▼
ContentBlock::ToolResult { tool_use_id, content, is_error }
  │
  │  追加到 messages，再发 LLM
  ▼
InputMessage { role: User, content: [ToolResult] }
```

## Tool trait 契约

```rust
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;                    // 唯一标识
    fn description(&self) -> &str;             // 给 LLM 看的说明
    fn input_schema(&self) -> serde_json::Value; // JSON Schema
    async fn execute(&self, call: &ToolCall) -> Result<ToolResult>;
}
```

**关键设计决策**：
- `async_trait` 宏：因为 Rust 原生 async trait 还不够成熟
- `Send + Sync`：必须满足跨线程调度要求
- `execute` 接收 `&ToolCall`（借用），返回 `Result<ToolResult>`——工具永远不 panic，错误包装在 `ToolResult.output` 的 `Err` 变体里
- `args` 用 `serde_json::Value` 而非泛型——牺牲编译时类型安全，换取 LLM 输出的灵活性

## 已实现的工具

| 工具 | 文件 | 功能 |
|------|------|------|
| `echo` | `tools/builtin/echo.rs` | 测试用，原样返回 message |
| `read` | `tools/builtin/read.rs` | 读文件，支持 offset/limit，行号格式 |
| `write` | `tools/builtin/write.rs` | 写文件（仅新文件，拒绝覆盖） |

## 路径安全模型

```
resolve_existing_path(root, user_path)    ── 读文件用
  ├─ 拒绝绝对路径
  ├─ canonicalize 后检查 starts_with(root)
  └─ 防止 "../" 逃逸

resolve_writable_path(root, user_path)    ── 写文件用
  ├─ 拒绝绝对路径
  ├─ 逐级向上找可 canonicalize 的父目录
  └─ 检查解析后的父目录 starts_with(root)
```

**设计理念**：所有工具共享同一个 `root`（cwd），以 chroot 模式限制文件访问范围。

## 与 LLM 的协议映射

```
Anthropic API                    kloud 内部
─────────────                    ──────────
tool_use block {id, name, input} → PendingDispatch → ToolCall
tool_result block {tool_use_id,  → ToolResult → ContentBlock::ToolResult
  content, is_error}

ChatRequest.tools[]              ← ToolRegistry.definitions()
  = [{name, description, input_schema}]
```

## 未决问题

1. **EditTool 缺失**：ROADMAP M2.2 计划，当前只有 read 和 write，缺少 in-place 编辑能力
2. **BashTool 缺失**：M2.3 计划，需要沙箱/超时/权限控制
3. **工具输入验证**：当前工具自己在 execute 里手动解析 args，没有基于 JSON Schema 的自动验证
4. **MCP 工具扩展**：`server_name` 字段已预留，但 MCP 工具注册和发现未实现
5. **工具输出截断**：read 有 MAX_LINES(2000) + MAX_LINE_LENGTH(1024)，但 tool_result 传给 LLM 时的总大小没有限制

## 参考

- 源文件：`src/tools/` 全目录
- 路径工具：`src/tools/path.rs`
- 工具在 session 中的调用：`src/app/session/tools.rs`
