# syntax=docker/dockerfile:1.7

# ===== Stage 1: builder =====
# rustc 1.96 + edition 2024：必须用 1.96 及以上的 image。
# 选 bookworm（不是 alpine）：ring 的预编译 assembly + utoipa-swagger-ui build.rs
# 在 glibc 环境最省心；alpine 要 musl，ring 需要额外构建。
FROM rust:1.96-bookworm AS builder

WORKDIR /build

# 1) 先只拷依赖清单，预编译依赖层（缓存友好：源码改动不会重编依赖）
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    # dummy 源码缺真实依赖会失败，但 cargo 会先把所有依赖 crate 编完缓存到 target/。
    # 用 `|| true` 让失败不阻断，下一步拷真源码再编一次即可。
    cargo build --release || true

# 2) 拷真实源码 + migrations，重新编译（依赖已缓存，只编本项目）
COPY . .
# touch 强制更新时间戳，让 cargo 识别源码已变更并重新编译本项目
# （若不 touch，cargo 会误以为 dummy 编译的 artifacts 比新复制的源码更新）
RUN find src -name '*.rs' -exec touch {} + && \
    cargo build --release --bin kugou_web_api

# ===== Stage 2: runtime =====
# bookworm-slim 与 builder 同代 glibc，二进制可直接跑。
# 比 alpine 大约多 30MB，但避免了 musl + ring 兼容问题。
FROM debian:bookworm-slim AS runtime

# ca-certificates：reqwest 走 rustls，但 rustls 校验证书链需要系统 CA。
# tini：作为 PID 1，正确转发信号（SIGTERM 让 axum 优雅退出）。
# wget：healthcheck 用（docker compose 探活 /health）。
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates tini wget && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 只拷必要文件：二进制 + 迁移脚本（启动时 sqlx::migrate! 会读）
COPY --from=builder /build/target/release/kugou_web_api /usr/local/bin/kugou_web_api
COPY --from=builder /build/migrations /app/migrations

# 容器内非 root 运行
RUN useradd --system --uid 1000 --home-dir /app kugou && \
    mkdir -p /app/data && chown -R kugou:kugou /app
USER kugou

EXPOSE 3000

# 默认环境变量（可被 docker run -e 或 compose 覆盖）
ENV APP_HOST=0.0.0.0 \
    APP_PORT=3000 \
    APP_ENV=production \
    RUST_LOG=kugou_web_api=info,tower_http=warn,sqlx=warn \
    DATABASE_URL=sqlite:///app/data/app.db?mode=rwc \
    KUGOU_GATEWAY_URL=https://gateway.kugou.com \
    HTTP_TIMEOUT_SECS=10

# SQLite 数据持久化（宿主挂卷到这里）
VOLUME ["/app/data"]

# tini 接管 PID 1，处理 SIGTERM/SIGINT
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["kugou_web_api"]
