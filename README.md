<div align="center">

# RFRP

**基于 Rust 的高性能反向代理工具**

[![Build](https://github.com/yourusername/rfrp/actions/workflows/build.yml/badge.svg)](https://github.com/yourusername/rfrp/actions/workflows/build.yml)
[![CI](https://github.com/yourusername/rfrp/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/rfrp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个现代化的 FRP (Fast Reverse Proxy) 实现，采用 Rust + QUIC + Web 技术栈，提供高性能的内网穿透解决方案。

[特性](#-特性) • [快速开始](#-快速开始) • [配置说明](#-配置说明) • [Web 管理界面](#-web-管理界面) • [架构](#-架构)

</div>

## ✨ 特性

### 🚀 核心优势

- **高性能**：基于 Rust + QUIC 协议，低延迟、高并发
- **安全可靠**：TLS 加密传输，Token 认证机制
- **跨平台**：支持 Linux、Windows、macOS (amd64/arm64)
- **易于使用**：简洁的配置文件，Web 可视化管理界面
- **自动重连**：客户端断线自动重连，保证服务稳定
- **流量监控**：实时统计客户端和隧道流量数据
- **多用户管理**：支持多用户、多客户端、多隧道管理

### 📦 功能列表

#### 服务端 (rfrps)
- ✅ QUIC 协议支持
- ✅ SQLite 数据持久化
- ✅ Web 管理界面 (React + Ant Design)
- ✅ JWT 身份认证
- ✅ 流量统计与监控
- ✅ 用户权限管理
- ✅ 客户端在线状态监控

#### 客户端 (rfrpc)
- ✅ 自动重连机制
- ✅ TCP/UDP 代理支持
- ✅ 多隧道并发
- ✅ 心跳保活

#### Web 管理界面
- ✅ 仪表盘总览
- ✅ 客户端管理
- ✅ 隧道管理
- ✅ 流量统计
- ✅ 用户管理 (管理员)
- ✅ 多语言支持 (中文/English)

## 📦 安装教程

RFRP 提供三种安装方式，根据您的需求选择：

- **[Docker Compose 安装](#docker-compose-安装推荐)** - 推荐方式，最简单，适合生产环境
- **[Docker 安装](#docker-安装)** - 容器化部署，适合熟悉 Docker 的用户
- **[原生安装](#原生安装)** - 直接运行二进制文件或从源码编译

### Docker Compose 安装（推荐）

这是最简单的部署方式，一条命令即可启动服务，适合生产环境使用。

#### 1. 前置要求：安装 Docker 和 Docker Compose

**Ubuntu/Debian:**
```bash
# 更新包索引
sudo apt update

# 安装依赖
sudo apt install -y apt-transport-https ca-certificates curl gnupg lsb-release

# 添加 Docker 官方 GPG 密钥
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

# 设置稳定版仓库
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# 安装 Docker Engine
sudo apt update
sudo apt install -y docker-ce docker-ce-cli containerd.io

# 启动 Docker 服务
sudo systemctl start docker
sudo systemctl enable docker

# 将当前用户添加到 docker 组 (可选，避免每次使用 sudo)
sudo usermod -aG docker $USER
newgrp docker
```

**CentOS/RHEL:**
```bash
# 安装依赖
sudo yum install -y yum-utils

# 添加 Docker 仓库
sudo yum-config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo

# 安装 Docker Engine
sudo yum install -y docker-ce docker-ce-cli containerd.io

# 启动 Docker 服务
sudo systemctl start docker
sudo systemctl enable docker

# 将当前用户添加到 docker 组 (可选)
sudo usermod -aG docker $USER
newgrp docker
```

**Windows:**
1. 下载并安装 [Docker Desktop for Windows](https://desktop.docker.com/win/main/amd64/Docker%20Desktop%20Installer.exe)
2. 安装完成后重启电脑
3. 启动 Docker Desktop

**macOS:**
1. 下载并安装 [Docker Desktop for Mac](https://desktop.docker.com/mac/main/amd64/Docker.dmg) (Intel) 或 [Apple Silicon](https://desktop.docker.com/mac/main/arm64/Docker.dmg)
2. 启动 Docker Desktop

#### 2. 安装 Docker Compose

> Docker Desktop (Windows/macOS) 已内置 Docker Compose，无需单独安装。

**Linux:**
```bash
# 下载 Docker Compose (v2)
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose

# 添加执行权限
sudo chmod +x /usr/local/bin/docker-compose

# 验证安装
docker-compose --version
```

#### 3. 部署 RFRP 服务端

```bash
# 创建部署目录
mkdir -p /opt/rfrp && cd /opt/rfrp

# 下载配置文件
wget https://raw.githubusercontent.com/yourusername/rfrp/master/docker-compose.yml
wget https://raw.githubusercontent.com/yourusername/rfrp/master/rfrps.toml

# 编辑配置文件 (可选，使用默认配置也可以)
# vim rfrps.toml

# 创建数据目录
mkdir -p data

# 启动服务 (后台运行)
docker-compose up -d

# 查看日志 - 重要: 首次启动会显示 admin 随机密码!
docker-compose logs -f rfrps
```

**首次启动后，请务必：**
1. 在日志中找到 admin 账号的初始密码
2. 访问 `http://your-server-ip:3000` 登录 Web 管理界面
3. 登录后立即修改默认密码
4. 创建客户端并获取 Token

#### 4. 配置防火墙

部署完成后，需要开放以下端口：

**Ubuntu/Debian (ufw):**
```bash
# 开放 QUIC 服务端口 (UDP)
sudo ufw allow 7000/udp

# 开放 Web 管理界面端口 (TCP)
sudo ufw allow 3000/tcp

# 开放代理端口范围 (根据实际需要)
sudo ufw allow 8000:8100/tcp

# 重载防火墙
sudo ufw reload
```

**CentOS/RHEL (firewalld):**
```bash
# 开放 QUIC 服务端口 (UDP)
sudo firewall-cmd --permanent --add-port=7000/udp

# 开放 Web 管理界面端口 (TCP)
sudo firewall-cmd --permanent --add-port=3000/tcp

# 开放代理端口范围
sudo firewall-cmd --permanent --add-port=8000-8100/tcp

# 重载防火墙
sudo firewall-cmd --reload
```

#### 5. 常用 Docker Compose 命令

```bash
# 启动服务 (后台运行)
docker-compose up -d

# 停止服务 (保留数据)
docker-compose stop

# 停止并删除容器 (保留数据卷)
docker-compose down

# 完全删除 (包括数据卷，慎用!)
docker-compose down -v

# 重启服务
docker-compose restart

# 重启特定服务
docker-compose restart rfrps

# 查看服务状态
docker-compose ps

# 查看实时日志
docker-compose logs -f

# 查看特定服务日志
docker-compose logs -f rfrps

# 查看最近 100 行日志
docker-compose logs --tail=100 rfrps

# 更新镜像并重启
docker-compose pull && docker-compose up -d

# 进入容器 (调试用)
docker-compose exec rfrps sh

# 查看资源使用情况
docker stats rfrps
```

#### 6. 部署客户端 (内网机器)

在需要被访问的内网机器上部署客户端：

```bash
# 创建客户端目录
mkdir -p /opt/rfrpc && cd /opt/rfrpc

# 创建客户端配置文件
cat > rfrpc.toml << EOF
server_addr = "your-server-ip"  # 替换为服务端公网 IP
server_port = 7000
token = "your-client-token"      # 从 Web 界面获取
EOF

# 创建 docker-compose 文件
cat > docker-compose.yml << EOF
version: '3.8'
services:
  rfrpc:
    image: harbor.yunnet.top/rfrp:latest
    container_name: rfrpc
    restart: unless-stopped
    volumes:
      - ./rfrpc.toml:/app/rfrpc.toml:ro
    environment:
      - TZ=Asia/Shanghai
      - RUST_LOG=info
    command: ["/app/rfrpc"]
    # 如果需要访问宿主机服务，取消下面的注释
    # extra_hosts:
    #   - "host.docker.internal:host-gateway"
EOF

# 启动客户端
docker-compose up -d

# 查看日志，确认连接成功
docker-compose logs -f
```

---

### Docker 安装

如果您熟悉 Docker，可以直接使用 Docker 命令运行容器，无需 Docker Compose。

#### 服务端部署

```bash
# 创建数据目录
mkdir -p /opt/rfrp/data
cd /opt/rfrp

# 创建配置文件
cat > rfrps.toml << EOF
bind_port = 7000
EOF

# 运行服务端容器
docker run -d \
  --name rfrps \
  --restart unless-stopped \
  -p 7000:7000/udp \
  -p 3000:3000/tcp \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/rfrps.toml:/app/rfrps.toml:ro \
  -e TZ=Asia/Shanghai \
  -e RUST_LOG=info \
  harbor.yunnet.top/rfrp:latest \
  /app/rfrps

# 查看日志，获取 admin 初始密码
docker logs -f rfrps
```

**开放防火墙端口：**
```bash
# Ubuntu/Debian
sudo ufw allow 7000/udp
sudo ufw allow 3000/tcp

# CentOS/RHEL
sudo firewall-cmd --permanent --add-port=7000/udp
sudo firewall-cmd --permanent --add-port=3000/tcp
sudo firewall-cmd --reload
```

#### 客户端部署

```bash
# 创建客户端目录
mkdir -p /opt/rfrpc
cd /opt/rfrpc

# 创建配置文件
cat > rfrpc.toml << EOF
server_addr = "your-server-ip"
server_port = 7000
token = "your-client-token"
EOF

# 运行客户端容器
docker run -d \
  --name rfrpc \
  --restart unless-stopped \
  -v $(pwd)/rfrpc.toml:/app/rfrpc.toml:ro \
  -e TZ=Asia/Shanghai \
  -e RUST_LOG=info \
  harbor.yunnet.top/rfrp:latest \
  /app/rfrpc

# 查看日志
docker logs -f rfrpc
```

**常用 Docker 命令：**
```bash
# 停止容器
docker stop rfrps

# 启动容器
docker start rfrps

# 重启容器
docker restart rfrps

# 查看日志
docker logs -f rfrps

# 查看容器状态
docker ps -a

# 更新镜像
docker pull harbor.yunnet.top/rfrp:latest
docker stop rfrps && docker rm rfrps
# 然后重新运行 docker run 命令

# 进入容器
docker exec -it rfrps sh

# 删除容器
docker stop rfrps && docker rm rfrps
```

---

### 原生安装

适合不想使用 Docker 或需要自定义编译的用户。

#### 方式一：使用预编译二进制文件

从 [Releases](https://github.com/yourusername/rfrp/releases) 页面下载对应平台的二进制文件。

**Linux (amd64):**
```bash
# 下载并解压
wget https://github.com/yourusername/rfrp/releases/latest/download/rfrps-linux-amd64.tar.gz
tar -xzf rfrps-linux-amd64.tar.gz

# 赋予执行权限
chmod +x rfrps rfrpc

# 移动到系统路径 (可选)
sudo mv rfrps rfrpc /usr/local/bin/
```

**Linux (arm64):**
```bash
wget https://github.com/yourusername/rfrp/releases/latest/download/rfrps-linux-arm64.tar.gz
tar -xzf rfrps-linux-arm64.tar.gz
chmod +x rfrps rfrpc
sudo mv rfrps rfrpc /usr/local/bin/
```

**Windows:**
```powershell
# 下载 ZIP 文件
# https://github.com/yourusername/rfrp/releases/latest/download/rfrps-windows-amd64.zip

# 解压后双击运行 rfrps.exe 或 rfrpc.exe
# 或在 PowerShell/CMD 中运行
.\rfrps.exe
```

**macOS (Intel):**
```bash
wget https://github.com/yourusername/rfrp/releases/latest/download/rfrps-darwin-amd64.tar.gz
tar -xzf rfrps-darwin-amd64.tar.gz
chmod +x rfrps rfrpc
sudo mv rfrps rfrpc /usr/local/bin/
```

**macOS (Apple Silicon):**
```bash
wget https://github.com/yourusername/rfrp/releases/latest/download/rfrps-darwin-arm64.tar.gz
tar -xzf rfrps-darwin-arm64.tar.gz
chmod +x rfrps rfrpc
sudo mv rfrps rfrpc /usr/local/bin/
```

#### 方式二：从源码编译

**环境要求：**
- Rust 1.85+ (2024 edition)
- Bun 1.0+ (用于构建 Web 界面)
- SQLite 3
- Git

**步骤：**

```bash
# 1. 克隆仓库
git clone https://github.com/yourusername/rfrp.git
cd rfrp

# 2. 编译服务端和客户端
cargo build --release

# 3. 编译 Web 界面
cd web
bun install
bun run build
cd ..

# 4. 可执行文件位于 target/release/ 目录
# rfrps - 服务端
# rfrpc - 客户端
```

#### 配置和启动

**1. 启动服务端：**

```bash
# 创建配置文件
cat > rfrps.toml << EOF
bind_port = 7000
EOF

# 启动服务端
./target/release/rfrps
# 或从系统路径启动
rfrps

# Windows
rfrps.exe
```

**首次启动注意事项：**
- 服务端会自动创建 admin 用户
- **请务必查看日志中的初始密码！**
- Web 界面地址：`http://localhost:3000`
- 默认用户名：`admin`

**2. 启动客户端：**

```bash
# 创建配置文件
cat > rfrpc.toml << EOF
server_addr = "your-server-ip"
server_port = 7000
token = "your-client-token"  # 从 Web 界面获取
EOF

# 启动客户端
./target/release/rfrpc
# 或
rfrpc

# Windows
rfrpc.exe
```

#### 配置为系统服务（Linux）

**使用 systemd 管理服务端：**

```bash
# 创建服务文件
sudo tee /etc/systemd/system/rfrps.service > /dev/null << EOF
[Unit]
Description=RFRP Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/rfrp
ExecStart=/usr/local/bin/rfrps
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# 启动并设置开机自启
sudo systemctl daemon-reload
sudo systemctl enable rfrps
sudo systemctl start rfrps

# 查看状态
sudo systemctl status rfrps

# 查看日志
sudo journalctl -u rfrps -f
```

**使用 systemd 管理客户端：**

```bash
# 创建服务文件
sudo tee /etc/systemd/system/rfrpc.service > /dev/null << EOF
[Unit]
Description=RFRP Client
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/rfrpc
ExecStart=/usr/local/bin/rfrpc
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# 启动并设置开机自启
sudo systemctl daemon-reload
sudo systemctl enable rfrpc
sudo systemctl start rfrpc

# 查看状态
sudo systemctl status rfrpc
```

---

## 🚀 快速开始

安装完成后，按照以下步骤快速开始使用：

### 1. 访问 Web 管理界面

打开浏览器访问：`http://your-server-ip:3000`

- 用户名：`admin`
- 密码：查看服务端首次启动日志

### 2. 修改默认密码

登录后立即修改 admin 密码：
1. 点击右上角用户头像
2. 选择"修改密码"
3. 输入新密码并保存

### 3. 创建客户端

1. 进入"客户端管理"页面
2. 点击"新建客户端"
3. 填写客户端名称和描述
4. 点击"保存"，复制生成的 Token

### 4. 创建隧道

1. 进入"隧道管理"页面
2. 点击"新建隧道"
3. 配置隧道参数：
   - **隧道名称**：自定义名称
   - **隧道类型**：TCP/UDP
   - **远程端口**：外网访问端口
   - **本地地址**：内网服务地址（如 127.0.0.1）
   - **本地端口**：内网服务端口
4. 点击"保存"

### 5. 使用示例

假设您想通过公网访问内网的 SSH 服务（22 端口）：

**隧道配置：**
- 隧道类型：TCP
- 远程端口：2222（公网访问端口）
- 本地地址：127.0.0.1
- 本地端口：22

**访问方式：**
```bash
ssh -p 2222 user@your-server-ip
```

**更多使用场景：**
- **远程桌面**：将内网 RDP (3389) 映射到公网
- **Web 服务**：将内网 HTTP (80/443) 映射到公网
- **数据库**：访问内网 MySQL (3306) / PostgreSQL (5432)
- **游戏服务器**：映射游戏端口供外网玩家连接

## ⚙️ 配置说明

### 服务端配置 (rfrps.toml)

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `bind_port` | QUIC 监听端口 | `7000` |

### 客户端配置 (rfrpc.toml)

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `server_addr` | 服务器地址 | - |
| `server_port` | 服务器端口 | `7000` |
| `token` | 客户端认证令牌 | - |

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
┌─────────────────────────────────────────────────────────────┐
│                         RFRP 架构                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌──────────────┐            ┌──────────────┐             │
│   │   rfrpc      │            │   rfrps      │             │
│   │   (客户端)   │◄───QUIC───►│  (服务端)    │             │
│   │              │   加密通信   │              │             │
│   └──────┬───────┘            └──────┬───────┘             │
│          │                           │                      │
│          │ TCP/UDP                   │                      │
│          ▼                           ▼                      │
│   ┌──────────────┐            ┌──────────────┐             │
│   │  本地服务    │            │  Web 界面    │             │
│   │              │            │  (React)     │             │
│   └──────────────┘            └──────────────┘             │
│                                          │                  │
│                                          ▼                  │
│                                  ┌──────────────┐          │
│                                  │  SQLite DB   │          │
│                                  └──────────────┘          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

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
git clone https://github.com/yourusername/rfrp.git
cd rfrp

# 构建所有组件
cargo build --release

# 构建并运行服务端
cargo run --release -p rfrps

# 构建并运行客户端
cargo run --release -p rfrpc

# 开发 Web 界面
cd web
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

RFRP 提供详细的流量统计功能：

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
- 查看日志排查错误：`docker-compose logs rfrps`

**Q: 客户端无法连接到服务端？**
- 确认服务端防火墙开放 7000/udp 端口
- 检查客户端配置中的 server_addr 和 token 是否正确
- 查看客户端日志：`docker-compose logs rfrpc`
- 确认服务端健康状态：访问 `http://server-ip:3000`

**Q: 忘记 admin 密码怎么办？**
```bash
# 停止服务
docker-compose down

# 删除数据库 (会清空所有数据!)
rm -rf data/rfrp.db

# 重新启动，会生成新的 admin 密码
docker-compose up -d
docker-compose logs -f rfrps
```

**Q: 如何更新到最新版本？**
```bash
# 拉取最新镜像
docker-compose pull

# 重新创建容器
docker-compose up -d

# 查看版本
docker-compose logs rfrps | grep version
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
tar -czf rfrp-backup-$(date +%Y%m%d).tar.gz data/ rfrps.toml

# 恢复数据
tar -xzf rfrp-backup-20260125.tar.gz
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
0 2 * * * cd /opt/rfrp && tar -czf backup/rfrp-$(date +\%Y\%m\%d).tar.gz data/
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
- 项目链接: [https://github.com/yourusername/rfrp](https://github.com/yourusername/rfrp)

---

<div align="center">

**如果这个项目对你有帮助，请给一个 ⭐️ Star！**

</div>
