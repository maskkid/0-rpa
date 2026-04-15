# RPA Automation VM

插件驱动的自动化执行平台 — 基于 Rust + QuickJS，支持 GUI 自动化（UIA/OCR/Image）、工作流编排（JSON+JS）、插件扩展、API 服务化（HTTP+gRPC）、Tauri 桌面客户端。

---

## 项目概述

这是一个**高度灵活的 RPA 自动化虚拟机**，核心设计理念是"灵活性 ≈ 手写代码"。通过 VM 抽象 + JS 运行时 + 插件系统的三层架构，实现：

- **GUI 自动化**：通过 UIA/OCR/Image 多策略查找 UI 元素，模拟鼠标键盘操作
- **工作流编排**：JSON 定义步骤 + JS 脚本灵活控制，两者可混合使用
- **插件化扩展**：每个软件适配为一个插件 ZIP，包含工作流、脚本和资源
- **API 服务化**：HTTP REST + gRPC 双轨对外，方便外部系统集成
- **桌面客户端**：Tauri v2 桌面应用，插件管理、工作流编辑、任务监控、调试可视化
- **跨平台**：架构上 trait 抽象，Windows 优先，后续扩展 macOS/Linux

## 技术栈

| 组件 | 技术 | 说明 |
|------|------|------|
| 核心语言 | Rust 2021 Edition | 性能、安全、并发 |
| 异步运行时 | tokio | 全特性异步 |
| JS 运行时 | QuickJS (rquickjs) | 轻量 JS 引擎 |
| HTTP 服务 | axum | REST API |
| gRPC 服务 | tonic + prost | RPC API |
| 桌面客户端 | Tauri v2 + React | 跨平台桌面应用 |
| 前端框架 | React 18 + TypeScript | SPA 架构 |
| 状态管理 | Zustand | 轻量级 React store |
| 编辑器 | Monaco Editor | 工作流脚本编辑 |
| 数据库 | SeaORM (SQLite/MySQL) | ORM + 可切换后端 |
| 样式 | Tailwind CSS | 原子化 CSS |
| Windows UIA | windows crate | Microsoft 官方 Rust 绑定 |
| 序列化 | serde + serde_json | 标准 |
| 错误处理 | thiserror + anyhow | 库级 + 应用级 |
| CLI | clap | 命令行参数 |
| 构建脚本 | just | 类 Make 的任务运行器 |

## 项目结构

```
rpa/
├── Cargo.toml                    # Workspace 根配置
├── justfile                      # 构建/开发脚本
├── scripts/dev.sh                # Shell 回退脚本（无 just 可用）
├── proto/
│   └── rpa.proto                 # gRPC 服务定义
├── _docs/
│   ├── design.md                 # 架构设计文档
│   └── uml.md                    # UML 图
├── crates/
│   ├── rpa-core/                 # 核心数据结构、trait、错误类型
│   ├── rpa-engine/               # VM 执行引擎（指令分发、多策略查找、重试、取消）
│   ├── rpa-workflow/             # JSON 工作流解析、编译、校验
│   ├── rpa-js/                   # QuickJS 运行时、JS ↔ Rust 桥接
│   ├── rpa-plugin/               # 插件加载（ZIP）、生命周期、沙箱
│   ├── rpa-perception/           # UIA/OCR/Image/Window 元素查找
│   ├── rpa-action/               # 鼠标键盘操作（Windows SendInput）
│   ├── rpa-api/                  # HTTP REST (axum) + gRPC (tonic)
│   └── rpa-orchestrator/         # 任务调度、并发控制、状态追踪
└── apps/
    ├── server/                   # 独立服务器二进制（HTTP + gRPC）
    └── desktop/                  # Tauri 桌面客户端
        ├── src/                  # React 前端源码
        │   ├── components/       # Layout、Sidebar 等通用组件
        │   ├── pages/            # 5 个页面：插件、工作流、任务、调试、设置
        │   ├── store/            # Zustand 状态管理（4 个 store）
        │   ├── lib/              # 工具函数、API 封装
        │   └── App.tsx           # 路由入口
        ├── src-tauri/            # Rust 后端
        │   ├── src/
        │   │   ├── commands/     # Tauri 命令（plugin/workflow/task/debug/settings/engine）
        │   │   ├── storage/      # SeaORM 数据层（entities + migration + connection）
        │   │   ├── engine.rs     # 引擎子进程管理
        │   │   ├── grpc_client.rs # gRPC 客户端封装
        │   │   └── lib.rs        # Tauri 入口 + 命令注册
        │   └── tauri.conf.json   # Tauri 配置
        ├── package.json          # Node 依赖
        ├── vite.config.ts        # Vite 配置
        └── tailwind.config.js    # Tailwind 配置
```

