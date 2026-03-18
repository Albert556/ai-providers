# AI Providers CLI 工具架构设计文档

## 项目背景

这是一个 Rust CLI 工具，用于管理 Claude Code 的配置文件。用户在不同的开发场景下需要使用不同的 Claude Code 配置（例如不同的 API 密钥、模型选择、权限设置等），手动编辑配置文件既繁琐又容易出错。本工具提供了一个简单的命令行界面来管理多个配置 profile，实现快速切换。

## 核心需求

1. 管理多个 Claude Code 配置 profile
2. 支持查看、添加、删除、编辑、切换 profile
3. 简单直观的命令行界面
4. 安全的配置文件操作

## 技术选型

### 依赖库

| 库名 | 版本 | 用途 |
|------|------|------|
| `clap` | 4.x | CLI 参数解析，支持子命令和参数验证 |
| `serde` | 1.x | 序列化/反序列化框架 |
| `serde_json` | 1.x | JSON 格式支持 |
| `anyhow` | 1.x | 错误处理，提供丰富的错误上下文 |
| `colored` | 2.x | 终端彩色输出 |

### 为什么选择这些库？

- **clap**: Rust 生态中最成熟的 CLI 解析库，支持子命令、自动生成帮助信息、参数验证
- **serde + serde_json**: 标准的 JSON 序列化方案，性能优秀，API 友好
- **anyhow**: 简化错误处理，适合应用层代码（非库代码）
- **colored**: 提供彩色输出，提升用户体验

## 架构设计

### 目录结构

```
ai-providers/
├── Cargo.toml
├── CLAUDE.md
├── README.md
├── src/
│   ├── main.rs           # 程序入口，CLI 定义
│   ├── commands/         # 命令实现
│   │   ├── mod.rs
│   │   ├── list.rs       # list 命令
│   │   ├── current.rs    # current 命令
│   │   ├── show.rs       # show 命令
│   │   ├── config.rs     # config 命令
│   │   ├── add.rs        # add 命令
│   │   ├── delete.rs     # delete 命令
│   │   ├── edit.rs       # edit 命令
│   │   └── use_cmd.rs    # use 命令
│   ├── profile/          # Profile 管理核心逻辑
│   │   ├── mod.rs
│   │   ├── manager.rs    # ProfileManager 结构体
│   │   └── storage.rs    # 文件存储操作
│   ├── config/           # 配置相关
│   │   ├── mod.rs
│   │   └── paths.rs      # 路径管理
│   └── error.rs          # 错误类型定义（可选）
```

### 核心模块设计

#### 1. CLI 定义 (main.rs)

使用 clap 的 derive API 定义命令行接口：

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aip")]
#[command(about = "AI Providers - Manage Claude Code configuration profiles")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

    /// Show current Claude Code configuration
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

    /// Edit a profile
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

#### 2. Profile Manager (profile/manager.rs)

核心业务逻辑，负责 profile 的 CRUD 操作：

```rust
pub struct ProfileManager {
    profiles_dir: PathBuf,
    claude_config_path: PathBuf,
    state_file: PathBuf,
}

impl ProfileManager {
    pub fn new() -> Result<Self>;

    // 查询操作
    pub fn list_profiles(&self) -> Result<Vec<String>>;
    pub fn get_current_profile(&self) -> Result<Option<String>>;
    pub fn get_profile(&self, name: &str) -> Result<serde_json::Value>;
    pub fn get_claude_config(&self) -> Result<serde_json::Value>;

    // 修改操作
    pub fn add_profile(&self, name: &str, source: ProfileSource) -> Result<()>;
    pub fn delete_profile(&self, name: &str) -> Result<()>;
    pub fn use_profile(&self, name: &str) -> Result<()>;

    // 辅助方法
    pub fn profile_exists(&self, name: &str) -> bool;
    pub fn validate_profile_name(&self, name: &str) -> Result<()>;
}

pub enum ProfileSource {
    Empty,
    FromCurrent,
    FromProfile(String),
}
```

#### 3. 路径管理 (config/paths.rs)

统一管理所有文件路径：

```rust
pub struct Paths {
    pub profiles_dir: PathBuf,      // ~/.ai-providers/
    pub state_file: PathBuf,         // ~/.ai-providers/state.json
    pub claude_config: PathBuf,      // ~/.claude/settings.json
}

impl Paths {
    pub fn new() -> Result<Self>;
    pub fn ensure_dirs(&self) -> Result<()>;
    pub fn profile_path(&self, name: &str) -> PathBuf;
}
```

#### 4. 状态管理

使用一个简单的 JSON 文件记录当前激活的 profile：

```json
{
  "current_profile": "work"
}
```

存储位置：`~/.ai-providers/state.json`

## CLI 命令详细设计

### 1. `aip list` / `aip ls`

**功能**: 列出所有 profile，并标记当前激活的 profile

