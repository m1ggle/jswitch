# 多发行版支持

本文档介绍 jswitch 的多发行版（multi-distribution）下载支持。

## 支持的发行版

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

## 相关命令速查

```bash
# 查看完整配置
jswitch config show

# 安装时指定发行版
jswitch install 17 --source corretto
jswitch install 11 --source oracle
jswitch install 21 --source openjdk
```