## 架构总览

```
┌──────────────────────────────────────────────────────────────┐
│                     Desktop Client (Tauri v2)                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────┐ ┌───────┐ │
│  │ Plugins  │ │Workflows │ │  Tasks   │ │Debug │ │Settings│ │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └──┬───┘ └───┬───┘ │
│       │            │            │           │          │      │
│  ┌────┴────────────┴────────────┴───────────┴──────────┘──┐ │
│  │              Tauri Commands (Rust)                       │ │
│  │    ┌────────────┐  ┌────────────┐  ┌──────────────┐     │ │
│  │    │  SeaORM    │  │ gRPC Client│  │Engine Process│     │ │
│  │    │ (SQLite/  )│  │            │  │  Manager     │     │ │
│  │    │  MySQL)    │  │            │  │              │     │ │
│  │    └────────────┘  └─────┬──────┘  └──────┬───────┘     │ │
│  └──────────────────────────┼─────────────────┼─────────────┘ │
└─────────────────────────────┼─────────────────┼───────────────┘
                              │                 │
                    ┌─────────▼─────────┐       │
                    │  API Layer        │       │
                    │  (HTTP / gRPC)    │◄──────┘
                    └─────────┬─────────┘
                              │
                    Orchestrator（调度）
                              │
                    Automation VM（执行核心）
                              │
              ┌───────────────┼───────────────┐
              │               │               │
        Workflow Engine  JS Runtime    Plugin System
              │               │               │
              └───────────────┼───────────────┘
                              │
                Perception（UIA/OCR/Image/Window）+ Action（鼠标键盘）
```

**核心执行流程**：

1. 外部调用 → API → Orchestrator → VM
2. VM 加载 Instruction 序列（来自 JSON 工作流或 JS 脚本）
3. 每条指令：Resolve Target → Find Element → Execute Action → Update Context
4. 元素查找按降级链：UIA → OCR → Image → Position → Window

---

## 快速开始

### 前置条件

- **Rust toolchain**: `rustup default stable`
- **Yarn** (包管理器): `corepack enable` 或 `npm install -g yarn`
- **just** (任务运行器): `cargo install just`
- **protoc** (gRPC 编译): `brew install protobuf` 或 Windows 上安装
- **Windows cross-compile target** (可选): `rustup target add x86_64-pc-windows-gnu`

### 服务端开发

```bash
# 编译检查
just dev

# 运行所有测试
just test

# 编译整个 workspace
just build

# 运行服务器（开发模式）
just run

# 代码格式化 + lint
just lint

# 清理构建产物
just clean
```

### 桌面客户端开发

```bash
# 首次初始化（安装前端依赖）
just desktop-init

# 启动开发模式（Tauri 热加载：前端 HMR + Rust 后端自动重编译）
just desktop-dev

# 仅启动前端开发服务器（Vite HMR，无 Tauri 窗口）
just desktop-fe

# 检查桌面端 Rust 后端编译
just desktop-check

# 构建生产版本
just desktop-build

# 仅构建前端
just desktop-build-fe
```

**Tauri 开发模式说明**：

`just desktop-dev` 启动时：
1. Vite 开发服务器在 `http://localhost:5173` 启动（前端热更新）
2. Tauri 窗口自动打开并连接到 Vite 服务器
3. 修改前端代码（`.tsx`/`.ts`/`.css`）→ 浏览器即时刷新
4. 修改 Rust 代码（`src-tauri/src/`）→ 自动重编译 + 窗口重启
5. 两者同时进行，无需手动重启

`just desktop-fe` 仅启动前端 Vite 服务器，适合纯 UI 开发（不需要 Tauri API 时更快）。

### 所有可用命令

