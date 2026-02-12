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
FROM rust:alpine AS rust-builder

# 安装编译依赖
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /build

# 先复制依赖文件，利用Docker缓存层
COPY Cargo.toml ./
COPY rfrps/Cargo.toml ./rfrps/
COPY rfrpc/Cargo.toml ./rfrpc/
COPY rfrp-common/Cargo.toml ./rfrp-common/

# 创建虚拟源文件以构建依赖
RUN mkdir -p rfrps/src rfrpc/src rfrp-common/src && \
    echo "fn main() {}" > rfrps/src/main.rs && \
    echo "fn main() {}" > rfrpc/src/main.rs && \
    echo "pub fn dummy() {}" > rfrp-common/src/lib.rs

# 构建依赖（这一层会被缓存）
RUN cargo build --release -p rfrps && \
    cargo build --release -p rfrpc && \
    rm -rf rfrps/src rfrpc/src rfrp-common/src

# 复制实际源码
COPY rfrps/src ./rfrps/src
COPY rfrpc/src ./rfrpc/src
COPY rfrp-common/src ./rfrp-common/src

# 复制前端构建产物到dist目录
COPY --from=web-builder /build/dashboard/dist ./dist

# 重新构建（只编译变更的代码）
RUN cargo build --release -p rfrps && \
    cargo build --release -p rfrpc

# 阶段3: 最终镜像
FROM alpine:latest

# 安装运行时依赖
RUN apk add --no-cache libgcc ca-certificates && \
    addgroup -g 1000 rfrp && \
    adduser -D -u 1000 -G rfrp rfrp

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=rust-builder /build/target/release/rfrps /app/
COPY --from=rust-builder /build/target/release/rfrpc /app/
COPY --from=rust-builder /build/dist /app/dist

# 复制配置文件模板（使用示例后缀）
COPY rfrps.toml /app/rfrps.toml.example
COPY rfrpc.toml /app/rfrpc.toml.example

# 创建数据目录并设置权限
RUN mkdir -p /app/data && \
    chown -R rfrp:rfrp /app

# 切换到非特权用户
USER rfrp

# 暴露端口
EXPOSE 7000/udp 3000/tcp

# 默认运行服务端
CMD ["/app/rfrps"]
