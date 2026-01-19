# RFRP 管理面板

基于 React + Vite 的 RFRP 服务器管理界面。

## 功能特性

- ✅ 客户端管理：创建、查看、删除客户端
- ✅ 代理管理：创建、查看、更新、删除代理规则
- ✅ 实时状态切换：启用/禁用代理
- ✅ Token 自动生成：创建客户端时可自动生成认证 token
- ✅ 现代化 UI：基于 React 19 和现代化 CSS 设计

## 开发模式

1. 安装依赖：
```bash
cd web
bun install
```

2. 启动开发服务器：
```bash
bun run dev
```

访问 http://localhost:5173

## 生产构建

构建到项目根目录的 `dist` 文件夹：
```bash
bun run build
```

## 使用说明

### 1. 启动后端服务器

首先确保 rfrps 正在运行：
```bash
cd rfrps
cargo run
```

服务器将在以下端口启动：
- QUIC 端口：7000
- Web 管理 API：http://localhost:3000

### 2. 启动前端开发服务器

```bash
cd web
bun run dev
```

### 3. 使用管理面板

1. 访问 http://localhost:5173

2. 创建客户端：
   - 点击"添加客户端"
   - 输入客户端名称
   - Token 可留空自动生成
   - 创建后记录 Token（客户端需要使用）

3. 创建代理：
   - 切换到"代理管理"标签
   - 点击"添加代理"
   - 选择所属客户端
   - 配置代理规则：
     - 名称：代理名称
     - 类型：TCP
     - 本地 IP：客户端目标服务 IP
     - 本地端口：客户端目标服务端口
     - 远程端口：服务器监听端口

4. 配置客户端：
   - 将 Token 复制到客户端配置文件 `rfrpc.toml`
   - 启动客户端：`cargo run`

## API 对接

所有 API 请求通过 Vite 代理转发到后端：
- 开发环境：`http://localhost:3000`
- 生产环境：需要配置反向代理

### API 端点

**客户端管理：**
- `GET /api/clients` - 列出所有客户端
- `POST /api/clients` - 创建客户端
- `DELETE /api/clients/:id` - 删除客户端

**代理管理：**
- `GET /api/proxies` - 列出所有代理
- `GET /api/clients/:id/proxies` - 列出指定客户端的代理
- `POST /api/proxies` - 创建代理
- `PUT /api/proxies/:id` - 更新代理
- `DELETE /api/proxies/:id` - 删除代理