**输出示例**:
```
Available profiles:
  * work      (current)
    personal
    test
```

**实现要点**:
- 读取 `~/.ai-providers/` 目录下所有 `.json` 文件
- 从 `state.json` 读取当前激活的 profile
- 使用彩色输出标记当前 profile（绿色 + 星号）

### 2. `aip current`

**功能**: 显示当前激活的 profile 名称

**输出示例**:
```
Current profile: work
```

**实现要点**:
- 读取 `state.json` 中的 `current_profile` 字段
- 如果没有激活的 profile，显示 "No profile is currently active"

### 3. `aip show <profile>`

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

**实现要点**:
- 读取 `~/.ai-providers/<profile>.json`
- 使用 `serde_json` 格式化输出（pretty print）
- 如果 profile 不存在，报错

### 4. `aip config`

**功能**: 显示当前 Claude Code 的配置内容

**输出示例**:
```
Current Claude Code configuration:
{
  "model": "claude-sonnet-4-6",
  "permissions": {
    "allow": ["*"]
  }
}
```

**实现要点**:
- 读取 `~/.claude/settings.json`
- 如果文件不存在，显示警告信息

### 5. `aip add <profile> [--from <source>] [--empty]`

**功能**: 添加新的 profile

**使用场景**:
- `aip add work` - 从当前 Claude Code 配置复制创建（默认行为）
- `aip add work --empty` - 创建空的 profile
- `aip add work --from personal` - 从现有 profile 复制

**实现要点**:
- 验证 profile 名称（不能包含特殊字符、路径分隔符）
- 检查 profile 是否已存在
- 根据参数选择数据源
- 创建 `~/.ai-providers/<profile>.json`

**错误处理**:
- Profile 已存在 → 报错
- 源 profile 不存在 → 报错
- Claude Code 配置不存在且未指定 --empty → 警告并创建空 profile

### 6. `aip delete <profile> [-f]`

**功能**: 删除指定 profile

**使用场景**:
- `aip delete work` - 删除前要求确认
- `aip delete work -f` - 强制删除，不确认

**实现要点**:
- 检查 profile 是否存在
- 如果是当前激活的 profile，警告用户
- 默认要求确认（y/n）
- 使用 `-f` 标志跳过确认
- 删除 `~/.ai-providers/<profile>.json`

**确认提示**:
```
Are you sure you want to delete profile 'work'? (y/n):
```

### 7. `aip edit <profile>`

**功能**: 使用编辑器编辑 profile 配置

**实现要点**:
- 检查 profile 是否存在
- 读取 `$EDITOR` 环境变量（fallback: `vim` → `vi` → `nano`）
- 使用 `std::process::Command` 调用编辑器
- 编辑后验证 JSON 格式是否正确
- 如果 JSON 无效，询问用户是否重新编辑

**错误处理**:
- Profile 不存在 → 报错
- 编辑器未找到 → 报错并提示设置 $EDITOR
- JSON 格式错误 → 显示错误详情，询问是否重新编辑

### 8. `aip use <profile>`

**功能**: 切换到指定 profile

**实现要点**:
- 检查 profile 是否存在
- 读取 `~/.ai-providers/<profile>.json`
- 覆盖写入 `~/.claude/settings.json`
- 更新 `state.json` 中的 `current_profile`

**输出示例**:
```
Switched to profile 'work'
```

**错误处理**:
- Profile 不存在 → 报错
- Profile JSON 格式错误 → 报错并显示详情
- 无法写入 Claude Code 配置 → 报错

## 文件存储设计

### Profile 存储格式

每个 profile 是一个独立的 JSON 文件：

```
~/.ai-providers/
├── state.json          # 状态文件
├── work.json           # work profile
├── personal.json       # personal profile
└── test.json           # test profile
```

### Profile 文件内容

直接存储 Claude Code 的 `settings.json` 内容：

```json
{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
  "model": "claude-opus-4-6",
  "permissions": {
    "allow": ["Read", "Grep", "Glob"],
    "ask": ["Edit", "Write"],
    "deny": ["Bash"]
  },
  "env": {
    "API_KEY": "sk-xxx"
  }
}
```

### 状态文件 (state.json)

```json
{
  "current_profile": "work",
  "last_updated": "2026-03-18T10:30:00Z"
}
```

## 错误处理策略

### 错误类型

使用 `anyhow::Result` 作为统一的返回类型，在需要时添加上下文信息：

```rust
use anyhow::{Context, Result};

pub fn read_profile(name: &str) -> Result<serde_json::Value> {
    let path = get_profile_path(name);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read profile '{}'", name))?;

    let json = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in profile '{}'", name))?;

    Ok(json)
}
```

### 用户友好的错误信息

所有错误信息应该：
1. 清晰说明发生了什么
2. 提供可能的解决方案
3. 使用彩色输出（红色表示错误）

示例：
```
Error: Profile 'work' not found

Available profiles:
  - personal
  - test

Tip: Use 'aip add work' to create a new profile
```

