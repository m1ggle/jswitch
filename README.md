# JSwitch - 极速 Java 版本切换器

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/yourusername/jswitch/ci.yml)](https://github.com/yourusername/jswitch/actions)
[![Crates.io](https://img.shields.io/crates/v/jswitch.svg)](https://crates.io/crates/jswitch)

> 一个用 Rust 编写的快速、安全的 Java 版本管理工具，灵感来自 `nvm`、`pyenv` 和 `jenv`，但更快、更现代。

## ✨ 特性

- 🚀 **极速启动**：Rust 编译的二进制文件，启动速度超快
- 🔒 **内存安全**：Rust 的所有权系统确保无内存错误
- 🎯 **精准控制**：全局、项目、会话三级版本管理
- 📦 **自动安装**：通过 Adoptium API 自动下载和安装 Eclipse Temurin (AdoptOpenJDK) 版本
- 🔄 **智能切换**：自动更新 `JAVA_HOME` 和 `PATH` 环境变量
- 🛡️ **安全可靠**：下载校验、版本验证、回滚机制
- 🎨 **美观输出**：彩色终端输出、进度条、emoji 图标
- 📱 **跨平台**：支持 macOS、Linux、Windows
- 🔧 **可扩展**：插件系统、配置同步、团队协作

## 📦 安装

### 使用 Cargo（推荐）

```bash
# 从 crates.io 安装
cargo install jswitch

# 或从 GitHub 安装最新版
cargo install --git https://github.com/yourusername/jswitch
```

### 预编译二进制

从 [Releases 页面](https://github.com/yourusername/jswitch/releases) 下载对应平台的二进制文件：

```bash
# Linux/macOS
curl -fsSL https://github.com/yourusername/jswitch/releases/latest/download/jswitch-x86_64-unknown-linux-gnu.tar.gz | tar -xz
sudo mv jswitch /usr/local/bin/

# macOS (Homebrew)
brew install yourusername/tap/jswitch

# Windows (PowerShell)
# 下载 jswitch.exe 并添加到 PATH
```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/jswitch.git
cd jswitch

# 构建并安装
cargo build --release
sudo cp target/release/jswitch /usr/local/bin/
```

## 🚀 快速开始

### 1. 初始化 shell

```bash
# 根据你的 shell 初始化
jswitch init bash    # 对于 bash
jswitch init zsh     # 对于 zsh
jswitch init fish    # 对于 fish
jswitch init powershell  # 对于 PowerShell

# 重新加载 shell 配置
source ~/.bashrc  # 或 source ~/.zshrc 等
```

### 2. 安装 Java 版本

```bash
# 安装最新的 Java LTS 版本
jswitch install lts

# 安装特定版本
jswitch install 17
jswitch install 11.0.2
jswitch install 21.0.1
```

### 3. 切换 Java 版本

```bash
# 查看已安装的版本
jswitch list --installed

# 切换到全局默认版本
jswitch switch 17 --global

# 仅为当前项目设置版本（创建 .java-version 文件）
cd my-project
jswitch switch 11 --local

# 在当前会话中临时切换
jswitch switch 21 --session

# 使用别名
jswitch switch lts     # 切换到最新的 LTS 版本
jswitch switch latest  # 切换到最新版本
```

### 4. 验证安装

```bash
# 查看当前使用的版本
jswitch current

# 验证 Java 环境
java -version
javac -version

# 运行医生检查
jswitch doctor
```

### 5. 体验与稳定性

```bash
# 启用调试日志
JSWITCH_DEBUG=true jswitch current
RUST_LOG=jswitch=debug jswitch install 17

# 静默安装（不显示进度条）
JSWITCH_QUIET=true jswitch install 17

# 生成 shell 补全脚本
jswitch completion zsh > ~/.zfunc/_jswitch
```

更多诊断、日志、进度条和补全说明见 [docs/stability.md](docs/stability.md) 与 [docs/completion.md](docs/completion.md)。

## 📖 详细使用指南

### 版本管理

```bash
# 列出所有可用版本
jswitch list --remote

# 列出已安装版本
jswitch list --installed

# 显示详细信息
jswitch list --verbose

# 安装并立即使用
jswitch install 17 --use-now

# 移除版本
jswitch remove 11
jswitch remove 1.8.0 --force  # 强制移除当前使用的版本
```

### 配置管理

```bash
# 显示当前配置
jswitch config show

# 设置自动更新
jswitch config set auto_update true

# 设置默认版本别名
jswitch config set alias.lts "17"
jswitch config set alias.latest "21"

# 获取特定配置
jswitch config get global.default_version

# 列出所有配置
jswitch config list
```

### 别名系统

```bash
# 使用预定义的别名
jswitch install lts      # 安装最新的 LTS 版本
jswitch switch latest    # 切换到最新版本
jswitch switch stable    # 切换到稳定版本

# 自定义别名
jswitch alias set myproject 11.0.2
jswitch switch myproject

jswitch alias list
jswitch alias remove myproject
```

### 多项目管理

```bash
# 项目 A 使用 Java 11
cd project-a
jswitch switch 11 --local
# 这会创建 .java-version 文件

# 项目 B 使用 Java 17
cd ../project-b
jswitch switch 17 --local

# 回到项目 A，自动切换到 Java 11
cd ../project-a
jswitch current  # 显示 11
```

### 高级功能

```bash
# 导出环境配置
jswitch export --format=json > java_env.json

# 导入环境配置
jswitch import java_env.json

# 清理缓存和临时文件
jswitch cleanup

# 显示磁盘使用情况
jswitch disk-usage

# 更新 jswitch 自身
jswitch self-update
```

## ⚙️ 配置

### 配置文件位置

- **全局配置**: `~/.jswitch/config.toml`
- **项目配置**: `./.java-version` 或 `./.jswitch.toml`
- **Shell 配置**: 自动添加到 `~/.bashrc`、`~/.zshrc` 等

### 配置示例

```toml
# ~/.jswitch/config.toml
[global]
default_version = "17"
auto_update = true
check_updates = true
quiet_mode = false

[aliases]
lts = "17"
latest = "21"
stable = "11"
myproject = "11.0.2"

[plugins]
# 插件配置
maven = true
gradle = true
```

### 环境变量

```bash
# 调试模式
export JSWITCH_DEBUG=true
export RUST_LOG=jswitch=debug

# 静默模式
export JSWITCH_QUIET=true
```

## 🏗️ 项目结构

```
jswitch/
├── Cargo.toml              # Rust 项目配置
├── Cargo.lock              # 依赖锁文件
├── src/
│   ├── main.rs             # 主入口点
│   ├── cli/                # CLI 解析和验证
│   ├── commands/           # 所有命令实现
│   │   ├── mod.rs
│   │   ├── switch.rs       # switch 命令
│   │   ├── install.rs      # install 命令
│   │   ├── list.rs         # list 命令
│   │   ├── remove.rs       # remove 命令
│   │   ├── current.rs      # current 命令
│   │   ├── config.rs       # config 命令
│   │   ├── init.rs         # init 命令
│   │   ├── doctor.rs       # doctor 命令
│   │   ├── alias.rs        # alias 命令
│   │   └── self_update.rs  # self-update 命令
│   ├── config/             # 配置管理
│   │   ├── mod.rs
│   │   ├── manager.rs      # 配置管理器
│   │   ├── global.rs       # 全局配置
│   │   ├── local.rs        # 本地配置
│   │   └── validator.rs    # 配置验证
│   ├── version/            # 版本管理核心
│   │   ├── mod.rs
│   │   ├── manager.rs      # 版本管理器
│   │   ├── resolver.rs     # 版本解析器
│   │   ├── java_version.rs # JavaVersion 结构体
│   │   └── metadata.rs     # 版本元数据
│   ├── download/           # 下载和安装
│   │   ├── mod.rs
│   │   ├── fetcher.rs      # 版本获取器
│   │   ├── installer.rs    # 安装器
│   │   ├── verifier.rs     # 校验器
│   │   └── progress.rs     # 下载进度
│   ├── env/                # 环境管理
│   │   ├── mod.rs
│   │   ├── variable.rs     # 环境变量管理
│   │   ├── shell.rs        # Shell 集成
│   │   └── hook.rs         # Shell 钩子
│   ├── fs/                 # 文件系统操作
│   │   ├── mod.rs
│   │   ├── operations.rs   # 文件操作
│   │   ├── cache.rs        # 缓存管理
│   │   └── lockfile.rs     # 锁文件
│   ├── error/              # 错误处理
│   │   ├── mod.rs
│   │   ├── jswitch_error.rs # 主错误类型
│   │   └── display.rs      # 错误显示
│   ├── ui/                 # 用户界面
│   │   ├── mod.rs
│   │   ├── terminal.rs     # 终端交互
│   │   ├── table.rs        # 表格显示
│   │   └── spinner.rs      # 加载动画
│   └── utils/              # 工具函数
│       ├── mod.rs
│       ├── logging.rs      # 日志工具
│       ├── validation.rs   # 验证工具
│       └── macros.rs       # 自定义宏
├── tests/                  # 集成测试
│   ├── integration/
│   ├── unit/
│   └── helpers.rs
├── docs/                   # 文档
│   ├── api.md
│   ├── contributing.md
│   └── design.md
├── scripts/                # 构建脚本
│   ├── install.sh
│   ├── release.sh
│   └── cross-compile.sh
├── .github/                # GitHub 配置
│   ├── workflows/
│   │   ├── ci.yml
│   │   ├── release.yml
│   │   └── test.yml
│   └── ISSUE_TEMPLATE/
├── completions/            # Shell 自动完成
│   ├── bash/
│   ├── zsh/
│   ├── fish/
│   └── powershell/
└── examples/               # 使用示例
    ├── basic-usage/
    ├── multi-project/
    └── ci-cd/
```

## 🧪 开发指南

### 环境设置

```bash
# 1. 安装 Rust（如果尚未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 克隆仓库
git clone https://github.com/yourusername/jswitch.git
cd jswitch

# 3. 安装开发依赖
rustup component add clippy rustfmt
cargo install cargo-watch cargo-audit cargo-tarpaulin

# 4. 安装预提交钩子
cargo install pre-commit
pre-commit install
```

### 构建和测试

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test

# 运行特定测试
cargo test test_switch_command

# 代码格式检查
cargo fmt --check

# 代码质量检查
cargo clippy --all-features -- -D warnings

# 安全审计
cargo audit

# 代码覆盖率
cargo tarpaulin --ignore-tests

# 基准测试
cargo bench
```

### 添加新命令

1. 在 `src/commands/` 中创建新文件，例如 `new_command.rs`
2. 实现 `Command` trait
3. 在 `src/commands/mod.rs` 中导出
4. 在 `src/cli/mod.rs` 中添加 CLI 定义
5. 添加测试用例

### 调试技巧

```bash
# 启用调试日志
RUST_LOG=jswitch=debug cargo run -- switch 17

# 使用调试器
rust-gdb --args target/debug/jswitch switch 17

# 性能分析
cargo flamegraph --bin jswitch -- switch 17
```

## 🔧 架构设计

### 核心组件

1. **CLI 层** (`src/cli/`) — 命令行接口解析和验证
2. **命令层** (`src/commands/`) — 具体命令实现
3. **版本管理层** (`src/version/`) — Java 版本管理核心逻辑
4. **下载层** (`src/download/`) — 网络下载和安装
5. **环境层** (`src/env/`) — 环境变量和 Shell 集成
6. **配置层** (`src/config/`) — 配置管理
7. **UI 层** (`src/ui/`) — 用户界面和输出格式化

### 数据流

```
用户输入 → CLI解析 → 命令分发 → 业务逻辑 → 结果输出
    ↓          ↓          ↓          ↓          ↓
终端输入   参数验证   命令路由   版本管理   格式化显示
                     ↓          ↓          ↓
                   配置读取   文件操作   进度显示
                     ↓          ↓          ↓
                   网络请求   环境更新   错误处理
```

### 并发模型

- **I/O 密集型操作**：使用 `tokio` 异步运行时
- **CPU 密集型操作**：使用 `rayon` 并行处理
- **文件操作**：使用 `async-std` 或 `tokio::fs`
- **进度显示**：使用 `indicatif` 和通道通信

## 🧪 测试策略

### 单元测试

```rust
// src/version/java_version.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let version = JavaVersion::parse("11.0.2").unwrap();
        assert_eq!(version.major, 11);
        assert_eq!(version.minor, Some(0));
        assert_eq!(version.patch, Some(2));
    }
}
```

### 集成测试

```rust
// tests/integration/switch_command.rs
#[test]
fn test_switch_command() {
    let mut cmd = Command::cargo_bin("jswitch").unwrap();
    cmd.arg("switch").arg("17").arg("--global");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("切换到 Java 17"));
}
```

### 端到端测试

```bash
#!/bin/bash
# tests/e2e/test_basic_workflow.sh

# 测试完整的工作流程
jswitch install 17
jswitch switch 17 --global
jswitch current | grep "17"
java -version 2>&1 | grep "17"
```

## 📚 API 文档

生成和查看 API 文档：

```bash
# 生成文档
cargo doc --no-deps --open

# 生成并查看
cargo doc --open

# 生成 Markdown 文档
cargo readme > README.md
```

## 🤝 贡献指南

### 开发流程

1. Fork 仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 代码规范

- 遵循 [Rust API 指南](https://rust-lang.github.io/api-guidelines/)
- 使用 `rustfmt` 格式化代码
- 使用 `clippy` 进行代码检查
- 编写详细的文档注释
- 添加适当的测试用例

### 提交信息规范

使用 [Conventional Commits](https://www.conventionalcommits.org/)：

```
feat: 添加新功能
fix: 修复 bug
docs: 文档更新
style: 代码格式调整
refactor: 代码重构
test: 测试相关
chore: 构建过程或辅助工具
```

## 📄 许可证

本项目采用双重许可证：

- **MIT 许可证** — 查看 [LICENSE-MIT](LICENSE-MIT) 文件
- **Apache 2.0 许可证** — 查看 [LICENSE-APACHE](LICENSE-APACHE) 文件

你可以根据需求选择任一许可证。

## 🙏 致谢

- 感谢所有 [贡献者](https://github.com/yourusername/jswitch/graphs/contributors)
- 灵感来自 [nvm](https://github.com/nvm-sh/nvm)、[pyenv](https://github.com/pyenv/pyenv)、[jenv](https://github.com/jenv/jenv)
- 使用 [clap](https://github.com/clap-rs/clap) 构建 CLI
- 使用 [reqwest](https://github.com/seanmonstar/reqwest) 进行网络请求
- 使用 [indicatif](https://github.com/console-rs/indicatif) 显示进度

## 📞 支持

- 📖 [查看文档](https://github.com/yourusername/jswitch/wiki)
- 🐛 [报告问题](https://github.com/yourusername/jswitch/issues)
- 💬 [讨论区](https://github.com/yourusername/jswitch/discussions)
- 📧 邮箱：your.email@example.com

## 🚀 路线图

### v0.1.0 (MVP)
- [x] 基本版本切换功能
- [x] 本地版本管理
- [x] 全局/本地/会话三级作用域
- [x] 基础 CLI 界面

### v0.2.0
- [x] 自动下载安装
- [x] Adoptium API 集成
- [ ] Shell 自动完成

### v0.3.0
- [ ] 插件系统
- [ ] GUI 界面
- [ ] 云同步配置
- [ ] 性能优化

### v1.0.0
- [ ] 稳定 API
- [ ] 完整文档
- [ ] 企业级功能
- [ ] 社区插件市场

---

**Happy Switching!** 🎯

*用 Rust 编写的 Java 版本切换器*