| 命令 | 说明 |
|------|------|
| `just dev` | 快速编译检查 (cargo check) |
| `just build` | Debug 构建 |
| `just build-release` | Release 构建 |
| `just build-server` | 构建 server 二进制 |
| `just build-desktop` | 构建 desktop 二进制 (release) |
| `just build-win` | 交叉编译 Windows (GNU) |
| `just build-win-msvc` | 编译 Windows MSVC |
| `just test` | 运行全部测试 |
| `just test-core` | 运行 rpa-core 测试 |
| `just test-engine` | 运行 rpa-engine 测试 |
| `just test-fast` | 运行 core + engine 快速测试 |
| `just test-desktop` | 运行桌面端 Rust 测试 |
| `just fmt` | 格式化代码 |
| `just fmt-check` | 检查格式 |
| `just lint` | 运行 clippy（workspace + desktop） |
| `just check` | 完整质量检查（fmt + lint + test） |
| `just run` | 运行服务器 (debug) |
| `just run-release` | 运行服务器 (release) |
| `just gen-proto` | 生成 gRPC Rust 代码 |
| `just doc` | 生成并打开文档 |
| `just doc-build` | 生成文档 |
| `just clean` | 清理构建产物 |
| `just reset` | 删除 lock + 清理 + 重新检查 |
| `just ci` | 完整 CI 流水线 |
| `just desktop-dev` | 启动 Tauri 开发模式（热加载） |
| `just desktop-fe` | 仅前端 Vite 开发服务器 |
| `just desktop-build` | 构建 Tauri 生产版本 |
| `just desktop-build-fe` | 仅构建前端 |
| `just desktop-check` | 检查桌面端编译 |
| `just desktop-install` | 安装前端依赖 (yarn) |
| `just desktop-init` | 首次初始化（安装依赖 + 初始化） |

无 `just` 时可用 `./scripts/dev.sh <command>` 作为替代，命令名相同。

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

# 桌面端编译检查
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml

# 桌面端测试
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
```

### 手动 yarn 命令（桌面客户端）

```bash
cd apps/desktop

# 安装依赖
yarn

# 启动前端开发服务器（仅 HMR，无 Tauri）
yarn dev

# 启动 Tauri 开发模式（前端 HMR + Rust 热重编译）
yarn tauri:dev

# 构建前端
yarn build

# 构建 Tauri 生产包
yarn tauri:build
```

---

## 桌面客户端

### 架构

桌面客户端基于 Tauri v2，采用前后端分离架构：

- **前端**：React 18 + TypeScript + Vite，Zustand 状态管理，Tailwind CSS 样式
- **后端**：Rust (Tauri)，负责系统 API 调用、数据库操作、引擎子进程管理
- **通信**：前端通过 `@tauri-apps/api` 的 `invoke()` 调用 Rust 命令
- **数据库**：SeaORM + SQLite（默认），支持切换到 MySQL

### 页面功能

| 页面 | 路由 | 功能 |
|------|------|------|
| 插件管理 | `/plugins` | 插件列表、导入/导出 ZIP、启用/禁用 |
| 插件详情 | `/plugins/:id` | 插件信息、所属工作流列表 |
| 工作流列表 | `/workflows` | 所有工作流、按插件筛选 |
| 工作流编辑器 | `/workflows/:id/edit` | Monaco Editor 编辑 JSON 工作流脚本 |
| 任务监控 | `/tasks` | 实时任务列表、执行日志流 |
| 调试可视化 | `/debug` | 调试开关、高亮配置、截图预览 |
| 设置 | `/settings` | 数据库连接、引擎配置、调试选项 |

### 数据模型

四张核心表：

- **plugins** — 插件（名称、版本、作者、ZIP 路径、启用状态）
- **workflows** — 工作流（所属插件、名称、脚本内容、运行次数）
- **task_history** — 任务历史（状态、日志、错误信息、时间）
- **settings** — 键值对设置（数据库 URL、引擎配置等）

### 引擎嵌入模式

桌面客户端可以：
1. **嵌入模式**：启动 RPA Engine 作为子进程，通过 gRPC 通信
2. **独立模式**：连接到外部运行的 Engine 服务

通过设置页面切换连接模式和配置引擎二进制路径。

---

## 开发阶段与依赖关系

```
Phase 0: Workspace 脚手架 ✅
    └→ Phase 1: core + engine ✅
          ├→ Phase 2: perception + action（可并行）
          ├→ Phase 3: workflow（可并行）
          ├→ Phase 4: js（可并行）
          ├→ Phase 5: plugin
          ├→ Phase 6: api + orchestrator
          └→ Phase 8: Desktop Client (Tauri v2) ✅
                └→ Phase 7: 集成与打磨
