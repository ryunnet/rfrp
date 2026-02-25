# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

RFRP 是一个基于 Rust 的高性能反向代理工具（内网穿透解决方案），采用三层架构：

- **Controller**：中央控制器，提供 Web 管理界面、RESTful API 和 gRPC 服务
- **Node**：节点服务器，提供 QUIC/KCP 隧道服务，通过 gRPC 连接到 Controller
- **Client**：客户端，通过 gRPC 连接到 Controller，建立到 Node 的隧道连接
- **Dashboard**：React + TypeScript 前端管理界面

## 核心架构

### 通信架构

```
Dashboard (React) ──HTTP/REST──> Controller (Axum)
                                      │
                                      ├──gRPC Stream──> Node (节点服务器)
                                      │                      │
                                      │                      └──QUIC/KCP──> 本地服务
                                      │
                                      └──gRPC Stream──> Client (客户端)
                                                             │
                                                             └──TCP/UDP──> 本地服务
```

### 关键概念

- **Node (节点)**：独立的节点服务器程序，提供 QUIC/KCP 隧道服务，通过 gRPC 连接到 Controller
- **Client (客户端)**：独立的客户端程序，通过 gRPC 连接到 Controller，建立到 Node 的隧道连接
- **Proxy (隧道)**：端口映射配置，定义本地端口到远程端口的转发规则
- **User (用户)**：可以被分配多个节点，管理自己的客户端和隧道
- **流量配额**：Admin 分配给 User，User 分配给 Client 的流量限制系统

### gRPC 双向流

Controller 与 Node/Client 之间使用 gRPC bidirectional streaming：
- **Node**：通过 `AgentServerService` 注册节点并接收代理配置
- **Client**：通过 `AgentClientService` 注册客户端并接收代理配置
- 流管理器：`node_manager.rs` 和 `client_stream_manager.rs` 维护活跃连接

## 常用命令

### 后端开发

```bash
# 构建所有组件
cargo build --release

# 运行 Controller
cargo run --release -p controller
# 或直接运行二进制文件
./target/release/controller

# 运行 Node（节点服务器）
cargo run --release -p node -- --controller-url http://localhost:3100 --token <token> --bind-port 7000
# 或直接运行二进制文件
./target/release/node --controller-url http://localhost:3100 --token <token> --bind-port 7000

# 运行 Client（客户端）
cargo run --release -p client -- --controller-url http://localhost:3100 --token <token>
# 或直接运行二进制文件
./target/release/client --controller-url http://localhost:3100 --token <token>

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 静态分析
cargo clippy --all-targets --all-features -- -D warnings
```

### 前端开发

```bash
cd dashboard

# 安装依赖
bun install

# 开发模式（热重载）
bun run dev

# 构建生产版本（输出到 ../dist）
bun run build

# 代码检查
bun run lint
```

### 数据库迁移

数据库迁移使用 SeaORM migration，位于 `controller/src/migration/`：

```bash
# 迁移会在 Controller 启动时自动运行
# 手动运行迁移（如果需要）
cd controller
cargo run --release
```

创建新迁移：
1. 在 `controller/src/migration/` 创建新文件 `m<YYYYMMDD>_<序号>_<描述>.rs`
2. 实现 `MigrationTrait`
3. 在 `controller/src/migration/mod.rs` 中注册

## 重要文件和模块

### Controller (controller/src/)

- `main.rs` - 启动入口，初始化数据库、gRPC 服务器、Web 服务器
- `grpc_server.rs` - gRPC 服务器实现
- `grpc_agent_server_service.rs` - Node 的 gRPC 服务
- `grpc_agent_client_service.rs` - Client 的 gRPC 服务
- `node_manager.rs` - 节点管理器，维护节点 gRPC 流和状态
- `client_stream_manager.rs` - 客户端流管理器，维护客户端 gRPC 流
- `api/` - RESTful API handlers
- `entity/` - SeaORM 数据库实体
- `migration/` - 数据库迁移
- `traffic.rs` - 流量记录和统计
- `traffic_limiter.rs` - 流量限制和配额验证逻辑

### Node (node/src/)

- `main.rs` - 启动入口，解析命令行参数
- `server/` - 节点服务器实现
  - `proxy_server.rs` - QUIC/KCP 代理服务器
  - `grpc_client.rs` - 连接到 Controller 的 gRPC 客户端
  - `local_proxy_control.rs` - 本地代理控制实现
  - `node_logs.rs` - 节点日志缓冲区（跨平台日志查询）

### Client (client/src/)

