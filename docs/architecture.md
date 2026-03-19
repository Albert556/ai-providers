# AI Providers (aip) 架构与实现细节

## 目录

- [项目概览](#项目概览)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [架构设计](#架构设计)
  - [分层架构](#分层架构)
  - [Provider 抽象层](#provider-抽象层)
  - [Profile 管理层](#profile-管理层)
  - [存储层](#存储层)
  - [命令层](#命令层)
- [数据模型](#数据模型)
  - [文件系统布局](#文件系统布局)
  - [状态文件格式](#状态文件格式)
  - [Profile 文件格式](#profile-文件格式)
- [CLI 设计](#cli-设计)
  - [命令结构](#命令结构)
  - [命令参考](#命令参考)
- [关键实现细节](#关键实现细节)
  - [原子写入](#原子写入)
  - [文件权限](#文件权限)
  - [Profile 名称校验](#profile-名称校验)
  - [编辑器选择链](#编辑器选择链)
  - [JSON 校验与重编辑循环](#json-校验与重编辑循环)
  - [错误处理](#错误处理)
  - [彩色输出](#彩色输出)
- [核心流程](#核心流程)
  - [添加 Profile](#添加-profile)
  - [切换 Profile](#切换-profile)
  - [删除 Profile](#删除-profile)
- [安全性设计](#安全性设计)
- [扩展指南：添加新 Provider](#扩展指南添加新-provider)
- [设计决策记录](#设计决策记录)

---

## 项目概览

`aip`（AI Providers）是一个 Rust CLI 工具，用于管理 AI 编程工具的配置 profile。用户可以为不同开发场景（工作、个人、测试等）创建独立的配置文件，通过一条命令快速切换。

当前版本（v0.1.0）实现了 Claude Code 支持。架构通过 `Provider` trait 抽象，预留了多 provider 扩展能力。

## 技术栈

| 依赖 | 版本 | 用途 |
|------|------|------|
| `clap` | 4.5 | CLI 参数解析，derive 宏，嵌套子命令 |
| `serde` | 1.0 | 序列化/反序列化框架 |
| `serde_json` | 1.0 | JSON 读写 |
| `anyhow` | 1.0 | 应用层错误处理，`.context()` 错误上下文 |
| `colored` | 2.1 | 终端彩色输出 |

Rust edition: 2021

## 项目结构

```
src/
├── main.rs                  # 入口，CLI 定义，命令分发
├── provider/                # Provider 抽象层
│   ├── mod.rs               # Provider trait 定义
│   └── claude.rs            # ClaudeProvider 实现
├── profile/                 # Profile 管理核心
│   ├── mod.rs               # 模块导出
│   ├── manager.rs           # ProfileManager（泛型，接收 &dyn Provider）
│   └── storage.rs           # 文件 I/O（原子写入、状态管理）
└── commands/                # 子命令实现
    ├── mod.rs               # 模块导出
    ├── list.rs              # list / ls
    ├── current.rs           # current
    ├── show.rs              # show
    ├── config.rs            # config
    ├── add.rs               # add
    ├── delete.rs            # delete
    ├── edit.rs              # edit
    └── use_cmd.rs           # use
```

## 架构设计

### 分层架构

```
┌─────────────────────────────────────┐
│           main.rs (CLI)             │  clap 解析 → 命令分发
├─────────────────────────────────────┤
│         commands/*                  │  各子命令的交互逻辑（输出、确认）
├─────────────────────────────────────┤
│       ProfileManager                │  业务逻辑（校验、协调）
├─────────────────────────────────────┤
│     Provider trait                  │  provider 抽象（name、config_path）
├─────────────────────────────────────┤
│         storage                     │  文件 I/O（原子写入、state 读写）
└─────────────────────────────────────┘
```

数据流方向：自上而下。CLI 层解析参数，调用 commands 层；commands 层调用 `ProfileManager`；`ProfileManager` 通过 `Provider` 获取配置路径，通过 `storage` 执行文件操作。

### Provider 抽象层

**文件**: `src/provider/mod.rs`

```rust
pub trait Provider {
    fn name(&self) -> &str;
    fn config_path(&self) -> PathBuf;
    fn validate_config(&self, _content: &serde_json::Value) -> Result<()> {
        Ok(())  // 默认实现：接受任何合法 JSON
    }
}
```

三个方法的职责：

| 方法 | 职责 | 示例返回值 |
|------|------|-----------|
| `name()` | 标识名，用于 CLI 子命令名和存储目录名 | `"claude"` |
| `config_path()` | 该 provider 的活跃配置文件绝对路径 | `~/.claude/settings.json` |
| `validate_config()` | 可选的配置内容校验 | 默认返回 `Ok(())` |

**ClaudeProvider 实现** (`src/provider/claude.rs`)：

```rust
pub struct ClaudeProvider;

impl Provider for ClaudeProvider {
    fn name(&self) -> &str { "claude" }

    fn config_path(&self) -> PathBuf {
        PathBuf::from(env::var("HOME").expect("..."))
            .join(".claude")
            .join("settings.json")
    }
}
```

`ClaudeProvider` 没有覆盖 `validate_config()`，因此接受任何合法 JSON 作为配置。

### Profile 管理层

**文件**: `src/profile/manager.rs`

`ProfileManager` 是核心业务逻辑所在，它持有一个 `&dyn Provider` 引用，所有操作对 provider 是泛型的。

```rust
pub struct ProfileManager<'a> {
    provider: &'a dyn Provider,
    profiles_dir: PathBuf,    // ~/.ai-providers/<provider_name>/
    state_file: PathBuf,      // ~/.ai-providers/state.json
}
```

**构造过程** (`new`)：
1. 读取 `$HOME` 环境变量
2. 计算 `profiles_dir` = `$HOME/.ai-providers/<provider.name()>/`
3. 计算 `state_file` = `$HOME/.ai-providers/state.json`
4. 如果 `profiles_dir` 不存在则自动创建（`create_dir_all`）

**公开方法一览**：

| 方法 | 签名 | 说明 |
|------|------|------|
| `list_profiles` | `() -> Result<Vec<String>>` | 扫描 profiles_dir 下所有 `.json` 文件，返回排序后的文件名（去扩展名） |
| `get_current_profile` | `() -> Result<Option<String>>` | 从 state.json 读取当前 provider 的活跃 profile |
| `get_profile` | `(name) -> Result<Value>` | 读取指定 profile 的 JSON 内容 |
| `get_active_config` | `() -> Result<Value>` | 读取 provider 的活跃配置文件（如 `~/.claude/settings.json`） |
| `add_profile` | `(name, source) -> Result<()>` | 创建新 profile，来源可以是空、当前配置或已有 profile |
| `delete_profile` | `(name) -> Result<()>` | 删除 profile 文件，若为当前 profile 则同时清除 state |
| `use_profile` | `(name) -> Result<()>` | 将 profile 内容写入配置文件，更新 state |
| `profile_exists` | `(name) -> bool` | 检查 profile 文件是否存在 |
| `profile_path` | `(name) -> PathBuf` | 返回 profile 文件路径 |
| `validate_profile_name` | `(name) -> Result<()>` | 校验名称合法性 |
| `provider_name` | `() -> &str` | 返回 provider 名称 |
| `is_common_profile` | `(name) -> bool` | 判断是否为 common profile（静态方法） |
| `get_common_config` | `() -> Result<Option<Value>>` | 读取 common 配置内容，不存在则返回 None |
| `has_common_config` | `() -> bool` | 检查 common 配置是否存在 |

**ProfileSource 枚举**：

```rust
pub enum ProfileSource {
    Empty,                  // 创建空 profile ({})
    FromCurrent,            // 从当前活跃配置复制
    FromProfile(String),    // 从已有 profile 复制
}
```

### 存储层

**文件**: `src/profile/storage.rs`

纯函数式设计，所有函数接收路径参数，不持有状态。

| 函数 | 说明 |
|------|------|
| `read_json(path)` | 读取文件并解析为 `serde_json::Value` |
| `write_json(path, value)` | 原子写入 JSON 到文件（见[原子写入](#原子写入)） |
| `remove_file(path)` | 删除文件 |
| `deep_merge(base, override_val)` | 深度合并两个 JSON 值，override 优先；对象递归合并，非对象（含数组）直接替换 |
| `read_current_profile(state_path, provider)` | 从 state.json 读取指定 provider 的当前 profile |
| `update_current_profile(state_path, provider, profile)` | 更新 state.json 中指定 provider 的当前 profile |

### 命令层

**文件**: `src/commands/*.rs`

每个命令是一个独立模块，导出 `execute` 函数。命令层负责：
- 用户交互（确认提示、输出格式化）
- 调用 `ProfileManager` 执行业务逻辑
- 彩色输出

命令层**不直接**操作文件系统或 state，全部委托给 `ProfileManager`。

## 数据模型

### 文件系统布局

```
~/.ai-providers/               # 顶层目录
├── state.json                 # 全局状态文件
├── claude/                    # Claude Code profiles 目录
│   ├── common.json            # 公共配置（可选，use 时自动合并）
│   ├── work.json
│   ├── personal.json
│   └── test.json
└── <future-provider>/         # 未来 provider 的 profiles 目录
    └── ...
```

### 状态文件格式

`~/.ai-providers/state.json` 用于记录每个 provider 当前激活的 profile。各 provider 独立，互不干扰。

```json
{
  "claude": {
    "current_profile": "work"
  }
}
```

- 当 provider 没有活跃 profile 时，对应的 key 会被**完全移除**（而不是设为 null）
- state.json 不存在时等同于所有 provider 无活跃 profile

### Profile 文件格式

Profile 文件是**纯配置**——其内容直接就是 provider 的 `settings.json` 内容，不包含任何元数据包装。

```json
{
  "model": "claude-opus-4-6",
  "permissions": {
    "allow": ["Read", "Grep", "Glob"],
    "ask": ["Edit", "Write"],
    "deny": ["Bash"]
  }
}
```

这意味着 `aip claude use work` 执行时，`~/.ai-providers/claude/work.json` 的内容会**原样**覆盖到 `~/.claude/settings.json`。

## CLI 设计

### 命令结构

采用 clap 的嵌套子命令模式：

```
aip <provider> <command> [args] [options]
```

当前只有 `claude` 一个 provider：

```
aip claude <command>
```

`main.rs` 中的分发流程：

```
Cli::parse()
  → ProviderCommand::Claude { command }
    → 创建 ClaudeProvider
    → 创建 ProfileManager::new(&provider)
    → handle_profile_command(manager, command)
      → match command → commands::*::execute()
```

### 命令参考

#### `list` (别名: `ls`)

列出所有 profile，当前激活的 profile 以绿色高亮显示并带 `*` 标记。

```
$ aip claude list
Available profiles:
  * work  (current)
    personal
    test
```

无 profile 时显示蓝色提示信息。

#### `current`

显示当前激活的 profile 名称。

```
$ aip claude current
Current profile: work
```

#### `show <profile> [--merged/-m]`

显示指定 profile 的完整 JSON 内容（pretty-print 格式）。

使用 `--merged` 时显示与 common 配置合并后的结果预览。

#### `config`

显示 provider 当前活跃配置文件（`~/.claude/settings.json`）的内容。文件不存在时显示黄色警告。

#### `add <profile> [--from <name>] [--empty]`

创建新 profile。三种来源模式：

| 参数 | 来源 | 说明 |
|------|------|------|
| 无 flag | `~/.claude/settings.json` | 默认行为，从当前配置复制 |
| `--empty` / `-e` | 空 JSON `{}` | 创建空白 profile |
| `--from <name>` / `-f <name>` | 已有 profile | 复制已有 profile 的内容 |

如果默认模式下 `settings.json` 不存在，会静默回退到创建空 profile `{}`。

#### `delete <profile> [-f/--force]`

删除指定 profile。

- 默认交互式确认（stdin 读取 y/n）
- `-f` 跳过确认
- 如果是当前激活 profile，先显示黄色警告
- 删除后如果该 profile 是当前 profile，自动清除 state

#### `edit <profile>`

用外部编辑器打开 profile 文件。编辑后自动校验 JSON 格式。

- **不会**自动将改动同步到 `~/.claude/settings.json`
- JSON 无效时提示用户是否重新编辑（循环直到有效或用户放弃）

#### `use <profile>`

切换到指定 profile。

1. 检查是否为 `common`（禁止切换到 common）
2. 读取 profile JSON 内容
3. 如果 `common.json` 存在，深度合并（profile 优先）
4. **覆盖** `~/.claude/settings.json`
5. 更新 `state.json` 中的 `current_profile`

**注意**：不会自动保存被覆盖的配置。用户需手动先执行 `aip claude add` 保存。

## 关键实现细节

### 原子写入

`storage::write_json` 使用临时文件 + `rename` 的方式确保写入的原子性：

```rust
// 1. 序列化为 pretty JSON
let content = serde_json::to_string_pretty(value)?;

// 2. 写入临时文件（同目录下 .tmp 扩展名）
let temp_path = path.with_extension("tmp");
fs::write(&temp_path, &content)?;

// 3. 设置权限（Unix only）
// ...

// 4. 原子 rename
fs::rename(&temp_path, path)?;
```

这保证了即使写入中途程序崩溃或断电，原始文件也不会被损坏——要么是完整的旧文件，要么是完整的新文件。

### 文件权限

在 Unix 系统上，所有通过 `write_json` 写入的文件权限设置为 `0600`（仅所有者可读写）：

```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&temp_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&temp_path, perms)?;
}
```

权限在 rename 之前设置于临时文件上，确保目标文件从创建瞬间就具有正确权限。

此设计通过条件编译 `#[cfg(unix)]` 实现，Windows 上不执行此逻辑。

### Profile 名称校验

`ProfileManager::validate_profile_name` 执行以下校验规则：

| 规则 | 错误信息 | 目的 |
|------|---------|------|
| 不能为空 | `"Profile name cannot be empty"` | 基础校验 |
| 不能包含 `/`、`\`、`..` | `"Profile name cannot contain path separators"` | 防止路径遍历 |
| 不能以 `.` 开头 | `"Profile name cannot start with a dot"` | 防止隐藏文件，防止 `..` 变体 |
| 不能是 `"state"` | `"'state' is a reserved name"` | 防止与 `state.json` 冲突 |

校验在 `add_profile` 时执行。`delete`、`use`、`show` 等命令不执行名称校验（它们通过 `profile_exists` 检查文件是否存在）。

### 编辑器选择链

`commands/edit.rs` 中的编辑器查找优先级：

```
$EDITOR → $VISUAL → vim（if available）→ vi（if available）→ nano
```

具体实现：
1. 先尝试 `$EDITOR` 环境变量
2. 再尝试 `$VISUAL` 环境变量
3. 如果都不存在，尝试执行 `vim --version` 检查 vim 是否可用
4. 再尝试 `vi --version`
5. 最后回退到 `nano`

### JSON 校验与重编辑循环

编辑命令在保存后进入校验循环：

```
打开编辑器 → 用户编辑 → 保存退出
  ↓
尝试解析 JSON
  ├── 成功 → 输出绿色 "Profile saved successfully"，结束
  └── 失败 → 输出红色错误详情
              ↓
           询问 "Do you want to edit again? (y/n)"
              ├── y → 重新打开编辑器（循环）
              └── n → 返回错误
```

### 错误处理

全程使用 `anyhow::Result` 和 `.context()` / `.with_context()` 提供丰富的错误上下文链。

`main.rs` 中的顶层错误处理：

```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", format!("Error: {}", e).red());
        std::process::exit(1);
    }
}
```

所有面向用户的错误以红色输出到 stderr，退出码为 1。

### 彩色输出

使用 `colored` crate，遵循以下规范：

| 场景 | 颜色 | 示例 |
|------|------|------|
| 成功 | 绿色 | `"Profile 'work' created successfully"` |
| 错误 | 红色 | `"Profile 'xxx' not found"` |
| 警告 | 黄色 | `"Warning: 'work' is the currently active profile"` |
| 提示 | 蓝色 | `"Tip: Use 'aip claude add <name>' to create a new profile"` |
| 当前 profile | 绿色加粗 | `* work  (current)` |

## 核心流程

### 添加 Profile

```
aip claude add <name> [--empty] [--from <src>]
        │
        ▼
validate_profile_name(name)
  ├── 失败 → 返回错误
  └── 通过
        │
        ▼
profile_exists(name)?
  ├── 已存在 → 返回 "Profile already exists" 错误
  └── 不存在
        │
        ▼
确定数据来源
  ├── --empty        → content = {}
  ├── --from <src>   → content = get_profile(src)?  （源不存在则报错）
  └── 默认           → content = get_active_config()
  │                     失败时静默回退到 {}
        │
        ▼
write_json(profiles_dir/<name>.json, content)
  （原子写入 + 0600 权限）
```

### 切换 Profile

```
aip claude use <name>
        │
        ▼
name == "common"?
  └── 是 → 返回错误（common 不可切换）
        │
        ▼
get_profile(name)
  ├── 不存在 → 返回错误
  └── 读取 JSON 内容
        │
        ▼
common.json 存在?
  ├── 是 → deep_merge(common, profile)，profile 优先
  └── 否 → 使用原始 profile 内容
        │
        ▼
write_json(~/.claude/settings.json, final_content)
  （原子写入，直接覆盖）
        │
        ▼
update_current_profile(state.json, "claude", Some(name))
  （更新 state）
```

#### 公共配置（Common Config）

`common.json` 是一个特殊的 profile，用于存放所有 profile 共享的基础配置。

**深度合并规则**：
- 对象：递归合并，profile 同名 key 覆盖 common
- 数组：profile 的数组直接替换 common 的数组（不做元素合并）
- 标量值：profile 优先

**示例**：
```
common.json:  {"permissions": {"allow": ["Read"]}, "theme": "dark"}
profile.json: {"permissions": {"allow": ["Read", "Write"]}, "model": "opus"}
合并结果:     {"permissions": {"allow": ["Read", "Write"]}, "theme": "dark", "model": "opus"}
```

**行为特点**：
- `use common` 被禁止，common 不是可切换的 profile
- `list` 中以 `[common]` 标记单独显示，不参与 current 标记
- common 不存在时行为与之前完全一致（向后兼容）
- 可通过 `add`、`edit`、`show`、`delete` 命令正常管理

### 删除 Profile

```
aip claude delete <name> [-f]
        │
        ▼
profile_exists(name)?
  ├── 不存在 → 输出红色错误，return Ok
  └── 存在
        │
        ▼
get_current_profile() == name?
  └── 是 → 输出黄色警告
        │
        ▼
force 参数?
  ├── 否 → 询问 "Are you sure? (y/n)"
  │         ├── n → 取消
  │         └── y → 继续
  └── 是 → 继续
        │
        ▼
remove_file(profiles_dir/<name>.json)
        │
        ▼
如果是当前 profile:
  update_current_profile(state.json, "claude", None)
  （移除 provider 的 state 条目）
```

## 安全性设计

| 机制 | 实现位置 | 说明 |
|------|---------|------|
| 原子写入 | `storage::write_json` | 临时文件 + rename，防止写入中断导致文件损坏 |
| 文件权限 0600 | `storage::write_json` | Unix 上仅所有者可读写，防止其他用户读取敏感配置 |
| 路径遍历防护 | `ProfileManager::validate_profile_name` | 禁止 `/`、`\`、`..`、`.` 开头 |
| 保留名防护 | `ProfileManager::validate_profile_name` | 禁止 `"state"` 作为 profile 名 |
| 删除确认 | `commands/delete.rs` | 默认交互式确认，防止误删 |
| 条件编译 | `#[cfg(unix)]` | 权限逻辑仅在 Unix 上编译，保持跨平台兼容 |

## 扩展指南：添加新 Provider

要添加新的 AI 工具支持（如 Codex），需要以下步骤：

### 1. 实现 Provider trait

创建 `src/provider/codex.rs`：

```rust
use std::path::PathBuf;
use super::Provider;

pub struct CodexProvider;

impl Provider for CodexProvider {
    fn name(&self) -> &str {
        "codex"
    }

    fn config_path(&self) -> PathBuf {
        let home = std::env::var("HOME").expect("Cannot determine home directory");
        PathBuf::from(home).join(".codex").join("config.json")
    }

    // 可选：覆盖 validate_config 添加自定义校验
}
```

### 2. 注册模块

在 `src/provider/mod.rs` 中添加：

```rust
pub mod codex;
```

### 3. 添加 CLI 子命令

在 `src/main.rs` 的 `ProviderCommand` 枚举中添加：

```rust
enum ProviderCommand {
    Claude { #[command(subcommand)] command: ProfileCommands },
    Codex { #[command(subcommand)] command: ProfileCommands },
}
```

### 4. 添加命令分发

在 `run()` 的 match 中添加分支：

```rust
ProviderCommand::Codex { command } => {
    let provider = CodexProvider;
    let manager = ProfileManager::new(&provider)?;
    handle_profile_command(&manager, command)?;
}
```

由于 `ProfileManager` 和所有命令都是基于 `&dyn Provider` 编写的，`handle_profile_command` 和 `commands/*` 无需任何修改。新 provider 自动获得全部 8 个子命令。

存储目录也会自动创建为 `~/.ai-providers/codex/`，state.json 中会独立追踪 `codex` 的当前 profile。

## 设计决策记录

| 决策 | 选择 | 理由 |
|------|------|------|
| CLI 结构 | `aip <provider> <command>` 嵌套子命令 | 各 provider 独立管理，命名空间清晰，可扩展 |
| 配置层级 | 仅用户级配置 | 项目级配置不需要频繁切换，保持简单 |
| 存储结构 | `~/.ai-providers/<provider>/` 按 provider 分目录 | 与 CLI 结构对应，直觉化 |
| 状态管理 | 各 provider 在 state.json 中独立追踪 | 允许 claude 在 `work`，codex 在 `dev`，互不干扰 |
| Profile 格式 | 纯配置，无元数据 | `profile.json` 内容 = `settings.json` 内容，简单直接 |
| 切换策略 | `use` 直接覆盖，不自动保存 | 显式优于隐式，用户需手动 `add` 保存 |
| edit 行为 | 只编辑 profile 文件 | 不自动同步到 settings.json，避免意外修改活跃配置 |
| 删除当前 profile | 允许 + 警告 + 确认 | 灵活但安全，删除后自动清除 state |
| 错误处理 | `anyhow::Result` 全程传播 | 适合应用层代码，错误上下文链清晰 |
| 文件写入 | 原子写入（temp + rename） | 防止中断导致文件损坏 |
| 公共配置 | `common.json` 作为特殊 profile，`use` 时深度合并 | 避免 profile 间重复配置，profile 优先级高于 common |
| common 不可切换 | `use common` 被禁止 | common 是基础层，不是可切换的 profile |
| 发布基线 | 比较 push 的 `before`/`after` 版本 | 判断“这次 push 是否引入新版本”比比较最新 tag 更准确 |
