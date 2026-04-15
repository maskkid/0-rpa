自动化操作虚拟机 + 插件生态平台

* 架构设计（Architecture）
* 工程设计（Engineering）
* 开发规范（Dev Guide）
* 核心执行逻辑（Core Logic）
* 扩展与演进（Evolution）

👉 你可以直接当作**项目技术白皮书 / 内部设计文档 / 开发手册**

---

# 📄 一、项目总览（Project Overview）

## 1.1 项目定义

> **RPA Automation VM（插件驱动自动化执行平台）**

核心能力：

* GUI 自动化（RPA / UIA / OCR）
* Workflow 编排（JSON + JS）
* 插件化扩展（软件适配）
* JS运行时（高灵活性）
* API服务化（HTTP / RPC）

---

## 1.2 核心设计目标

| 目标  | 说明               |
| --- | ---------------- |
| 灵活性 | ≈ 手写代码           |
| 可扩展 | 插件化              |
| 可维护 | VM抽象             |
| 稳定性 | retry + fallback |
| 跨软件 | 通用能力             |

---

---

# 🏗 二、系统架构设计（Architecture）

---

## 2.1 总体架构

```text id="arch_final"
API Layer (HTTP / RPC)
        ↓
Orchestrator（调度）
        ↓
Automation VM（执行核心）
        ↓
┌───────────────┬───────────────┬───────────────┐
│ Workflow Engine │ JS Runtime     │ Plugin System │
└───────────────┴───────────────┴───────────────┘
        ↓
Perception（UIA/OCR/Image） + Action（输入控制）
```

---

## 2.2 分层职责

| 层            | 职责   |
| ------------ | ---- |
| API          | 外部调用 |
| Orchestrator | 任务调度 |
| VM           | 指令执行 |
| Workflow     | 编排   |
| JS           | 灵活逻辑 |
| Plugin       | 软件适配 |
| Perception   | 识别   |
| Action       | 执行   |

---

---

# 📦 三、工程结构设计（Rust Workspace）

---

## 3.1 项目结构

```text id="proj_final"
rpa/
 ├── crates/
 │    ├── rpa-core/
 │    ├── rpa-engine/
 │    ├── rpa-workflow/
 │    ├── rpa-plugin/
 │    ├── rpa-js/
 │    ├── rpa-perception/
 │    ├── rpa-action/
 │    ├── rpa-api/
 │    └── rpa-orchestrator/
 └── apps/server/
```

---

## 3.2 模块职责

| crate        | 作用         |
| ------------ | ---------- |
| core         | 数据结构       |
| engine       | VM         |
| workflow     | JSON解析     |
| plugin       | 插件加载       |
| js           | JS runtime |
| perception   | OCR/UIA    |
| action       | 鼠标键盘       |
| api          | HTTP       |
| orchestrator | 调度         |

---

---

# 🧱 四、核心数据模型

---

## 4.1 Instruction（核心）

```rust id="ins_final"
enum Instruction {
    Click(Target),
    Input(String),
    Extract(DataSpec),

    Call { workflow: String, args: Value },

    Loop { max: Option<u32>, body: Vec<Instruction> },

    If {
        condition: Condition,
        then_body: Vec<Instruction>,
        else_body: Option<Vec<Instruction>>,
    },

    Break,
}
```

---

## 4.2 Target（定位抽象）

```rust id="target_final"
enum Target {
    UIA { name: String },
    Image { path: String },
    Text { pattern: String },
    Position { x: i32, y: i32 },
}
```

---

## 4.3 Element

```rust id="el_final"
struct Element {
    id: String,
    bounds: Rect,
    text: Option<String>,
}
```

---

## 4.4 Context

```rust id="ctx_final"
struct Context {
    vars: HashMap<String, Value>,
}
```

---

---

# ⚙️ 五、核心执行逻辑（VM）

---

## 5.1 执行流程

```text id="vm_flow"
Instruction → Resolve → Execute → Update Context
```

---

## 5.2 核心执行伪代码

```rust id="vm_exec"
fn exec(instr) {
    match instr {
        Click(target) => {
            el = find(target)
            click(el)
        }

        Call => load_workflow + run

        Loop => 循环执行

        If => 条件判断
    }
}
```

---

## 5.3 多策略查找

```text id="find_strategy"
UIA → OCR → Image → Position
```

---

---

# 🔄 六、Workflow 引擎

---

## 6.1 JSON结构

```json id="wf_final"
{
  "steps": [
    { "type": "click", "target": "发送按钮" },
    { "type": "call", "workflow": "open_chat" }
  ]
}
```

---

## 6.2 执行流程

```text id="wf_flow"
JSON → Step → Instruction → VM执行
```

---

---

# 🧩 七、插件系统

---

## 7.1 插件结构

```text id="plugin_final"
plugin.zip
 ├── manifest.json
 ├── workflows/
 ├── scripts/
 ├── assets/
```

---

## 7.2 插件职责

* 提供 workflow
* 提供 JS
* 提供资源
* 可扩展能力

---

## 7.3 插件生命周期

```text id="plugin_life"
load → init → run → destroy
```

---

---

# 🧠 八、JS运行时设计

---

使用：

* QuickJS

---

## 8.1 API示例

```js id="js_final"
let el = await find("发送按钮")
await click(el)
```

---

## 8.2 JS职责

* 控制流程
* 数据处理
* 调用VM能力

---

## 8.3 Bridge

```rust id="bridge_final"
JS → Rust → VM
```

---

---

# 🌐 九、API设计

---

## 9.1 HTTP接口

```http id="api_final"
POST /run
```

---

## 9.2 请求

```json id="api_req"
{
  "plugin": "qianniu",
  "workflow": "get_messages"
}
```

---

---

# 🔁 十、调度系统

---

## 功能

* 队列
* 并发
* 状态

---

---

# 🛡 十一、稳定性设计

---

## 11.1 retry

```text id="retry_final"
失败自动重试
```

---

## 11.2 timeout

---

## 11.3 fallback

```text id="fallback_final"
UIA → OCR → Image
```

---

---

# 🚀 十二、开发规范（非常重要）

---

## 12.1 模块开发顺序

1. core
2. engine
3. workflow
4. js
5. plugin
6. api

---

## 12.2 编码原则

* 所有能力异步化
* 不写死业务逻辑
* 所有操作通过 VM

---

---

# 📈 十三、扩展设计

---

## 插件能力扩展

```rust id="ext_final"
register("smart_click", fn)
```

---

---

# 🧠 十四、关键设计总结

---

## 1️⃣ VM抽象（核心）

---

## 2️⃣ JS灵活性（灵魂）

---

## 3️⃣ 插件隔离（扩展）


