# AI Providers (aip) 架构设计文档

## 项目背景

这是一个 Rust CLI 工具，用于管理多种 AI 编程工具的配置文件。用户在不同的开发场景下需要使用不同的配置（例如不同的 API 密钥、模型选择、权限设置等），手动编辑配置文件既繁琐又容易出错。本工具提供了一个统一的命令行界面来管理多个配置 profile，实现快速切换。

**当前版本（v1）先实现 Claude Code 支持，架构预留多 provider 扩展能力。**

## 设计决策记录

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 配置层级 | 仅用户级 `~/.claude/settings.json` | 保持简单，项目级配置通常不需要频繁切换 |
| CLI 结构 | 按 provider 分组子命令 `aip claude <cmd>` | 各 provider 独立管理，清晰不混淆 |
| 存储结构 | 按 provider 分目录 `~/.ai-providers/claude/` | 与 CLI 结构一致，便于管理 |
| 状态管理 | 各 provider 独立追踪当前 profile | claude 可以在 'work' 而 codex 在 'dev' |
| Profile 格式 | 纯配置，无元数据 | profile.json 内容 = settings.json 内容，简单直接 |
| 切换策略 | 不自动保存 | 用户需手动 `aip claude add` 保存当前配置 |
| edit 行为 | 只编辑 profile 文件 | 不自动同步到 settings.json，用户需 `use` 应用 |
| 删除当前 profile | 允许 + 警告 | 显示警告并要求确认，删除后清除 state 中的当前标记 |
| 代码架构 | Provider trait 抽象 | 先实现 ClaudeProvider，后续加 Codex 只需新增 struct |
| 工具命名 | 保持 `aip` (AI Providers) | 通用名称，适合未来多 provider 扩展 |

## 技术选型

### 依赖库

| 库名 | 版本 | 用途 |
|------|------|------|
| `clap` | 4.x | CLI 参数解析，支持嵌套子命令、derive API |
| `serde` | 1.x | 序列化/反序列化框架 |
| `serde_json` | 1.x | JSON 格式支持 |
| `anyhow` | 1.x | 错误处理，提供丰富的错误上下文 |
| `colored` | 2.x | 终端彩色输出 |

### 选型理由

- **clap**: Rust 生态中最成熟的 CLI 解析库，原生支持嵌套子命令（`aip claude list`），自动生成帮助信息
- **serde + serde_json**: 标准的 JSON 序列化方案，性能优秀，API 友好
- **anyhow**: 简化错误处理，适合应用层代码（非库代码）
- **colored**: 提供彩色输出，提升用户体验

## 架构设计

### 项目目录结构

```
ai-providers/
├── Cargo.toml
├── CLAUDE.md
├── PLAN.md
├── README.md
├── src/
│   ├── main.rs              # 程序入口，CLI 定义
│   ├── provider/            # Provider 抽象层
│   │   ├── mod.rs           # Provider trait 定义
│   │   └── claude.rs        # ClaudeProvider 实现
│   ├── profile/             # Profile 管理核心逻辑
│   │   ├── mod.rs
│   │   ├── manager.rs       # ProfileManager（通用，接收 Provider）
│   │   └── storage.rs       # 文件 I/O 操作
│   └── commands/            # 子命令实现
│       ├── mod.rs
│       ├── list.rs          # list 命令
│       ├── current.rs       # current 命令
│       ├── show.rs          # show 命令
│       ├── config.rs        # config 命令
│       ├── add.rs           # add 命令
│       ├── delete.rs        # delete 命令
│       ├── edit.rs          # edit 命令
│       └── use_cmd.rs       # use 命令
```

### 数据存储结构

```
~/.ai-providers/
├── state.json              # 全局状态（各 provider 的当前 profile）
├── claude/                  # Claude Code profiles
│   ├── work.json
│   ├── personal.json
│   └── test.json
└── codex/                   # 未来：Codex profiles
    ├── work.json
    └── dev.json
```

### 核心模块设计

#### 1. Provider Trait (`provider/mod.rs`)

定义 provider 的统一接口，各 AI 工具实现此 trait：

