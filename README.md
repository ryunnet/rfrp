<div align="center">

# OxiProxy

**基于 Rust 的高性能反向代理工具**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](https://www.rust-lang.org/)
[![QUIC](https://img.shields.io/badge/Protocol-QUIC-blue.svg)](https://quicwg.org/)

一个现代化的 FRP (Fast Reverse Proxy) 实现，采用 Rust + QUIC + Web 技术栈，提供高性能的内网穿透解决方案。

[特性](#-特性) | [快速开始](#-快速开始) | [安装教程](#-安装教程) | [配置说明](#-配置说明) | [Web 管理界面](#-web-管理界面) | [架构](#-架构)

</div>

## ✨ 特性

| 特性 | 说明 |
|------|------|
| **高性能** | 基于 Rust + QUIC 协议，低延迟、高并发 |
| **安全可靠** | TLS 加密传输，Token/JWT 认证机制 |
| **跨平台** | 支持 Linux、Windows、macOS (amd64/arm64) |
| **易于使用** | 简洁配置 + Web 可视化管理界面 |
| **自动重连** | 客户端断线自动重连，服务稳定 |
| **流量监控** | 实时统计客户端和隧道流量 |
| **多用户** | 支持多用户、多客户端、多隧道管理 |

<details>
<summary><b>功能详情</b></summary>

**服务端 (oxiproxy-server)**：QUIC 协议、SQLite 持久化、Web 管理界面、JWT 认证、流量统计、用户权限管理、在线状态监控

**客户端 (oxiproxy-client)**：自动重连、TCP/UDP 代理、多隧道并发、心跳保活

**Web 界面**：仪表盘、客户端管理、隧道管理、流量统计、用户管理、多语言 (中文/English)

</details>

## 🚀 快速开始

### 1. 部署服务端

```bash
# Docker Compose 一键部署（推荐）
mkdir -p /opt/oxiproxy && cd /opt/oxiproxy
curl -O https://raw.githubusercontent.com/oxiproxy/oxiproxy/master/docker-compose.yml
curl -O https://raw.githubusercontent.com/oxiproxy/oxiproxy/master/oxiproxy-server.toml
mkdir -p data && docker-compose up -d

# 查看日志获取 admin 初始密码
docker-compose logs oxiproxy-server
```

### 2. 访问 Web 管理界面

打开 `http://your-server-ip:3000`，使用日志中的密码登录 admin 账号。

### 3. 创建客户端和隧道

1. 进入「客户端管理」→「新建客户端」→ 复制生成的 Token
2. 进入「隧道管理」→「新建隧道」→ 配置端口映射

### 4. 部署客户端

#### Docker 方式（推荐）

```bash
mkdir -p /opt/oxiproxy-client && cd /opt/oxiproxy-client

cat > docker-compose.yml << EOF
version: '3.8'
services:
  oxiproxy-client:
    image: harbor.yunnet.top/oxiproxy:latest
    container_name: oxiproxy-client
    restart: unless-stopped
    command: ["/app/client", "--controller-url", "http://your-server-ip:3100", "--token", "your-client-token"]
EOF

docker-compose up -d
```

#### 原生部署

**Linux/macOS (守护进程模式)**
```bash
# 前台运行
./client --controller-url http://your-server-ip:3100 --token your-client-token

# 守护进程模式
./client --controller-url http://your-server-ip:3100 --token your-client-token --daemon
```

**Windows (服务模式)**
```powershell
# 安装为 Windows 服务（需要管理员权限）
.\client.exe --install-service --controller-url http://your-server-ip:3100 --token your-client-token

# 启动服务
sc start OxiProxyClient

# 停止服务
sc stop OxiProxyClient

# 卸载服务
.\client.exe --uninstall-service
```

### 5. 使用示例

| 场景 | 本地端口 | 远程端口 | 访问方式 |
|------|---------|---------|----------|
| SSH | 22 | 2222 | `ssh -p 2222 user@server-ip` |
| 远程桌面 | 3389 | 33389 | RDP 连接 `server-ip:33389` |
| Web 服务 | 80 | 8080 | 访问 `http://server-ip:8080` |
| MySQL | 3306 | 13306 | 连接 `server-ip:13306` |

## 📦 安装教程

OxiProxy 提供三种安装方式：

| 方式 | 适用场景 | 难度 |
|------|---------|------|
| [Docker Compose](#docker-compose-安装推荐) | 生产环境，推荐 | ⭐ |
| [Docker](#docker-安装) | 熟悉 Docker 的用户 | ⭐⭐ |
| [原生安装](#原生安装) | 自定义编译或无 Docker 环境 | ⭐⭐⭐ |

### Docker Compose 安装（推荐）

<details>
<summary><b>前置要求：安装 Docker</b></summary>

**Linux (Ubuntu/Debian):**
```bash
curl -fsSL https://get.docker.com | sh
sudo systemctl enable --now docker
sudo usermod -aG docker $USER && newgrp docker
```

**Linux (CentOS/RHEL):**
```bash
curl -fsSL https://get.docker.com | sh
sudo systemctl enable --now docker
sudo usermod -aG docker $USER && newgrp docker
```

**Windows/macOS:** 下载安装 [Docker Desktop](https://www.docker.com/products/docker-desktop/)

</details>

#### 部署服务端

```bash
mkdir -p /opt/oxiproxy && cd /opt/oxiproxy

# 下载配置文件
curl -O https://raw.githubusercontent.com/oxiproxy/oxiproxy/master/docker-compose.yml
curl -O https://raw.githubusercontent.com/oxiproxy/oxiproxy/master/oxiproxy-server.toml

mkdir -p data && docker-compose up -d
docker-compose logs oxiproxy-server  # 获取 admin 初始密码
```

> **重要**: 首次启动后查看日志获取 admin 密码，访问 `http://your-server-ip:3000` 登录并修改密码。

<details>
<summary><b>配置防火墙</b></summary>

```bash
# Ubuntu/Debian (ufw)
sudo ufw allow 7000/udp  # QUIC 服务端口
sudo ufw allow 3000/tcp  # Web 界面端口
sudo ufw reload

# CentOS/RHEL (firewalld)
sudo firewall-cmd --permanent --add-port=7000/udp
sudo firewall-cmd --permanent --add-port=3000/tcp
sudo firewall-cmd --reload
```

</details>

<details>
<summary><b>常用命令</b></summary>

```bash
docker-compose up -d          # 启动
docker-compose stop           # 停止
docker-compose restart        # 重启
docker-compose logs -f        # 查看日志
docker-compose pull && docker-compose up -d  # 更新
```

</details>

---

### Docker 安装

<details>
<summary><b>服务端部署</b></summary>

```bash
mkdir -p /opt/oxiproxy/data && cd /opt/oxiproxy
cat > oxiproxy-server.toml << EOF
bind_port = 7000
EOF

docker run -d --name oxiproxy-server --restart unless-stopped \
  -p 7000:7000/udp -p 3000:3000/tcp \
  -v $(pwd)/data:/app/data -v $(pwd)/oxiproxy-server.toml:/app/oxiproxy-server.toml:ro \
  -e TZ=Asia/Shanghai -e RUST_LOG=info \
  harbor.yunnet.top/oxiproxy:latest /app/oxiproxy-server

docker logs -f oxiproxy-server  # 获取 admin 初始密码
```

</details>

<details>
<summary><b>客户端部署</b></summary>

```bash
mkdir -p /opt/oxiproxy-client && cd /opt/oxiproxy-client
cat > oxiproxy-client.toml << EOF
server_addr = "your-server-ip"
server_port = 7000
token = "your-client-token"
EOF

docker run -d --name oxiproxy-client --restart unless-stopped \
  -v $(pwd)/oxiproxy-client.toml:/app/oxiproxy-client.toml:ro \
  -e TZ=Asia/Shanghai -e RUST_LOG=info \
  harbor.yunnet.top/oxiproxy:latest /app/oxiproxy-client
```

</details>

---

### 原生安装

<details>
<summary><b>预编译二进制文件</b></summary>

从 [Releases](https://github.com/oxiproxy/oxiproxy/releases) 下载对应平台的文件：

| 平台 | 下载 |
|------|------|
| Linux amd64 | `oxiproxy-server-linux-amd64.tar.gz` |
| Linux arm64 | `oxiproxy-server-linux-arm64.tar.gz` |
| Windows | `oxiproxy-server-windows-amd64.zip` |
| macOS Intel | `oxiproxy-server-darwin-amd64.tar.gz` |
| macOS Apple Silicon | `oxiproxy-server-darwin-arm64.tar.gz` |

```bash
tar -xzf oxiproxy-server-linux-amd64.tar.gz
chmod +x oxiproxy-server oxiproxy-client
sudo mv oxiproxy-server oxiproxy-client /usr/local/bin/
```

</details>

<details>
<summary><b>从源码编译</b></summary>

**环境要求**: Rust 1.85+, Bun 1.0+, SQLite 3, Git

```bash
git clone https://github.com/oxiproxy/oxiproxy.git && cd oxiproxy
cargo build --release
cd web && bun install && bun run build
# 可执行文件: target/release/oxiproxy-server, target/release/oxiproxy-client
```

</details>

<details>
<summary><b>配置为 systemd 服务 (Linux)</b></summary>

```bash
sudo tee /etc/systemd/system/oxiproxy-server.service > /dev/null << EOF
[Unit]
Description=OxiProxy Server
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/oxiproxy
ExecStart=/usr/local/bin/oxiproxy-server
Restart=always

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now oxiproxy-server
```

</details>

## ⚙️ 配置说明

### 服务端配置 (oxiproxy-server.toml)

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `bind_port` | QUIC 监听端口 | `7000` |

### 客户端配置

客户端通过命令行参数配置：

| 参数 | 说明 | 必需 |
|------|------|------|
| `--controller-url` | Controller 地址（例如 http://server:3100） | 是 |
| `--token` | 客户端认证令牌 | 是 |
| `--daemon` | 守护进程模式（仅 Unix 系统） | 否 |
| `--pid-file` | PID 文件路径（守护进程模式） | 否 |
| `--log-file` | 日志文件路径（守护进程模式） | 否 |
| `--install-service` | 安装为 Windows 服务 | 否 |
| `--uninstall-service` | 卸载 Windows 服务 | 否 |

## 🌐 Web 管理界面

### 功能模块

#### 仪表盘 (Dashboard)
- 总览统计：用户数、客户端数、隧道数
- 流量统计：总发送/接收流量
- 实时状态监控

#### 客户端管理
- 创建/删除客户端
- 生成客户端 Token
- 查看客户端在线状态
- 查看客户端流量统计

#### 隧道管理
- 创建/编辑/删除隧道
- 支持多种隧道类型 (TCP/UDP)
- 配置本地和远程端口
- 查看隧道连接状态

#### 流量统计
- 全局流量概览
- 按用户查看流量详情
- 时间维度流量统计

#### 用户管理 (管理员)
- 创建/编辑/删除用户
- 分配客户端给用户
- 管理用户权限

### API 接口

服务端提供 RESTful API，前缀为 `/api`：

| 端点 | 方法 | 说明 |
|------|------|------|
| `/auth/login` | POST | 用户登录 |
| `/auth/me` | GET | 获取当前用户信息 |
| `/dashboard/stats/{user_id}` | GET | 获取仪表盘统计 |
| `/clients` | GET/POST | 列出/创建客户端 |
| `/clients/{id}` | GET/DELETE | 获取/删除客户端 |
| `/proxies` | GET/POST | 列出/创建隧道 |
| `/proxies/{id}` | PUT/DELETE | 更新/删除隧道 |
| `/traffic/overview` | GET | 流量概览 |
| `/users` | GET/POST | 列出/创建用户 |
| `/users/{id}` | PUT/DELETE | 更新/删除用户 |

## 🏗️ 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         OxiProxy 三层架构                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Dashboard (React) ──HTTP/REST──> Controller (Axum)             │
│                                         │                        │
│                                         ├──gRPC Stream──> Node   │
│                                         │                   │    │
│                                         │                   └──QUIC/KCP──> 本地服务
│                                         │                        │
│                                         └──gRPC Stream──> Client │
│                                                             │    │
│                                                             └──TCP/UDP──> 本地服务
│                                                                 │
│                                    ┌──────────────┐            │
│                                    │  SQLite DB   │            │
│                                    └──────────────┘            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 核心组件

- **Controller**：中央控制器，提供 Web 管理界面、RESTful API 和 gRPC 服务
- **Node**：节点服务器，提供 QUIC/KCP 隧道服务，通过 gRPC 连接到 Controller
- **Client**：客户端，通过 gRPC 连接到 Controller，建立到 Node 的隧道连接
- **Dashboard**：React + TypeScript 前端管理界面

### 技术栈

- **服务端**：
  - Rust 2024 Edition
  - [quinn](https://github.com/quinn-rs/quinn) - QUIC 协议实现
  - [tokio](https://tokio.rs/) - 异步运行时
  - [axum](https://github.com/tokio-rs/axum) - Web 框架
  - [sea-orm](https://www.sea-ql.org/SeaORM/) - ORM 框架

- **客户端**：
  - Rust 2024 Edition
  - [quinn](https://github.com/quinn-rs/quinn) - QUIC 协议实现
  - [tokio](https://tokio.rs/) - 异步运行时

- **Web 界面**：
  - React 19 + TypeScript
  - [Ant Design](https://ant.design/) - UI 组件库
  - [Vite](https://vitejs.dev/) - 构建工具
  - [TailwindCSS](https://tailwindcss.com/) - 样式框架
  - [i18next](https://www.i18next.com/) - 国际化

## 📝 开发

### 环境要求

- Rust 1.85+ (2024 edition)
- Bun 1.0+
- SQLite 3

### 构建项目

```bash
# 克隆仓库
git clone https://github.com/oxiproxy/oxiproxy.git
cd oxiproxy

# 构建所有组件
cargo build --release

# 运行 Controller
cargo run --release -p controller

# 运行 Node（节点服务器）
cargo run --release -p node -- --controller-url http://localhost:3100 --token <token> --bind-port 7000

# 运行 Client（客户端）
cargo run --release -p client -- --controller-url http://localhost:3100 --token <token>

# 开发 Dashboard
cd dashboard
bun install
bun run dev
```

### 运行测试

```bash
# Rust 测试
cargo test

# Web 前端测试
cd web
bun run lint
bun run build
```

### 代码检查

```bash
# 格式化代码
cargo fmt

# Clippy 静态分析
cargo clippy --all-targets --all-features -- -D warnings
```

## 🔄 CI/CD

项目使用 GitHub Actions 进行自动化构建和发布：

- **CI**: 每次提交和 PR 自动运行测试和代码检查
- **Release**: 推送 tag 时自动构建多平台二进制文件并创建 Release

```bash
# 创建新版本发布
git tag v1.0.0
git push origin v1.0.0
```

## 📊 流量统计

OxiProxy 提供详细的流量统计功能：

- **客户端流量**：记录每个客户端的发送/接收字节数
- **隧道流量**：记录每个隧道的流量使用情况
- **用户流量**：按用户聚合统计总流量
- **时间维度**：支持按天、周、月统计流量趋势

## 🔐 安全性

- **TLS 加密**：所有通信使用 QUIC 内置的 TLS 加密
- **Token 认证**：客户端使用 Token 进行身份验证
- **JWT 认证**：Web 界面使用 JWT 进行用户认证
- **密码加密**：用户密码使用 bcrypt 加密存储

## 🔧 故障排除

### 常见问题

**Q: 服务端启动后无法访问 Web 界面？**
- 检查防火墙是否开放 3000 端口
- 检查容器是否正常运行：`docker-compose ps`
- 查看日志排查错误：`docker-compose logs oxiproxy-server`

**Q: 客户端无法连接到 Controller？**
- 确认 Controller 的 gRPC 端口（默认 3100）可访问
- 检查客户端的 controller-url 和 token 是否正确
- 查看客户端日志：`docker-compose logs oxiproxy-client` 或查看守护进程日志
- 确认 Controller 健康状态：访问 `http://server-ip:3000`

**Q: Windows 服务安装失败？**
- 确保以管理员权限运行命令提示符或 PowerShell
- 检查是否已存在同名服务：`sc query OxiProxyClient`
- 查看 Windows 事件查看器中的应用程序日志

**Q: Unix 守护进程无法启动？**
- 检查 PID 文件路径是否有写入权限
- 检查日志文件路径是否有写入权限
- 查看日志文件：`tail -f /var/log/oxiproxy-client.log`

**Q: 忘记 admin 密码怎么办？**
```bash
# 停止服务
docker-compose down

# 删除数据库 (会清空所有数据!)
rm -rf data/oxiproxy.db

# 重新启动，会生成新的 admin 密码
docker-compose up -d
docker-compose logs -f oxiproxy-server
```

**Q: 如何更新到最新版本？**
```bash
# 拉取最新镜像
docker-compose pull

# 重新创建容器
docker-compose up -d

# 查看版本
docker-compose logs oxiproxy-server | grep version
```

**Q: Docker 容器占用空间过大？**
```bash
# 清理未使用的镜像
docker image prune -a

# 清理未使用的卷
docker volume prune

# 清理所有未使用的资源
docker system prune -a
```

**Q: 如何备份数据？**
```bash
# 备份数据库和配置
tar -czf oxiproxy-backup-$(date +%Y%m%d).tar.gz data/ oxiproxy-server.toml

# 恢复数据
tar -xzf oxiproxy-backup-20260125.tar.gz
```

## 📊 性能优化

### 生产环境建议

1. **使用 SSD 存储**：将数据目录挂载到 SSD，提升数据库性能

2. **调整资源限制**：在 docker-compose.yml 中配置合理的 CPU 和内存限制

3. **启用日志轮转**：防止日志文件过大
```yaml
logging:
  driver: "json-file"
  options:
    max-size: "10m"
    max-file: "3"
```

4. **使用反向代理**：为 Web 界面配置 Nginx + HTTPS
```nginx
server {
    listen 443 ssl http2;
    server_name frp.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

5. **定期备份数据**：设置定时任务自动备份
```bash
# 添加到 crontab
0 2 * * * cd /opt/oxiproxy && tar -czf backup/oxiproxy-$(date +\%Y\%m\%d).tar.gz data/
```

## 🗺️ 路线图

- [x] Docker 镜像支持
- [x] Web 管理界面
- [x] 流量统计监控
- [ ] 支持更多隧道类型 (HTTP/HTTPS)
- [ ] 隧道带宽限制
- [ ] 隧道连接数限制
- [ ] Websocket 隧道支持
- [ ] P2P 直连模式
- [ ] 更多平台支持 (FreeBSD, ARM v7)
- [ ] 配置热更新
- [ ] Prometheus 指标导出

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

本项目采用 [MIT](LICENSE) 许可证。

## 🙏 致谢

- [frp](https://github.com/fatedier/frp) - 灵感来源
- [quinn](https://github.com/quinn-rs/quinn) - QUIC 协议实现
- [Tokio](https://tokio.rs/) - 异步运行时

## 📮 联系方式

- 作者: Your Name
- 项目链接: [https://github.com/oxiproxy/oxiproxy](https://github.com/oxiproxy/oxiproxy)

---

<div align="center">

**如果这个项目对你有帮助，请给一个 ⭐️ Star！**

</div>
