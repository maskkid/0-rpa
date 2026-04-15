# RPA Automation VM

插件驱动的自动化执行平台 — 基于 Rust + QuickJS，支持 GUI 自动化（UIA/OCR/Image）、工作流编排（JSON+JS）、插件扩展、API 服务化（HTTP+gRPC）。

---

## 项目概述

这是一个**高度灵活的 RPA 自动化虚拟机**，核心设计理念是"灵活性 ≈ 手写代码"。通过 VM 抽象 + JS 运行时 + 插件系统的三层架构，实现：

- **GUI 自动化**：通过 UIA/OCR/Image 多策略查找 UI 元素，模拟鼠标键盘操作
- **工作流编排**：JSON 定义步骤 + JS 脚本灵活控制，两者可混合使用
- **插件化扩展**：每个软件适配为一个插件 ZIP，包含工作流、脚本和资源
- **API 服务化**：HTTP REST + gRPC 双轨对外，方便外部系统集成
- **跨平台**：架构上 trait 抽象，Windows 优先，后续扩展 macOS/Linux

## 技术栈

| 组件 | 技术 | 说明 |
|------|------|------|
| 核心语言 | Rust 2021 Edition | 性能、安全、并发 |
| 异步运行时 | tokio | 全特性异步 |
| JS 运行时 | QuickJS (rquickjs) | 轻量 JS 引擎 |
| HTTP 服务 | axum | REST API |
| gRPC 服务 | tonic + prost | RPC API |
| Windows UIA | windows crate | Microsoft 官方 Rust 绑定 |
| 序列化 | serde + serde_json | 标准 |
| 错误处理 | thiserror + anyhow | 库级 + 应用级 |
| CLI | clap | 命令行参数 |
| 构建脚本 | just | 类 Make 的任务运行器 |

## 项目结构

```
rpa/
├── Cargo.toml                  # Workspace 根配置
├── justfile                    # 构建/开发脚本
├── proto/
│   └── rpa.proto               # gRPC 服务定义
├── _docs/
│   ├── design.md               # 架构设计文档
│   └── uml.md                  # UML 图
├── crates/
│   ├── rpa-core/               # 核心数据结构、trait、错误类型
│   ├── rpa-engine/             # VM 执行引擎（指令分发、多策略查找、重试、取消）
│   ├── rpa-workflow/           # JSON 工作流解析、编译、校验
│   ├── rpa-js/                 # QuickJS 运行时、JS ↔ Rust 桥接
│   ├── rpa-plugin/             # 插件加载（ZIP）、生命周期、沙箱
│   ├── rpa-perception/         # UIA/OCR/Image 元素查找
│   ├── rpa-action/             # 鼠标键盘操作（Windows SendInput）
│   ├── rpa-api/                # HTTP REST (axum) + gRPC (tonic)
│   └── rpa-orchestrator/       # 任务调度、并发控制、状态追踪
└── apps/
    └── server/                 # 主服务器二进制
```

## 架构总览

```
API Layer (HTTP / gRPC)
        ↓
Orchestrator（调度）
        ↓
Automation VM（执行核心）
        ↓
┌───────────────┬───────────────┬───────────────┐
│ Workflow Engine │ JS Runtime    │ Plugin System │
└───────────────┴───────────────┴───────────────┘
        ↓
Perception（UIA/OCR/Image） + Action（鼠标键盘）
```

**核心执行流程**：

1. 外部调用 → API → Orchestrator → VM
2. VM 加载 Instruction 序列（来自 JSON 工作流或 JS 脚本）
3. 每条指令：Resolve Target → Find Element → Execute Action → Update Context
4. 元素查找按降级链：UIA → OCR → Image → Position

## 快速开始

### 前置条件

- Rust toolchain: `rustup default stable`
- Windows cross-compile target: `rustup target add x86_64-pc-windows-gnu`
- just (任务运行器): `cargo install just`
- protoc (gRPC 编译): `brew install protobuf` 或 Windows 上安装

### 常用命令

