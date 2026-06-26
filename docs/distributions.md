# 多发行版支持与镜像源配置

本文档介绍 jswitch 的多发行版（multi-distribution）下载支持和镜像源（mirror）配置机制。

## 多发行版支持

### 支持的发行版

jswitch 支持从以下四个 OpenJDK 发行版源下载安装 Java：

| 发行版 | `--source` 值 | 下载来源 | 校验支持 |
|--------|---------------|----------|----------|
| Eclipse Temurin | `adoptopenjdk` | Adoptium API (`api.adoptium.net`) | ✅ 有 checksum |
| Amazon Corretto | `corretto` | AWS Corretto (`corretto.aws`) | ✅ 有 checksum |
| Oracle JDK | `oracle` | Oracle (`download.oracle.com`) | ❌ 无独立 checksum 端点 |
| OpenJDK | `openjdk` | `download.java.net` | ❌ 无独立 checksum 端点 |

### 安装时指定发行版

```bash
# 默认使用 Adoptium（Eclipse Temurin）
jswitch install 17

# 从特定发行版安装
jswitch install 17 --source corretto
jswitch install 11 --source oracle
jswitch install 21 --source openjdk
jswitch install 17 --source adoptopenjdk

# 安装并立即切换到该版本
jswitch install 17 --source corretto --use-now
```

### 下载流程

各发行版的下载策略不同：

- **Adoptium**：通过 Adoptium API 查询最新 GA 版本的资产信息，获取下载链接和 checksum 链接。
- **Corretto**：直接拼接下载 URL 和 checksum URL，无需 API 调用。
- **Oracle**：直接拼接下载 URL，使用 `download.oracle.com/java/{major}/latest/` 路径。Oracle 不提供独立 checksum 端点，因此跳过校验。
- **OpenJDK**：直接拼接下载 URL，使用 `download.java.net/java/GA/jdk/{major}/latest/` 路径。同样跳过校验。

## 镜像源配置

jswitch 支持三种镜像源配置方式，可以加速在中国大陆等地区的下载速度。

### 方式一：全局镜像

设置 `global.mirror_url` 后，所有发行版的下载 URL 都会替换镜像地址的 base（保留路径和查询参数）。

```bash
# 设置全局镜像源
jswitch config set global.mirror_url "https://mirrors.tuna.tsinghua.edu.cn"

# 查看当前配置
jswitch config get global.mirror_url

# 取消设置
jswitch config set global.mirror_url ""
```

URL 替换示例：

```
原始:  https://api.adoptium.net/v3/assets/feature_releases/17/ga?...
镜像后: https://mirrors.tuna.tsinghua.edu.cn/v3/assets/feature_releases/17/ga?...

原始:  https://corretto.aws/downloads/latest/amazon-corretto-17-x64-linux-jdk.tar.gz
镜像后: https://mirrors.tuna.tsinghua.edu.cn/downloads/latest/amazon-corretto-17-x64-linux-jdk.tar.gz
```

### 方式二：按发行版单独配置

为每个发行版指定独立的镜像源，**优先级高于全局镜像**。

```bash
# 仅为 Corretto 设置镜像
jswitch config set sources.corretto "https://corretto-mirror.example.com"

# 仅为 Adoptium 设置镜像
jswitch config set sources.adoptopenjdk "https://adoptium-mirror.example.com"

# 仅为 Oracle 设置镜像
jswitch config set sources.oracle "https://oracle-mirror.example.com"

# 仅为 OpenJDK 设置镜像
jswitch config set sources.openjdk "https://openjdk-mirror.example.com"
```

例如同时设置了全局镜像和 `sources.corretto`：

```bash
jswitch config set global.mirror_url "https://global-mirror.com"
jswitch config set sources.corretto "https://corretto-mirror.com"
```

- 安装 Corretto 时使用 `https://corretto-mirror.com`
- 安装 Adoptium 时使用 `https://global-mirror.com`

### 方式三：环境变量（临时覆盖）

通过环境变量临时指定镜像源，**优先级高于 `config.toml` 中的 `global.mirror_url`**。

```bash
# 临时使用镜像安装
JSWITCH_MIRROR_URL="https://mirror.example.com" jswitch install 17 --source corretto
```

### 优先级总结

```
环境变量 JSWITCH_MIRROR_URL
        ↓
按发行版镜像 (sources.*)
        ↓
全局镜像 (global.mirror_url)
        ↓
原始上游 URL（不替换）
```

## 代理配置

如果需要通过代理下载 Java，可以配置 HTTP/HTTPS 代理：

### 方式一：配置文件

```bash
jswitch config set proxy.http_proxy "http://proxy.company.com:8080"
jswitch config set proxy.https_proxy "http://proxy.company.com:8080"
jswitch config set proxy.no_proxy "localhost,127.0.0.1"
```

### 方式二：环境变量（优先于配置文件）

```bash
export JSWITCH_HTTP_PROXY="http://proxy:8080"
export JSWITCH_HTTPS_PROXY="http://proxy:8080"
```

### 代理优先级

```
环境变量 JSWITCH_HTTP_PROXY / JSWITCH_HTTPS_PROXY
        ↓
配置文件 [proxy] http_proxy / https_proxy
        ↓
不使用代理
```

## 完整配置示例

`~/.jswitch/config.toml`：

```toml
[global]
default_version = "17"
mirror_url = "https://mirrors.tuna.tsinghua.edu.cn"
auto_update = true
check_updates = true
quiet_mode = false

[aliases]
lts = "17"
latest = "21"
stable = "11"

[sources]
openjdk = "https://openjdk-mirror.example.com"
corretto = "https://corretto-mirror.example.com"
adoptopenjdk = "https://adoptium-mirror.example.com"
oracle = "https://oracle-mirror.example.com"

[proxy]
http_proxy = "http://proxy.company.com:8080"
https_proxy = "http://proxy.company.com:8080"
no_proxy = "localhost,127.0.0.1"

[plugins]
maven = true
gradle = true
```

## 相关命令速查

```bash
# 查看完整配置
jswitch config show

# 查看特定配置项
jswitch config get global.mirror_url
jswitch config get sources.corretto

# 设置配置项
jswitch config set global.mirror_url "https://mirror.example.com"
jswitch config set sources.corretto "https://corretto-mirror.example.com"

# 列出所有配置
jswitch config list

# 安装时指定发行版
jswitch install 17 --source corretto
```
