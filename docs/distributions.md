# 多发行版支持

本文档介绍 jswitch 的多发行版（multi-distribution）下载支持。

## 支持的发行版

jswitch 支持从以下四个 OpenJDK 发行版源下载安装 Java：

| 发行版 | `--source` 值 | 默认下载来源 | 校验支持 |
|--------|---------------|----------|----------|
| Eclipse Temurin | `adoptopenjdk` | Adoptium API (`api.adoptium.net`) | ✅ 有 checksum |
| Amazon Corretto | `corretto` | AWS Corretto (`corretto.aws`) | ✅ 有 checksum |
| Oracle JDK | `oracle` | Oracle (`download.oracle.com`) | ❌ 无独立 checksum 端点 |
| OpenJDK | `openjdk` | `jdk.java.net`（HTML 抓取） | ✅ 有 checksum |

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
- **OpenJDK**：通过抓取 `jdk.java.net/{major}/`（或 `jdk.java.net/archive/`）的 HTML 页面，解析 `href` 属性获取真实的下载 URL 和 `.sha256` 校验文件 URL。`download.java.net` 的 URL 包含版本特定的 hash/build 段，无法直接拼接，必须从页面中提取。

## 自定义下载源地址

当上游发行版更改 URL 结构时，无需等待 jswitch 发版——通过 `~/.jswitch/config.toml` 中的 `[sources]` 段即可覆盖各发行版的基础 URL。

### 可配置项

| 配置键 | 说明 | 默认值 |
|--------|------|--------|
| `sources.adoptopenjdk` | Adoptium API 基础 URL | `https://api.adoptium.net/v3/assets/feature_releases` |
| `sources.corretto` | Corretto 下载基础 URL | `https://corretto.aws/downloads/latest` |
| `sources.corretto_checksum` | Corretto 校验文件基础 URL | `https://corretto.aws/downloads/latest_sha256` |
| `sources.oracle` | Oracle 下载基础 URL | `https://download.oracle.com/java` |
| `sources.openjdk` | jdk.java.net 页面 URL | `https://jdk.java.net` |

未设置的项会使用内置默认值。

### 使用示例

```bash
# 通过命令行覆盖 Corretto 下载地址
jswitch config set sources.corretto https://my-mirror.example.com/corretto

# 覆盖 OpenJDK 下载地址
jswitch config set sources.openjdk https://my-mirror.example.com/openjdk

# 查看当前配置
jswitch config show
```

也可以直接编辑 `~/.jswitch/config.toml`：

```toml
[sources]
corretto = "https://my-mirror.example.com/corretto"
corretto_checksum = "https://my-mirror.example.com/corretto-checksums"
openjdk = "https://my-mirror.example.com/openjdk"
```

### 注意事项

- 覆盖的是 **基础 URL**，fetcher 会在此之上拼接路径（如 `/{archive_name}` 或 `/jdk{major}/latest/GPL/{archive_name}`）。
- Adoptium 覆盖的是 API 地址，fetcher 仍使用 API 返回的下载链接。
- OpenJDK 覆盖的是 jdk.java.net 页面 URL（默认 `https://jdk.java.net`），fetcher 会抓取 `{url}/{major}/` 和 `{url}/archive/` 页面来发现下载链接。
- 如果覆盖后下载失败，可用 `jswitch config set sources.<key> ""` 重置为默认值（设为空字符串等价于 unset）。

## 相关命令速查

```bash
# 查看完整配置
jswitch config show

# 安装时指定发行版
jswitch install 17 --source corretto
jswitch install 11 --source oracle
jswitch install 21 --source openjdk
```