```bash
# 开发模式 — 编译检查 + 测试
just dev

# 运行所有测试
just test

# 编译整个 workspace
just build

# 编译 Windows 版本（交叉编译）
just build-win

# 运行服务器（开发模式）
just run

# 代码格式化 + lint
just lint

# 清理构建产物
just clean
```

详见 [justfile](justfile) 获取所有可用命令。

### 手动 Cargo 命令

```bash
# 检查编译
cargo check

# 运行核心测试
cargo test -p rpa-core -p rpa-engine

# 运行全部测试
cargo test

# 编译 release
cargo build --release

# 交叉编译 Windows
cargo build --release --target x86_64-pc-windows-gnu
```

## 开发阶段与依赖关系

```
Phase 0: Workspace 脚手架 ✅
    └→ Phase 1: core + engine ✅
          ├→ Phase 2: perception + action（可并行）
          ├→ Phase 3: workflow（可并行）
          └→ Phase 4: js（可并行）
                └→ Phase 5: plugin
                      └→ Phase 6: api + orchestrator
                            └→ Phase 7: 集成与打磨
```

**当前进度**：Phase 0 + Phase 1 已完成，Phase 2-7 为骨架代码。

## 核心 Crate 详解

### rpa-core — 共享数据结构

所有其他 crate 的基础依赖，定义了核心类型和 trait：

| 文件 | 内容 |
|------|------|
| `instruction.rs` | `Instruction` 枚举：Click, Input, Extract, Call, Loop, If, Break, Return, Log, Scroll 等 |
| `target.rs` | `Target` 枚举：UIA(selector), Image(path), Text(pattern), Position(x,y) |
| `element.rs` | `Element` 结构体：id, bounds, text, element_type, platform_handle |
| `value.rs` | `Value` 动态类型：Null, Bool, Number, String, Array, Object |
| `context.rs` | `Context`：变量表、调用栈、重试配置；`RetryConfig` + 退避策略 |
| `condition.rs` | `Condition` 枚举：VarEquals, VarNotEmpty, ElementExists, And/Or/Not |
| `error.rs` | `RpaError` — 统一错误类型 |
| `traits.rs` | `Perceptor`, `Actor`, `WorkflowProvider`, `JsRuntime` — 核心 trait |
| `task.rs` | `TaskStatus`, `TaskResult`, `TaskPriority` |
| `plugin.rs` | `PluginManifest`, `Permission` |
| `spec.rs` | `DataSpec`, `FieldSpec`, `ExtractAttribute` |

### rpa-engine — VM 执行引擎

| 文件 | 内容 |
|------|------|
| `vm.rs` | `Vm` struct — builder 模式注入 Perceptor/Actor/JsRuntime |
| `executor.rs` | `Executor` — 指令分发，控制流（Continue/Break/Return） |
| `finder.rs` | `MultiStrategyFinder` — 多策略降级查找（UIA→OCR→Image→Position） |
| `retry.rs` | 重试逻辑 + 退避策略（Fixed/Linear/Exponential） |
| `cancellation.rs` | `CancellationToken` — 基于 tokio watch channel 的协作式取消 |
| `events.rs` | `ExecutionEvent` — 执行过程事件流 |

### 核心 trait 接口

```rust
// 元素查找
#[async_trait]
pub trait Perceptor: Send + Sync {
    async fn find(&self, target: &Target, ctx: &Context) -> Result<Element>;
    async fn find_all(&self, target: &Target, ctx: &Context) -> Result<Vec<Element>>;
}

// UI 操作
#[async_trait]
pub trait Actor: Send + Sync {
    async fn click(&self, element: &Element, button: MouseButton) -> Result<()>;
    async fn input_text(&self, element: &Element, text: &str, clear_first: bool) -> Result<()>;
    async fn key_press(&self, key: &str, modifiers: Vec<ModifierKey>) -> Result<()>;
    // ...
}

// 工作流提供
#[async_trait]
pub trait WorkflowProvider: Send + Sync {
    fn name(&self) -> &str;
    fn get_workflow(&self, name: &str) -> Result<Vec<Instruction>>;
    fn list_workflows(&self) -> Vec<String>;
}
```

