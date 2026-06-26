# Shell 集成原理

本文档说明 `jswitch init` 的工作原理，以及它如何实现会话级环境变量注入和目录级自动切换。

## 概述

`jswitch init <shell>` 输出一段 shell 脚本，用户通过 `eval` 将其嵌入到 shell 配置文件中。该脚本完成两件事：

1. **包装 `jswitch` 命令**——拦截 `jswitch switch --session` 输出的 `export` 语句并在当前 shell 中执行，使环境变量直接生效。
2. **注册 `cd` 钩子**——当用户进入包含 `.java-version` 文件的目录时，自动读取并切换到指定版本。

## 安装方式

在 shell 配置文件中加入一行 `eval`：

```bash
# ~/.bashrc
eval "$(jswitch init bash)"

# ~/.zshrc
eval "$(jswitch init zsh)"
```

```fish
# ~/.config/fish/config.fish
jswitch init fish | source
```

```powershell
# PowerShell Profile
jswitch init powershell | Out-String | Invoke-Expression
```

## 核心机制

### 为什么需要包装函数

`jswitch` 是一个独立的二进制程序，作为子进程运行。子进程**无法修改父进程（shell）的环境变量**——只有 `eval` 在当前 shell 中执行才能做到。

因此 init 脚本定义了一个与二进制同名的 shell 函数，在调用真正的二进制后判断输出内容：

```bash
jswitch() {
    output=$(command jswitch "$@")     # 调用真实二进制
    status=$?
    if [[ "$output" == "export "* ]]; then
        eval "$output"                  # 是 export 语句 → 在当前 shell 执行
    else
        printf '%s\n' "$output"         # 普通输出 → 直接打印
    fi
    return $status
}
```

- `command jswitch` 绕过函数递归，直接调用二进制
- 输出以 `export ` 开头时（即 `--session` 模式），`eval` 使变量在当前 shell 生效
- 其他情况正常打印，退出码透传

### `switch --session` 的输出

当传入 `--session` 参数时，`jswitch switch` 不会写入配置文件，而是输出 export 语句：

```bash
$ jswitch switch 17 --session
export JAVA_HOME="/home/user/.jswitch/versions/17.0.19+10"
export PATH="/home/user/.jswitch/versions/17.0.19+10/bin:$PATH"
```

包装函数检测到 `export ` 前缀后执行 `eval`，从而在当前 shell 进程中设置 `JAVA_HOME` 和 `PATH`。

各 shell 的 export 语法：

| Shell | 语法 |
|-------|------|
| Bash / Zsh | `export VAR="value"` |
| Fish | `set -gx VAR "value"` |
| PowerShell | `$env:VAR = "value"` |

## 自动切换

当用户 `cd` 进入一个包含 `.java-version` 文件的目录时，init 脚本会自动读取该文件并切换 Java 版本。

### `.java-version` 文件

项目根目录下放置一个 `.java-version` 文件，内容为目标版本号：

```text
17
```

或完整的版本号：

```text
17.0.19+10
```

### 各 Shell 的钩子实现

不同 shell 的目录变更钩子机制不同：

**Bash**——通过别名替换 `cd`：

```bash
__jswitch_cd() {
    builtin cd "$@" || return $?
    if [[ "$PWD" != "$__jswitch_prev_dir" ]]; then
        __jswitch_auto_switch
    fi
    __jswitch_prev_dir="$PWD"
}
alias cd='__jswitch_cd'
```

**Zsh**——使用 `chpwd_functions` 钩子（zsh 原生支持）：

```zsh
chpwd_functions+=(__jswitch_auto_switch)
```

**Fish**——监听 `PWD` 变量变化：

```fish
function __jswitch_auto_switch --on-variable PWD
```

**PowerShell**——在 `Prompt` 函数中检查：

```powershell
function Prompt {
    if (Test-Path ".java-version") { ... }
}
```

### 自动切换流程

```text
用户执行 cd /my-project
  → cd 钩子触发
  → 检查当前目录是否有 .java-version
  → 读取版本号（如 "21"）
  → 调用 jswitch switch 21 --session
  → 输出 export JAVA_HOME=... / export PATH=...
  → eval 执行 → 当前 shell 环境更新
```

## 三层作用域

`jswitch` 的版本作用域按优先级从高到低：

| 作用域 | 存储位置 | 生效范围 | 命令 |
|--------|----------|----------|------|
| Session | 当前 shell 进程的 `JAVA_HOME` | 仅当前 shell | `jswitch switch 17 --session` |
| Local | 项目目录 `.java-version` 文件 | 当前项目目录树 | `jswitch switch 17 --local` |
| Global | `~/.jswitch/config.toml` | 全局默认值 | `jswitch switch 17 --global` |

解析顺序：**session > local > global**。

init 脚本的自动切换功能依赖 local 作用域（`.java-version`），通过 session 作用域（`--session`）在当前 shell 中生效。

## 完整调用链

```text
┌─────────────────────────────────────────────────────┐
│  shell 配置文件 (.bashrc / .zshrc / ...)            │
│  eval "$(jswitch init bash)"                        │
│  → 定义 jswitch() 包装函数                           │
│  → 注册 cd 钩子                                     │
└──────────────────────┬──────────────────────────────┘
                       │
          ┌────────────┴────────────┐
          ▼                         ▼
   手动切换                      cd 进入项目目录
   jswitch switch 17              钩子读取 .java-version
   --session                      → jswitch switch <ver> --session
          │                         │
          ▼                         ▼
   二进制输出 export 语句      二进制输出 export 语句
          │                         │
          ▼                         ▼
   包装函数 eval 执行           包装函数 eval 执行
   → JAVA_HOME / PATH 更新     → JAVA_HOME / PATH 更新
```
