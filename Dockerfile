# 使用多阶段构建
# 阶段1: 构建前端
FROM node:20-alpine AS web-builder

WORKDIR /build/dashboard

# 复制前端依赖文件
COPY dashboard/package*.json ./

# 安装依赖
RUN npm install

# 复制前端源码
COPY dashboard/ ./

# 构建前端
RUN npm run build

# 阶段2: 构建Rust后端
FROM rust:1.75-alpine AS rust-builder

# 安装编译依赖
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /build

# 复制整个项目
COPY . .

# 复制前端构建产物到dist目录
COPY --from=web-builder /build/dashboard/dist ./dist

# 构建rfrps
RUN cargo build --release -p rfrps

# 构建rfrpc
RUN cargo build --release -p rfrpc

# 阶段3: 最终镜像
FROM alpine:latest

# 安装运行时依赖
RUN apk add --no-cache libgcc ca-certificates

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=rust-builder /build/target/release/rfrps /app/
COPY --from=rust-builder /build/target/release/rfrpc /app/
COPY --from=rust-builder /build/dist /app/dist

# 复制配置文件模板
COPY rfrps.toml /app/rfrps.toml.example
COPY rfrpc.toml /app/rfrpc.toml.example

# 暴露端口
EXPOSE 7000 3000

# 默认运行服务端
CMD ["/app/rfrps"]