## 工作流 JSON 格式

```json
{
  "steps": [
    { "type": "click", "target": "发送按钮" },
    { "type": "wait", "duration_ms": 500 },
    { "type": "extract", "target": { "type": "uia", "name": "消息内容" }, "fields": [{"name": "text", "attribute": "Text"}], "into": "message" },
    { "type": "loop", "max": 10, "steps": [
      { "type": "click", "target": "下一页" }
    ]},
    { "type": "if", "condition": { "type": "VarNotEmpty", "var": "message" }, "then": [
      { "type": "log", "message": "Found message", "level": "Info" }
    ]}
  ]
}
```

`target` 支持简写：`"发送按钮"` 自动按 UIA→OCR→Image 降级查找。

## JS 脚本 API

```javascript
// 元素查找
const el = await rpa.find("发送按钮");
// 精确指定
const el2 = await rpa.find({ type: "uia", name: "发送", className: "Button" });

// 操作
await rpa.click(el);
await rpa.input(el, "Hello");
await rpa.waitFor("对话框", { timeout: 5000 });

// 变量
const text = await rpa.extract(el, "text");
rpa.setVar("result", "done");

// 流程控制就是 JS
for (let i = 0; i < 10; i++) {
    await rpa.click(await rpa.find("下一页"));
    await rpa.wait(1000);
}
```

## 插件结构

```
plugin.zip
├── manifest.json        # 名称、版本、权限声明
├── workflows/
│   └── open_chat.json  # 工作流定义
├── scripts/
│   ├── init.js          # 初始化脚本
│   └── helpers.js       # 辅助脚本
└── assets/
    └── button.png       # 图像资源
```

## API 端点

### HTTP REST

```
POST   /api/v1/run              # 异步运行工作流
POST   /api/v1/run/sync         # 同步运行
GET    /api/v1/tasks/:id         # 查询任务状态
POST   /api/v1/tasks/:id/cancel  # 取消任务
GET    /api/v1/tasks/:id/events  # SSE 事件流
GET    /api/v1/plugins            # 列出插件
POST   /api/v1/plugins/load      # 加载插件
DELETE /api/v1/plugins/:name     # 卸载插件
GET    /api/v1/health            # 健康检查
```

### gRPC

```protobuf
service RpaService {
    rpc RunWorkflow(RunWorkflowRequest) returns (RunWorkflowResponse);
    rpc GetTaskStatus(GetTaskStatusRequest) returns (GetTaskStatusResponse);
    rpc CancelTask(CancelTaskRequest) returns (CancelTaskResponse);
    rpc ListPlugins(ListPluginsRequest) returns (ListPluginsResponse);
    rpc StreamTaskEvents(StreamTaskEventsRequest) returns (stream TaskEvent);
}
```

## 接手指南 — AI / 人工快速上手

### 你是谁

你可能是一个新接手这个项目的 AI 或开发人员。以下信息帮你快速理解并开始工作。

### 代码阅读顺序（优先级从高到低）

1. **`_docs/design.md`** — 完整设计文档，理解整体架构必读
2. **`crates/rpa-core/src/traits.rs`** — 核心 trait 定义，理解系统边界
3. **`crates/rpa-core/src/instruction.rs`** — 指令枚举，理解 VM 能做什么
4. **`crates/rpa-engine/src/vm.rs`** — VM 入口，理解执行引擎如何组装
5. **`crates/rpa-engine/src/executor.rs`** — 指令分发，理解每条指令如何执行
6. **`crates/rpa-engine/src/finder.rs`** — 多策略查找，理解降级链逻辑

### 添加新指令的步骤

