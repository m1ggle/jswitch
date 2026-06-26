# 下载源配置

本文档介绍 jswitch 的下载源配置。

## 下载来源

jswitch 通过 Adoptium API（`api.adoptium.net`）下载 Eclipse Temurin（AdoptOpenJDK）发行版。

Adoptium API 返回完整的下载链接和 SHA256 校验链接，因此 jswitch 无需自行拼接 URL，也支持自动校验下载文件的完整性。

## 自定义 API 地址

当上游 API 地址变更时，无需等待 jswitch 发版——通过 `~/.jswitch/config.toml` 中的 `[sources]` 段即可覆盖。

### 可配置项

| 配置键 | 说明 | 默认值 |
|--------|------|--------|
| `sources.adoptopenjdk` | Adoptium API 基础 URL | `https://api.adoptium.net/v3/assets/feature_releases` |

未设置时使用内置默认值。

### 使用示例

```bash
# 覆盖 Adoptium API 地址
jswitch config set sources.adoptopenjdk https://my-mirror.example.com/adoptium-api

# 查看当前配置
jswitch config show
```

也可以直接编辑 `~/.jswitch/config.toml`：

```toml
[sources]
adoptopenjdk = "https://my-mirror.example.com/adoptium-api"
```

### 注意事项

- 覆盖的是 API 基础 URL，fetcher 仍使用 API 返回的下载链接和校验链接。
- 如果覆盖后下载失败，可用 `jswitch config set sources.adoptopenjdk ""` 重置为默认值（设为空字符串等价于 unset）。

## 相关命令速查

```bash
# 查看完整配置
jswitch config show

# 安装 Java 版本
jswitch install 17

# 安装并立即切换
jswitch install 17 --use-now
```
