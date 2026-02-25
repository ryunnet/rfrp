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
RUN apk add --no-cache musl-dev openssl-dev pkgconfig protobuf-dev

WORKDIR /build

# 先复制依赖文件，利用Docker缓存层
COPY Cargo.toml Cargo.lock ./
COPY node/Cargo.toml ./node/
COPY client/Cargo.toml ./client/
COPY common/Cargo.toml ./common/
COPY controller/Cargo.toml ./controller/

# 复制 build.rs 和 proto 文件（protobuf 代码生成需要）
COPY common/build.rs ./common/
COPY common/proto ./common/proto

# 创建虚拟源文件以构建依赖
RUN mkdir -p node/src client/src common/src controller/src && \
    echo "fn main() {}" > node/src/main.rs && \
    echo "fn main() {}" > client/src/main.rs && \
    echo "fn main() {}" > controller/src/main.rs && \
    echo "pub fn dummy() {}" > common/src/lib.rs

# 构建依赖（这一层会被缓存）
RUN cargo build --release -p node && \
    cargo build --release -p client && \
    cargo build --release -p controller && \
    rm -rf node/src client/src common/src controller/src

# 复制实际源码
COPY node/src ./node/src
COPY client/src ./client/src
COPY common/src ./common/src
COPY controller/src ./controller/src

# 复制前端构建产物到dist目录（前端构建输出到项目根目录的dist）
COPY --from=web-builder /build/dist ./dist

# 清除本地 crate 的编译缓存，确保使用真实源码重新编译
RUN rm -rf target/release/.fingerprint/common-* \
           target/release/deps/libcommon-* \
           target/release/.fingerprint/node-* \
           target/release/deps/node-* \
           target/release/.fingerprint/client-* \
           target/release/deps/client-* \
           target/release/.fingerprint/controller-* \
           target/release/deps/controller-*

# 重新构建（只编译变更的代码）
RUN cargo build --release -p node && \
    cargo build --release -p client && \
    cargo build --release -p controller

# 阶段3: 最终镜像
FROM alpine:latest

# 安装运行时依赖
RUN apk add --no-cache libgcc ca-certificates && \
    addgroup -g 1000 rfrp && \
    adduser -D -u 1000 -G rfrp rfrp

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=rust-builder /build/target/release/node /app/
COPY --from=rust-builder /build/target/release/client /app/
COPY --from=rust-builder /build/target/release/controller /app/
COPY --from=rust-builder /build/dist /app/dist

# 创建数据目录并设置权限
RUN mkdir -p /app/data && \
    chown -R rfrp:rfrp /app

# 切换到非特权用户
USER rfrp

# 暴露端口
EXPOSE 7000 3000 3100

# 默认运行 controller
# 可以通过 docker run 时指定不同的命令来运行 node 或 client
CMD ["/app/controller"]