1. 在 `rpa-core/src/instruction.rs` 添加新的 `Instruction` 变体
2. 在 `rpa-core/src/instruction.rs` 更新 `Serialize/Deserialize` 派生
3. 在 `rpa-engine/src/executor.rs` 的 `execute()` match 分支中添加处理逻辑
4. 如果涉及新 trait 方法，在 `rpa-core/src/traits.rs` 添加
5. 在 `rpa-workflow/src/step.rs` 添加对应 Step 变体
6. 在 `rpa-workflow/src/compiler.rs` 添加 Step → Instruction 编译逻辑
7. 更新测试

### 添加新的 Perceptor（比如 macOS Accessibility）

1. 在 `rpa-perception/src/` 新建模块文件
2. 实现 `rpa_core::traits::Perceptor` trait
3. 在 `rpa-perception/src/lib.rs` 用 `#[cfg(target_os)]` 条件编译
4. 在 `rpa-engine/src/vm.rs` 的 `with_perceptor()` 注册

### 添加新的 Actor（比如 Linux xdotool）

1. 在 `rpa-action/src/` 新建模块文件
2. 实现 `rpa_core::traits::Actor` trait
3. 在 `rpa-action/src/lib.rs` 用 `#[cfg(target_os)]` 条件编译
4. 在 `rpa-engine/src/vm.rs` 的 `with_actor()` 注册

### 关键设计决策

| 决策 | 选择 | 原因 |
|------|------|------|
| VM 输入是 trait object | `Box<dyn Perceptor>`, `Arc<dyn Actor>` | 运行时注入平台实现 |
| JS 和 Rust 通信 | mpsc channel + oneshot | QuickJS 单线程，channel 异步桥接 |
| 执行流控制流 | ControlFlow enum | 避免 Option 嵌套，Break/Return 语义清晰 |
| 多策略查找 | 降级链 | UIA→OCR→Image→Position，一个失败自动尝试下一个 |
| 插件隔离 | 独立 JsRuntime | 每个 ZIP 解压到内存，JS 上下文隔离 |
| 工作流两阶段 | JSON→Step→Instruction | 校验在中间步骤，运行时只有 Instruction |

### 常见坑点

- **Windows UIA 必须在 Windows 上测试**：UIA 的 `windows` crate 只在 Windows 上编译。macOS 上开发时用 `MockPerceptor`。
- **QuickJS 是单线程的**：不能多线程同时调用 JS，必须通过 `spawn_blocking` 或 channel 通信。
- **`Cargo.lock` 已加入 .gitignore**：因为 workspace 主要输出是二进制，lock 文件的争议不大；如果需要可 `git add -f Cargo.lock`。
- **`Instruction::Break` 的处理**：在 `execute()` 中返回 `ControlFlow::Break`，由 `execute_block` 的循环捕获。
- **gRPC proto 变更后需手动 `cargo build`**：tonic 会在编译时生成 Rust 代码。

### 测试策略

```bash
# 核心类型测试（纯数据结构，macOS 可运行）
just test-core

# VM 引擎测试（使用 Mock，macOS 可运行）
just test-engine

# 全部单元测试（macOS 可运行，Mock 平台层）
just test

# Windows 集成测试（必须在 Windows 上运行）
# 测试 UIA 查找、SendInput 操作等
cargo test -p rpa-perception --target x86_64-pc-windows-gnu
cargo test -p rpa-action --target x86_64-pc-windows-gnu
```

### Git 提交约定

```
feat: 新功能
fix: 修复 bug
docs: 文档
refactor: 重构
test: 测试
chore: 构建/工具
```

## 配置

服务器配置文件 `config.toml`：

```toml
[server]
http_addr = "0.0.0.0:8080"
grpc_addr = "0.0.0.0:50051"

[engine]
default_timeout_ms = 30000
find_strategy = "sequential"
retry_max = 3
retry_delay_ms = 1000

[orchestrator]
max_concurrent_tasks = 4

[js]
max_memory_mb = 64
max_execution_time_ms = 60000

[plugins]
load_paths = ["./plugins"]
auto_load = true
```

## 依赖版本

核心依赖在 `Cargo.toml` 的 `[workspace.dependencies]` 中统一管理。各 crate 通过 `{ workspace = true }` 引用，避免版本冲突。

## License

MIT