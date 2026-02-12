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
COPY Cargo.toml Cargo.lock ./
COPY agent/Cargo.toml ./agent/
COPY common/Cargo.toml ./common/
COPY controller/Cargo.toml ./controller/

# 创建虚拟源文件以构建依赖
RUN mkdir -p agent/src common/src controller/src && \
    echo "fn main() {}" > agent/src/main.rs && \
    echo "fn main() {}" > controller/src/main.rs && \
    echo "pub fn dummy() {}" > common/src/lib.rs

# 构建依赖（这一层会被缓存）
RUN cargo build --release -p agent && \
    cargo build --release -p controller && \
    rm -rf agent/src common/src controller/src

# 复制实际源码
COPY agent/src ./agent/src
COPY common/src ./common/src
COPY controller/src ./controller/src

# 复制前端构建产物到dist目录（前端构建输出到项目根目录的dist）
COPY --from=web-builder /build/dist ./dist

# 清除本地 crate 的编译缓存，确保使用真实源码重新编译
RUN rm -rf target/release/.fingerprint/common-* \
           target/release/deps/libcommon-* \
           target/release/.fingerprint/agent-* \
           target/release/deps/agent-* \
           target/release/.fingerprint/controller-* \
           target/release/deps/controller-*

# 重新构建（只编译变更的代码）
RUN cargo build --release -p agent && \
    cargo build --release -p controller

# 阶段3: 最终镜像
FROM alpine:latest

# 安装运行时依赖
RUN apk add --no-cache libgcc ca-certificates && \
    addgroup -g 1000 rfrp && \
    adduser -D -u 1000 -G rfrp rfrp

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=rust-builder /build/target/release/agent /app/
COPY --from=rust-builder /build/target/release/controller /app/
COPY --from=rust-builder /build/dist /app/dist

# 复制配置文件模板（使用示例后缀）
COPY controller.toml /app/controller.toml.example

# 创建数据目录并设置权限
RUN mkdir -p /app/data && \
    chown -R rfrp:rfrp /app

# 切换到非特权用户
USER rfrp

# 暴露端口
EXPOSE 7000 3000/tcp

# 默认运行服务端
CMD ["/app/agent", "server"]
