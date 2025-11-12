FROM rust:1.88 as builder


RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    libopus-dev \
    pkg-config \
    cmake && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app


COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

# 下载依赖
RUN cargo fetch

# 复制完整的源代码
COPY src ./src

RUN cargo build --release


# --- STAGE 2: The Final Image ---
# 使用与构建环境基础版本一致的较新 Debian 版本，以解决 libssl 的问题
FROM debian:bookworm-slim


RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ffmpeg \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*


COPY --from=builder /usr/src/app/target/release/discord-qqmusic-bot /usr/local/bin/discord-qqmusic-bot

# 设置容器启动命令
CMD ["/usr/local/bin/discord-qqmusic-bot"]