```rust
use std::path::PathBuf;
use anyhow::Result;

pub trait Provider {
    /// Provider 标识名，用于 CLI 子命令和存储目录名
    fn name(&self) -> &str;

    /// 该 provider 的配置文件路径（如 ~/.claude/settings.json）
    fn config_path(&self) -> PathBuf;

    /// 验证 JSON 内容是否为合法的配置（可选，默认只检查 JSON 格式）
    fn validate_config(&self, content: &serde_json::Value) -> Result<()> {
        let _ = content;
        Ok(())
    }
}
```

#### 2. ClaudeProvider (`provider/claude.rs`)

Claude Code 的具体实现：

```rust
pub struct ClaudeProvider;

impl Provider for ClaudeProvider {
    fn name(&self) -> &str {
        "claude"
    }

    fn config_path(&self) -> PathBuf {
        // ~/.claude/settings.json
        dirs::home_dir()
            .expect("Cannot determine home directory")
            .join(".claude")
            .join("settings.json")
    }
}
```

#### 3. ProfileManager (`profile/manager.rs`)

通用的 profile 管理逻辑，接收一个 Provider 实例：

```rust
pub struct ProfileManager<'a> {
    provider: &'a dyn Provider,
    profiles_dir: PathBuf,      // ~/.ai-providers/<provider_name>/
    state_file: PathBuf,        // ~/.ai-providers/state.json
}

impl<'a> ProfileManager<'a> {
    pub fn new(provider: &'a dyn Provider) -> Result<Self>;

    // 查询操作
    pub fn list_profiles(&self) -> Result<Vec<String>>;
    pub fn get_current_profile(&self) -> Result<Option<String>>;
    pub fn get_profile(&self, name: &str) -> Result<serde_json::Value>;
    pub fn get_active_config(&self) -> Result<serde_json::Value>;

    // 修改操作
    pub fn add_profile(&self, name: &str, source: ProfileSource) -> Result<()>;
    pub fn delete_profile(&self, name: &str) -> Result<()>;
    pub fn use_profile(&self, name: &str) -> Result<()>;

    // 辅助方法
    pub fn profile_exists(&self, name: &str) -> bool;
    pub fn validate_profile_name(&self, name: &str) -> Result<()>;
    pub fn profile_path(&self, name: &str) -> PathBuf;
}

pub enum ProfileSource {
    Empty,
    FromCurrent,           // 从当前生效的配置文件复制
    FromProfile(String),   // 从已有 profile 复制
}
```

#### 4. Storage (`profile/storage.rs`)

文件 I/O 操作封装：

```rust
/// 读取 JSON 文件
pub fn read_json(path: &Path) -> Result<serde_json::Value>;

/// 原子写入 JSON 文件（临时文件 + rename）
pub fn write_json(path: &Path, value: &serde_json::Value) -> Result<()>;

/// 删除文件
pub fn remove_file(path: &Path) -> Result<()>;

/// 读取全局状态
pub fn read_state(path: &Path) -> Result<State>;

/// 更新全局状态中某个 provider 的当前 profile
pub fn update_state(path: &Path, provider: &str, profile: Option<&str>) -> Result<()>;
```

#### 5. 状态文件格式 (`state.json`)

```json
{
  "claude": {
    "current_profile": "work"
  }
}
```

各 provider 独立追踪，互不影响。未来新增 provider 只需在对应 key 下添加即可。

#### 6. CLI 定义 (`main.rs`)

使用 clap 的嵌套子命令支持 `aip <provider> <command>` 结构：

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aip")]
#[command(about = "AI Providers - Manage AI tool configuration profiles")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: ProviderCommand,
}