```

**当前进度**：Phase 0-1 已完成（核心类型 + VM 引擎，33 个测试通过），Phase 2-6 为骨架代码，Desktop Client 脚手架已完成。

---

## 核心 Crate 详解

### rpa-core — 共享数据结构

所有其他 crate 的基础依赖，定义了核心类型和 trait：

| 文件 | 内容 |
|------|------|
| `instruction.rs` | `Instruction` 枚举：Click, Input, Extract, Call, Loop, If, Break, Return, Log, Scroll, MouseMove, MouseDown, MouseUp, Drag, SetForeground, MoveWindow, Screenshot, OcrRegion 等 |
| `target.rs` | `Target` 枚举：UIA(selector), Image(path), Text(pattern), Position(x,y), Window(selector), Region(window, rect) |
| `element.rs` | `Element` 结构体：id, bounds, text, element_type, platform_handle, process_id, process_name, window_title |
| `value.rs` | `Value` 动态类型：Null, Bool, Number, String, Array, Object |
| `context.rs` | `Context`：变量表、调用栈、重试配置；`RetryConfig` + 退避策略 |
| `condition.rs` | `Condition` 枚举：VarEquals, VarNotEmpty, ElementExists, And/Or/Not |
| `error.rs` | `RpaError` — 统一错误类型（含 WindowNotFound, OcrFailed, ScreenshotFailed, ProcessNotFound） |
| `traits.rs` | `Perceptor`, `Actor`, `WorkflowProvider`, `JsRuntime`, `WindowPerceptor`, `OcrEngine`, `ScreenCapturer`, `DebugCapturer` — 核心 trait |
| `task.rs` | `TaskStatus`, `TaskResult`, `TaskPriority` |
| `plugin.rs` | `PluginManifest`, `Permission` |
| `spec.rs` | `DataSpec`, `FieldSpec`, `ExtractAttribute` |
| `window_selector.rs` | `WindowSelector` — 按进程名/窗口标题/类名定位窗口 |

### rpa-engine — VM 执行引擎

| 文件 | 内容 |
|------|------|
| `vm.rs` | `Vm` struct — builder 模式注入 Perceptor/Actor/JsRuntime/WindowPerceptor/OcrEngine/ScreenCapturer/DebugCapturer |
| `executor.rs` | `Executor` — 指令分发，控制流（Continue/Break/Return） |
| `finder.rs` | `MultiStrategyFinder` — 多策略降级查找（UIA→OCR→Image→Position→Window） |
| `retry.rs` | 重试逻辑 + 退避策略（Fixed/Linear/Exponential） |
| `cancellation.rs` | `CancellationToken` — 基于 tokio watch channel 的协作式取消 |
| `events.rs` | `ExecutionEvent` — 执行过程事件流（含调试高亮、截图、操作记录） |

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
    // 新增：鼠标增强、窗口前置、截图
    async fn mouse_move(&self, x: i32, y: i32) -> Result<()>;
    async fn mouse_down(&self, button: MouseButton, x: i32, y: i32) -> Result<()>;
    async fn mouse_up(&self, button: MouseButton, x: i32, y: i32) -> Result<()>;
    async fn set_foreground(&self, element: &Element) -> Result<()>;
    async fn screenshot(&self, region: Option<Rect>) -> Result<Vec<u8>>;
}

// 窗口感知
#[async_trait]
pub trait WindowPerceptor: Send + Sync {
    async fn find_window(&self, selector: &WindowSelector) -> Result<Element>;
    async fn find_all_windows(&self, selector: &WindowSelector) -> Result<Vec<Element>>;
    async fn set_foreground(&self, element: &Element) -> Result<()>;
    async fn get_foreground_window(&self) -> Result<Element>;
}

// OCR 引擎
#[async_trait]
pub trait OcrEngine: Send + Sync {
    async fn recognize(&self, image_data: &[u8], region: Option<Rect>) -> Result<String>;
}

// 截图
#[async_trait]
pub trait ScreenCapturer: Send + Sync {
    async fn capture_screen(&self) -> Result<Vec<u8>>;
    async fn capture_region(&self, region: Rect) -> Result<Vec<u8>>;
    async fn capture_window(&self, element: &Element) -> Result<Vec<u8>>;
}

// 调试可视化
#[async_trait]
pub trait DebugCapturer: Send + Sync {
    async fn capture_with_highlight(
        &self, region: Option<Rect>, highlights: Vec<DebugHighlight>, save_path: &str,
    ) -> Result<String>;
}
```

---

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

`target` 支持简写：`"发送按钮"` 自动按 UIA→OCR→Image→Position 降级查找。

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

// 窗口操作（新增）
const win = await rpa.findWindow({ processName: "notepad.exe" });
await rpa.setForeground(win);
await rpa.click({ window: win, offsetX: 100, offsetY: 50 });

// 区域 OCR（新增）
const text = await rpa.ocr({ window: win, region: { x: 10, y: 20, width: 200, height: 30 } });

// 截图（新增）
await rpa.screenshot({ window: win, savePath: "screenshot.png" });

// 鼠标增强（新增）
await rpa.mouseMove(500, 300);
await rpa.drag(fromEl, toEl);

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

---

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
7. **`apps/desktop/src-tauri/src/lib.rs`** — 桌面端 Tauri 命令注册
8. **`apps/desktop/src/App.tsx`** — 前端路由结构

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

