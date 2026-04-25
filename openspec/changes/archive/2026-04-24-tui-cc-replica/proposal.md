## Why

kloud 的 TUI 已有基础框架（ratatui + crossterm），但视觉效果和交互体验与 Claude Code 差距明显。Claude Code 使用 Ink (React TUI) 实现了精致的终端 UI——虚拟滚动、主题系统、流式渲染动画、工具调用折叠、多行输入等。作为学习项目，精确复刻 Claude Code 的 TUI 既能提升用户体验，又能深入理解终端 UI 工程的每个细节。

## What Changes

- 重新设计消息渲染系统，支持虚拟滚动和消息分组
- 实现多行输入框，支持历史导航、Vim 模式、自动补全
- 添加主题系统，支持 dark/light/dark-ansi/light-ansi/dark-daltonized/light-daltonized 六种配色
- 实现工具调用可视化：进度指示器、折叠展开、批量分组
- 添加流式渲染动画系统：shimmer 效果、stall 检测、token 计数动画
- 实现 thinking blocks 的呼吸效果和 ultrathink 彩虹高亮
- 添加状态栏：模型名、权限模式、token 用量、费用、worktree 信息
- 实现搜索功能（/键触发，n/N 导航，高亮匹配）
- 添加 transcript 模式查看完整历史
- 实现命令面板和斜杠命令菜单

## Capabilities

### New Capabilities

- `tui-theme`: 主题系统和配色方案，支持 6 种预设主题和语义化颜色
- `tui-virtual-scroll`: 虚拟滚动消息列表，高效渲染大量消息
- `tui-input`: 多行输入系统，支持历史、Vim 模式、自动补全、斜杠命令
- `tui-streaming-animations`: 流式渲染动画引擎——shimmer、stall 检测、token 计数动画、thinking 呼吸效果
- `tui-tool-rendering`: 工具调用可视化——进度指示、折叠展开、批量分组、语法高亮
- `tui-status-bar`: 状态栏组件——模型、权限、token、费用、worktree
- `tui-search`: 消息搜索和高亮导航

### Modified Capabilities

- `streaming-pipeline`: 流式事件需要携带更多 UI 渲染元数据（token 增量、thinking delta、stall 信号）
- `session-lifecycle`: session 视图需扩展以支持新 TUI 状态（搜索模式、transcript 模式）

## Impact

- **核心文件变更**: `src/ui/` 整体重构，新增 `theme/`、`components/`、`animations/` 子模块
- **事件协议扩展**: `AppEvent`/`UiAction` 需新增搜索、主题切换、transcript 等事件
- **依赖新增**: 可能需要 `syntect`（语法高亮）、`unicode-width`（精确字符宽度计算）
- **性能要求**: 虚拟滚动要求消息渲染在 16ms 内完成（60fps 目标）
- **渲染频率**: 动画系统需要从当前 20fps 提升到至少 30fps
