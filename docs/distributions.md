# 多发行版支持与镜像源配置

本文档介绍 jswitch 的多发行版（multi-distribution）下载支持和镜像源（mirror）配置机制。

## 多发行版支持

### 支持的发行版

jswitch 支持从以下四个 OpenJDK 发行版源下载安装 Java：

| 发行版 | `--source` 值 | 默认下载来源 | 校验支持 |
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

jswitch 支持两种镜像源配置方式，可以加速在中国大陆等地区的下载速度。

### 方式一：按发行版配置（sources.\*）

为每个发行版指定独立的下载 base URL。配置后，jswitch 会使用该 URL 作为该发行版的完整下载基址，**直接拼接归档文件名**，不会与上游 URL 做主机替换。

```bash
# 为 Corretto 设置自定义下载源
jswitch config set sources.corretto "https://my-mirror.com/corretto"

# 为 Adoptium 设置自定义 API 地址
jswitch config set sources.adoptopenjdk "https://my-mirror.com/adoptium/api"

# 为 Oracle 设置自定义下载源
jswitch config set sources.oracle "https://my-mirror.com/oracle"

# 为 OpenJDK 设置自定义下载源
jswitch config set sources.openjdk "https://my-mirror.com/openjdk"
```

URL 构造示例（以 Corretto 为例）：

```
sources.corretto = "https://my-mirror.com/corretto"

下载 URL: https://my-mirror.com/corretto/amazon-corretto-17-x64-macos-jdk.tar.gz
```

> **注意**：当使用 `sources.*` 自定义源时，jswitch 无法推断镜像站的 checksum URL 结构，因此会跳过校验和验证。如需校验，请使用全局镜像方式。

### 方式二：全局镜像（global.mirror_url）

设置 `global.mirror_url` 后，所有发行版的下载 URL 都会替换镜像地址的主机部分（保留路径和查询参数）。

```bash
# 设置全局镜像源
jswitch config set global.mirror_url "https://mirrors.tuna.tsinghua.edu.cn"

# 查看当前配置
jswitch config get global.mirror_url
```

URL 替换示例：

```
原始:  https://api.adoptium.net/v3/assets/feature_releases/17/ga?...
镜像后: https://mirrors.tuna.tsinghua.edu.cn/v3/assets/feature_releases/17/ga?...

原始:  https://corretto.aws/downloads/latest/amazon-corretto-17-x64-linux-jdk.tar.gz
镜像后: https://mirrors.tuna.tsinghua.edu.cn/downloads/latest/amazon-corretto-17-x64-linux-jdk.tar.gz
```

全局镜像会同时替换 checksum URL 的主机，因此校验和验证仍然可用。

### 方式三：环境变量（临时覆盖）

通过环境变量临时指定全局镜像，**优先级高于 `config.toml` 中的 `global.mirror_url`**。

```bash
# 临时使用镜像安装
JSWITCH_MIRROR_URL="https://mirror.example.com" jswitch install 17 --source corretto
```

### 优先级总结

```
环境变量 JSWITCH_MIRROR_URL
        ↓
全局镜像 (global.mirror_url)  —— 主机替换，保留路径
        ↓
按发行版自定义源 (sources.*)   —— 完整 base URL 替换
        ↓
原始上游 URL（不替换）
```

> **`sources.*` 与 `global.mirror_url` 的区别**：
> - `sources.*`：**完整 base URL 替换**。直接用配置的 URL 作为下载基址，拼接归档文件名。不与上游 URL 做主机替换，因此不会产生路径重复问题。
> - `global.mirror_url`：**主机替换**。将上游 URL 的 `scheme://host[:port]` 替换为镜像地址，保留原始路径。

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

# 按发行版自定义下载源（完整 base URL 替换）
# 设置后该发行版直接从指定 URL 下载，不走全局镜像
[sources]
corretto = "https://my-mirror.com/corretto"
adoptopenjdk = "https://my-mirror.com/adoptium"

[proxy]
http_proxy = "http://proxy.company.com:8080"
https_proxy = "http://proxy.company.com:8080"
no_proxy = "localhost,127.0.0.1"

[plugins]
maven = true
gradle = true
```

## 常见问题

### 下载 URL 路径重复

如果配置 `sources.corretto` 后出现 URL 路径重复（如 `/downloads/resources/downloads/latest/...`），请检查配置值是否为上游 URL 而非镜像 base URL。`sources.*` 应该是**完整的下载 base URL**，jswitch 会直接在其后拼接归档文件名。

**错误配置**（值为上游路径，会导致路径重复）：
```toml
[sources]
corretto = "https://corretto.aws/downloads/resources"
```

**正确配置**（值为镜像 base URL，jswitch 会拼接 `/archive_name`）：
```toml
[sources]
corretto = "https://my-mirror.com/corretto"
```

### DNS 解析失败

如果出现 `failed to lookup address information` 错误，请检查：
1. 代理配置是否正确（`proxy.http_proxy` / `proxy.https_proxy` 不能使用占位符）
2. 网络是否能访问目标域名
3. 是否需要配置镜像源加速访问

## 相关命令速查

```bash
# 查看完整配置
jswitch config show

# 查看特定配置项
jswitch config get global.mirror_url
jswitch config get sources.corretto

# 设置配置项
jswitch config set global.mirror_url "https://mirror.example.com"
jswitch config set sources.corretto "https://my-mirror.com/corretto"

# 列出所有配置
jswitch config list

# 安装时指定发行版
jswitch install 17 --source corretto
```