### 添加桌面端新页面

1. 在 `apps/desktop/src/pages/` 创建新页面组件
2. 在 `apps/desktop/src/App.tsx` 添加路由
3. 在 `apps/desktop/src/components/Sidebar.tsx` 添加导航项
4. 如需 Rust 后端支持，在 `apps/desktop/src-tauri/src/commands/` 添加命令
5. 在 `apps/desktop/src-tauri/src/lib.rs` 注册新命令

### 关键设计决策

| 决策 | 选择 | 原因 |
|------|------|------|
| VM 输入是 trait object | `Box<dyn Perceptor>`, `Arc<dyn Actor>` | 运行时注入平台实现 |
| JS 和 Rust 通信 | mpsc channel + oneshot | QuickJS 单线程，channel 异步桥接 |
| 执行流控制流 | ControlFlow enum | 避免 Option 嵌套，Break/Return 语义清晰 |
| 多策略查找 | 降级链 | UIA→OCR→Image→Position→Window，一个失败自动尝试下一个 |
| 插件隔离 | 独立 JsRuntime | 每个 ZIP 解压到内存，JS 上下文隔离 |
| 工作流两阶段 | JSON→Step→Instruction | 校验在中间步骤，运行时只有 Instruction |
| 桌面端框架 | Tauri v2 | Rust 后端 + Web 前端，体积小、性能好 |
| 桌面端数据库 | SeaORM + SQLite | 零配置嵌入式数据库，可选 MySQL |
| 前端状态管理 | Zustand | 轻量、TypeScript 友好 |
| Tauri 热加载 | `tauri dev` + Vite HMR | 前端即时刷新，Rust 后端自动重编译 |

### 常见坑点

- **Windows UIA 必须在 Windows 上测试**：UIA 的 `windows` crate 只在 Windows 上编译。macOS 上开发时用 `MockPerceptor`。
- **QuickJS 是单线程的**：不能多线程同时调用 JS，必须通过 `spawn_blocking` 或 channel 通信。
- **`Instruction::Break` 的处理**：在 `execute()` 中返回 `ControlFlow::Break`，由 `execute_block` 的循环捕获。
- **gRPC proto 变更后需手动 `cargo build`**：tonic 会在编译时生成 Rust 代码。
- **Tauri v2 `devtools` 字段已移除**：如需 DevTools，在 debug 模式下默认可用，不需要配置。
- **桌面端 Cargo 目标目录共享**：`apps/desktop/src-tauri/.cargo/config.toml` 设置 `target = "../../target"`，避免 workspace 和桌面端分别维护构建缓存。
- **dev 模式下依赖优化**：`Cargo.toml` 中 `[profile.dev.package."*"]` 将依赖库设为 opt-level=2，加速增量编译。
- **前端修改不需要重启 Tauri**：Vite HMR 自动更新；Rust 代码修改会触发自动重编译。

### 测试策略

```bash
# 核心类型测试（纯数据结构，macOS 可运行）
just test-core

# VM 引擎测试（使用 Mock，macOS 可运行）
just test-engine

# 全部单元测试（macOS 可运行，Mock 平台层）
just test

# 桌面端 Rust 后端测试
just test-desktop

# Windows 集成测试（必须在 Windows 上运行）
cargo test -p rpa-perception --target x86_64-pc-windows-gnu
cargo test -p rpa-action --target x86_64-pc-windows-gnu

# 桌面端 E2E 测试：启动 Tauri 应用 → 手动验证各页面功能
just desktop-dev
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

---

## 配置

### 服务器配置 `config.toml`

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

[debug]
enabled = false
highlight = true
highlight_duration_ms = 500
screenshot_on_step = false
screenshot_dir = "./debug_screenshots"
slow_motion_ms = 0
```

### 桌面客户端配置

桌面客户端通过 UI 设置页面配置（数据存储在 SQLite 中），支持：
- 数据库连接 URL（默认 SQLite，可切换 MySQL）
- 引擎二进制路径
- 默认超时、端口
- 调试选项（高亮持续时间、慢动作延迟、每步截图）

---

## 依赖版本

核心依赖在 `Cargo.toml` 的 `[workspace.dependencies]` 中统一管理。各 crate 通过 `{ workspace = true }` 引用，避免版本冲突。

桌面端独立依赖（`apps/desktop/src-tauri/Cargo.toml`）包括 tauri 2、sea-orm、tokio、zip 等。

前端依赖（`apps/desktop/package.json`）包括 React 18、react-router-dom 7、zustand 5、@monaco-editor/react、Tailwind CSS 等。

## License

MIT