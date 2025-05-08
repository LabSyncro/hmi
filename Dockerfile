ARG DEBIAN_VERSION=bookworm

# Stage 1: Build environment
FROM rust:1.75-slim-${DEBIAN_VERSION} as builder

RUN apt-get update && apt-get install -y \
    curl \
    wget \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libssl-dev \
    pkg-config \
    build-essential \
    nodejs \
    git \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

WORKDIR /app

COPY package*.json ./
COPY bun.lock ./

RUN curl -fsSL https://bun.sh/install | bash

ENV PATH="/root/.bun/bin:${PATH}"

RUN bun install

COPY src-tauri/Cargo.* ./src-tauri/
COPY src-tauri/tauri.conf.json ./src-tauri/

RUN mkdir -p src-tauri/src
RUN echo "fn main() {}" > src-tauri/src/main.rs

RUN cd src-tauri && cargo fetch

COPY . .

RUN bun run tauri build

# Stage 2: Runtime environment (minimal)
FROM debian:${DEBIAN_VERSION}-slim

RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.0-0 \
    libgtk-3-0 \
    libayatana-appindicator3-1 \
    librsvg2-2 \
    libssl3 \
    ca-certificates \
    libudev1 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash appuser

WORKDIR /app

COPY --from=builder /app/src-tauri/target/aarch64-unknown-linux-gnu/release/app /app/app
COPY --from=builder /app/src-tauri/icons /app/icons
COPY --from=builder /app/dist /app/dist

RUN chown -R appuser:appuser /app
USER appuser

ENV RUST_LOG=info \
    DISPLAY=:0 \
    WAYLAND_DISPLAY=wayland-0 \
    XDG_RUNTIME_DIR=/run/user/1000 \
    DEBIAN_RELEASE=bookworm

HEALTHCHECK --interval=60s --timeout=10s --start-period=5s --retries=3 \
  CMD pgrep app > /dev/null || exit 1

CMD ["/app/app"]