- `main.rs` - 启动入口，解析命令行参数
- `client/` - 客户端实现
  - `connector.rs` - 连接管理
  - `grpc_client.rs` - 连接到 Controller 的 gRPC 客户端

### Common (common/src/)

- `proto/rfrp.proto` - gRPC 协议定义
- `grpc/` - 生成的 gRPC 代码
- `protocol/` - 共享协议定义和 traits

### Dashboard (dashboard/src/)

- `lib/services.ts` - API 服务层
- `lib/types.ts` - TypeScript 类型定义
- `pages/` - React 页面组件
  - `Users.tsx` - 用户管理（含流量配额分配）
  - `Clients.tsx` - 客户端管理（含流量配额分配）
  - `Nodes.tsx` - 节点管理
  - `Proxies.tsx` - 隧道管理

## 关键实现细节

### 健康监控系统

Controller 启动时会启动两个健康监控任务（`main.rs:191-254`）：
- `start_node_health_monitor()` - 每 30 秒检查所有节点状态
- `start_client_health_monitor()` - 每 30 秒检查所有客户端状态

这些监控器查询数据库中的所有实体，检查 gRPC 流是否存在，并更新 `is_online` 状态。

### 流量配额系统

流量配额采用三层分配模式：
1. **Admin → User**：管理员为用户分配总流量配额
2. **User → Client**：用户将配额分配给自己的客户端
3. **验证逻辑**：`traffic_limiter.rs` 中的 `check_user_quota_allocation()` 确保分配不超过可用配额

配额模式优先于传统的上传/下载限制。如果设置了 `traffic_quota_gb`，系统会使用配额模式。

### gRPC 流管理

- **NodeManager** (`node_manager.rs`)：维护 `HashMap<node_id, Stream>` 存储节点的 gRPC 流
- **ClientStreamManager** (`client_stream_manager.rs`)：维护 `HashMap<client_id, Stream>` 存储客户端的 gRPC 流
- 流用于实时推送代理配置更新和接收流量统计

### 数据库实体关系

- `User` ↔ `UserNode` ↔ `Node` (多对多)
- `User` → `Client` (一对多)
- `Client` → `Proxy` (一对多)
- `Node` → `Proxy` (一对多，可选)

## 开发注意事项

### 添加新的 API 端点

1. 在 `controller/src/api/handlers/` 创建或修改 handler
2. 在 `controller/src/api/mod.rs` 注册路由
3. 更新 `dashboard/src/lib/services.ts` 添加服务方法
4. 更新 `dashboard/src/lib/types.ts` 添加类型定义（如需要）

### 修改数据库 Schema

1. 创建新的 migration 文件
2. 更新对应的 `entity/` 文件
3. 更新 Rust handler 中的请求/响应结构
4. 更新 TypeScript 类型定义
5. 更新前端组件

### gRPC 协议修改

1. 修改 `common/proto/rfrp.proto`
2. 运行 `cargo build` 自动重新生成代码（通过 `build.rs`）
3. 更新 Controller、Node 和 Client 中的实现

### 流量统计

流量记录在 `controller/src/traffic.rs` 中：
- `record_traffic()` - 记录流量到数据库
- 更新 User、Client、Proxy 的 `total_bytes_sent/received`
- 检查流量限制和配额

## 端口配置

默认端口：
- **3000** - Web 管理界面 (HTTP)
- **3100** - Controller 内部 API (gRPC)
- **7000** - Node 隧道端口 (QUIC/KCP)

## 环境变量

- `RUST_LOG` - 日志级别（默认：info）
- `DATABASE_URL` - SQLite 数据库路径（默认：data/rfrp.db）

## 前端构建输出

前端构建输出到 `dist/` 目录，Controller 通过 `tower-http::ServeDir` 提供静态文件服务。

## 测试策略

- Rust 单元测试：在各模块中使用 `#[cfg(test)]`
- 集成测试：在 `tests/` 目录
- 前端测试：使用 ESLint 进行代码检查

## 常见问题排查

### Controller 启动失败
- 检查端口 3000 和 3100 是否被占用
- 检查数据库文件权限
- 查看日志中的数据库迁移错误

### Node/Client 无法连接到 Controller
- 确认 Controller gRPC 服务运行在 3100 端口
- 检查 token 是否正确
- 查看 Node/Client 日志中的连接错误

### 前端构建失败
- 确保使用 Bun 1.0+
- 检查 TypeScript 类型错误
- 运行 `bun run lint` 查看代码问题

### 流量配额不生效
- 检查 `traffic_quota_gb` 是否已设置
- 查看 `traffic_limiter.rs` 中的验证逻辑
- 确认配额分配没有超过用户总配额