## 输出格式设计

### 彩色输出规范

- **成功信息**: 绿色
- **错误信息**: 红色
- **警告信息**: 黄色
- **提示信息**: 蓝色
- **当前激活项**: 绿色 + 粗体

### 示例输出

```rust
use colored::*;

println!("{}", "✓ Profile created successfully".green());
println!("{}", "✗ Profile not found".red());
println!("{}", "⚠ Claude Code configuration not found".yellow());
println!("{}", "ℹ Tip: Use 'aip list' to see all profiles".blue());
```

## 安全性考虑

1. **文件权限**: Profile 文件可能包含敏感信息（API 密钥），创建时设置为 `0600`（仅所有者可读写）
2. **路径验证**: 验证 profile 名称，防止路径遍历攻击
3. **原子操作**: 使用临时文件 + 重命名的方式确保写入操作的原子性
4. **备份**: 在覆盖 Claude Code 配置前，可选地创建备份（未来功能）

## 未来扩展功能

以下功能在初始版本中不实现，但在架构设计中预留扩展空间：

1. **交互式创建**: `aip add --interactive` - 通过问答式界面创建 profile
2. **Profile 模板**: 提供常用配置模板
3. **配置验证**: 验证 profile 配置是否符合 Claude Code schema
4. **配置合并**: 支持部分配置覆盖而非完全替换
5. **多文件支持**: 同时管理 `settings.json` 和 `.claude.json`
6. **备份功能**: 自动备份被覆盖的配置
7. **导入导出**: 支持导入导出 profile 用于分享
8. **配置 diff**: 比较两个 profile 的差异

## 实现步骤

### Phase 1: 基础框架
1. 更新 `Cargo.toml`，添加依赖
2. 实现 `config/paths.rs` - 路径管理
3. 实现 `profile/storage.rs` - 基础文件操作
4. 实现 `profile/manager.rs` - ProfileManager 核心逻辑

### Phase 2: 命令实现
5. 实现 `commands/list.rs` - list 命令
6. 实现 `commands/current.rs` - current 命令
7. 实现 `commands/show.rs` - show 命令
8. 实现 `commands/config.rs` - config 命令
9. 实现 `commands/add.rs` - add 命令
10. 实现 `commands/delete.rs` - delete 命令
11. 实现 `commands/edit.rs` - edit 命令
12. 实现 `commands/use_cmd.rs` - use 命令

### Phase 3: 集成和优化
13. 更新 `main.rs` - 集成所有命令
14. 添加错误处理和用户友好的错误信息
15. 添加彩色输出
16. 编写测试

### Phase 4: 文档和发布
17. 更新 README.md
18. 编写使用文档
19. 测试完整工作流
20. 发布第一个版本

## 关键文件清单

实现过程中需要创建/修改的文件：

- `Cargo.toml` - 添加依赖
- `src/main.rs` - CLI 定义和程序入口
- `src/config/mod.rs` - 配置模块导出
- `src/config/paths.rs` - 路径管理
- `src/profile/mod.rs` - Profile 模块导出
- `src/profile/manager.rs` - ProfileManager 核心逻辑
- `src/profile/storage.rs` - 文件存储操作
- `src/commands/mod.rs` - 命令模块导出
- `src/commands/list.rs` - list 命令
- `src/commands/current.rs` - current 命令
- `src/commands/show.rs` - show 命令
- `src/commands/config.rs` - config 命令
- `src/commands/add.rs` - add 命令
- `src/commands/delete.rs` - delete 命令
- `src/commands/edit.rs` - edit 命令
- `src/commands/use_cmd.rs` - use 命令

## 验证计划

实现完成后，通过以下场景验证功能：

1. **基础流程**:
   ```bash
   aip add work              # 从当前配置创建 work profile
   aip list                  # 查看所有 profiles
   aip show work             # 查看 work profile 详情
   aip edit work             # 编辑 work profile
   aip use work              # 切换到 work profile
   aip current               # 确认当前 profile
   ```

2. **错误处理**:
   ```bash
   aip show nonexistent      # 测试 profile 不存在
   aip add work              # 测试重复创建
   aip delete work           # 测试删除确认
   aip delete work -f        # 测试强制删除
   ```

3. **边界情况**:
   ```bash
   aip add "invalid/name"    # 测试非法 profile 名称
   aip use work              # 测试切换到不存在的 profile
   # 手动破坏 JSON 格式，测试错误处理
   ```

4. **完整工作流**:
   ```bash
   # 场景：在 work 和 personal 配置间切换
   aip add work --empty
   aip edit work             # 配置 work 环境
   aip add personal --empty
   aip edit personal         # 配置 personal 环境
   aip use work              # 切换到 work
   aip config                # 验证配置已切换
   aip use personal          # 切换到 personal
   aip config                # 验证配置已切换
   ```
