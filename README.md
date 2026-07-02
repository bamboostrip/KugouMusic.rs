# 酷狗 Web API (Rust)

这是一个使用 Rust 重构的酷狗音乐 Web API 后端服务。

本项目深度参考并致敬了 .NET 项目 [KugouMusic.NET](https://github.com/Linsxyx/KugouMusic.NET.git)。感谢原作者 [Linsxyx](https://github.com/Linsxyx) 及其项目的无私开源与杰出贡献，为本项目的协议实现与核心算法提供了坚实的基础与指引。

---

## 🌟 特性

- **高性能 & 轻量级**：基于 Rust 异步框架 `axum`、`tokio` 与 SQLite 数据库（`sqlx`）。
- **设备身份与会话持久化**：支持自动生成、归一化设备指纹并持久化存储，多端会话安全隔离。
- **全套接口重构**：涵盖登录（手机号验证码/扫码）、歌曲播放链接解析（V5 签名）、云盘歌曲管理、听歌排行与用户详情等功能。
- **Docker 部署**：原生支持 Docker 及 Docker Compose，一键容器化部署。
- **Swagger API 文档**：内置 Swagger UI，方便调试与集成。

---

## 🚀 快速开始

### 方式一：使用 Docker (推荐)

项目已完整配置好 Dockerfile 与 Docker Compose。

1. **启动服务**
   ```bash
   docker compose up --build -d
   ```
2. **查看运行日志**
   ```bash
   docker compose logs -f kugou-api
   ```
3. **访问接口文档 (Swagger UI)**
   打开浏览器访问：`http://localhost:3000/swagger-ui/`

### 方式二：本地运行

1. **环境准备**
   确保本地已安装 Rust 工具链（Edition 2024 / rustc >= 1.96）和 SQLite3。

2. **配置环境变量**
   复制 `.env.example` 为 `.env` 并根据实际需求修改：
   ```bash
   cp .env.example .env
   ```

3. **运行服务**
   ```bash
   cargo run --release
   ```

---

## 🤝 致谢

再次感谢原 .NET 项目的优秀实践：
- **原项目名称**：KugouMusic.NET
- **GitHub 地址**：[https://github.com/Linsxyx/KugouMusic.NET.git](https://github.com/Linsxyx/KugouMusic.NET.git)

---

## 📄 免责声明

本项目仅供学习与学术研究之用。请勿将其用于任何商业目的或非法用途。因使用本项目产生的一切后果，均由使用者自行承担。