#[derive(Subcommand)]
enum ProviderCommand {
    /// Manage Claude Code profiles
    Claude {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    // 未来：
    // Codex { #[command(subcommand)] command: ProfileCommands },
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// List all profiles
    #[command(alias = "ls")]
    List,

    /// Show current active profile
    Current,

    /// Show profile details
    Show {
        /// Profile name
        profile: String,
    },

    /// Show current configuration file content
    Config,

    /// Add a new profile
    Add {
        /// Profile name
        profile: String,
        /// Copy from existing profile
        #[arg(short, long)]
        from: Option<String>,
        /// Create empty profile (default: copy from current config)
        #[arg(short, long)]
        empty: bool,
    },

    /// Delete a profile
    Delete {
        /// Profile name
        profile: String,
        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Edit a profile with $EDITOR
    Edit {
        /// Profile name
        profile: String,
    },

    /// Switch to a profile
    Use {
        /// Profile name
        profile: String,
    },
}
```

## CLI 命令详细设计

所有命令格式为 `aip claude <command> [args]`。

### 1. `aip claude list` / `aip claude ls`

**功能**: 列出所有 Claude Code profiles，标记当前激活的 profile

**输出示例**:
```
Claude Code profiles:
  * work      (current)
    personal
    test
```

**实现要点**:
- 读取 `~/.ai-providers/claude/` 目录下所有 `.json` 文件
- 从 `state.json` 的 `claude.current_profile` 读取当前激活 profile
- 当前 profile 绿色高亮 + 星号标记
- 无 profile 时显示 "No profiles found. Use 'aip claude add <name>' to create one."

### 2. `aip claude current`

**功能**: 显示当前激活的 profile 名称

**输出示例**:
```
Current profile: work
```

**无激活 profile 时**:
```
No profile is currently active
```

### 3. `aip claude show <profile>`

**功能**: 显示指定 profile 的完整配置内容

**输出示例**:
```
Profile: work
{
  "model": "claude-opus-4-6",
  "permissions": {
    "allow": ["Read", "Grep"]
  }
}
```

**错误处理**:
- Profile 不存在 → `Error: Profile 'xxx' not found`

### 4. `aip claude config`

**功能**: 显示当前生效的 Claude Code 配置文件内容（`~/.claude/settings.json`）

**输出示例**:
```
Current Claude Code configuration (~/.claude/settings.json):
{
  "model": "opus[1m]",
  "effortLevel": "high"
}
```

**说明**: 此命令显示的是实际生效的配置文件，可能与任何 profile 不同（用户可能手动修改过）。

**错误处理**:
- 文件不存在 → 黄色警告 "Claude Code configuration file not found"

### 5. `aip claude add <profile> [--from <source>] [--empty]`

**功能**: 创建新的 profile

**使用场景**:
```bash
aip claude add work              # 从当前 settings.json 复制（默认）
aip claude add work --empty      # 创建空 profile ({})
aip claude add work --from dev   # 从已有 profile 复制
```

**实现要点**:
- 验证 profile 名称（禁止：路径分隔符、`.`开头、空名称、`state` 保留名）
- 检查 profile 是否已存在
- 根据参数选择数据源：
  - 默认（无 flag）：读取 `~/.claude/settings.json` 内容
  - `--empty`：创建空 JSON `{}`
  - `--from <name>`：读取 `~/.ai-providers/claude/<name>.json`
- 写入 `~/.ai-providers/claude/<profile>.json`

**错误处理**:
- Profile 已存在 → 报错
- `--from` 源 profile 不存在 → 报错
- 默认模式下 `~/.claude/settings.json` 不存在 → 警告并创建空 profile

### 6. `aip claude delete <profile> [-f]`

**功能**: 删除 profile

**使用场景**:
```bash
aip claude delete work      # 需要确认
aip claude delete work -f   # 强制删除
```

**实现要点**:
- 检查 profile 是否存在
- 如果是当前激活 profile，显示额外警告：
  ```
  Warning: 'work' is the currently active profile.
  Are you sure you want to delete profile 'work'? (y/n):
  ```
- 删除后，如果是当前 profile，清除 `state.json` 中该 provider 的 `current_profile`
- `-f` 跳过所有确认

### 7. `aip claude edit <profile>`

**功能**: 用编辑器编辑 profile 配置文件

**实现要点**:
- 检查 profile 是否存在
- 编辑器选择优先级：`$EDITOR` → `vim` → `vi` → `nano`
- 使用 `std::process::Command` 调用编辑器打开 profile 文件
- 编辑后验证 JSON 格式
- JSON 无效时显示错误详情，询问是否重新编辑

**注意**: edit 只修改 profile 文件（`~/.ai-providers/claude/<profile>.json`），**不会**自动同步到 `~/.claude/settings.json`。即使该 profile 是当前激活的，用户也需要手动 `aip claude use <profile>` 来应用修改。

**错误处理**:
- Profile 不存在 → 报错
- 编辑器未找到 → 报错并提示设置 `$EDITOR`
- JSON 格式错误 → 显示错误详情，询问是否重新编辑

### 8. `aip claude use <profile>`

**功能**: 切换到指定 profile（将 profile 内容写入 settings.json）

**实现要点**:
- 检查 profile 是否存在
- 读取 `~/.ai-providers/claude/<profile>.json`
- **直接覆盖** `~/.claude/settings.json`（不自动保存当前配置）
- 更新 `state.json` 中 `claude.current_profile`

**输出示例**:
```
Switched to profile 'work'
```

**错误处理**:
- Profile 不存在 → 报错
- Profile JSON 格式错误 → 报错并显示详情
- 无法写入目标配置文件 → 报错

## 安全性设计

1. **文件权限**: profile 文件创建时设置为 `0600`（仅所有者可读写），因为可能包含 API 密钥
2. **路径验证**: 验证 profile 名称，拒绝包含 `/`、`\`、`..` 的名称，防止路径遍历
3. **原子写入**: 使用临时文件 + `rename` 的方式确保写入操作的原子性，避免写入中断导致文件损坏
4. **保留名检查**: 拒绝 `state` 作为 profile 名（与 state.json 冲突）

## 错误处理策略

使用 `anyhow::Result` 统一错误处理，所有面向用户的错误信息遵循以下规范：

### 彩色输出规范

| 类型 | 颜色 | 前缀 |
|------|------|------|
| 成功 | 绿色 | ✓ |
| 错误 | 红色 | Error: |
| 警告 | 黄色 | Warning: |
| 提示 | 蓝色 | Tip: |

### 错误信息示例

```
Error: Profile 'work' not found

Available profiles:
  - personal
  - test

Tip: Use 'aip claude add work' to create a new profile
```

## 未来扩展

以下功能在 v1 中不实现，但架构已预留扩展空间：

1. **新增 Provider**: 实现 `Provider` trait 即可支持新的 AI 工具（Codex、Cursor 等）
2. **配置 diff**: `aip claude diff <profile1> <profile2>` 比较两个 profile 差异
3. **导入导出**: `aip claude export/import` 用于分享 profile
4. **配置验证**: 根据 JSON Schema 验证配置是否合法
5. **备份功能**: `aip claude use` 时自动备份被覆盖的配置

## 实现步骤

### Phase 1: 重构基础架构
1. 新增 `provider/` 模块，定义 Provider trait
2. 实现 `ClaudeProvider`
3. 重构 `profile/manager.rs`，接收 Provider 参数
4. 重构存储目录结构（从 `~/.ai-providers/*.json` 到 `~/.ai-providers/claude/*.json`）
5. 重构 `state.json` 格式（从 `{current_profile}` 到 `{claude: {current_profile}}`）

### Phase 2: 重构 CLI
6. 修改 `main.rs`，使用嵌套子命令 `aip claude <cmd>`
7. 更新所有命令实现，使用 Provider 和新的 ProfileManager

### Phase 3: 测试和文档
8. 编写/更新测试
9. 更新 README.md 和 CLAUDE.md

## 验证计划

### 基础流程
```bash
aip claude add work              # 从当前配置创建 work profile
aip claude list                  # 查看所有 profiles
aip claude show work             # 查看 work profile 详情
aip claude config                # 查看当前生效配置
aip claude edit work             # 编辑 work profile
aip claude use work              # 切换到 work profile
aip claude current               # 确认当前 profile
```

### 错误处理
```bash
aip claude show nonexistent      # profile 不存在
aip claude add work              # 重复创建
aip claude delete work           # 删除确认
aip claude delete work -f        # 强制删除
aip claude add "invalid/name"    # 非法 profile 名称
```

### 完整工作流
```bash
aip claude add work --empty
aip claude edit work             # 配置 work 环境
aip claude add personal --empty
aip claude edit personal         # 配置 personal 环境
aip claude use work              # 切换到 work
aip claude config                # 验证配置已切换
aip claude use personal          # 切换到 personal
aip claude config                # 验证配置已切换
aip claude delete work           # 删除（需确认，因为不是当前 profile）
```
