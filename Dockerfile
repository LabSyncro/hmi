ARG DEBIAN_VERSION=bookworm

# Stage 1: Build environment
FROM rust:1.82-slim-${DEBIAN_VERSION} AS builder

# Install system dependencies in a single layer
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    unzip \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libssl-dev \
    pkg-config \
    build-essential \
    nodejs \
    git \
    xdg-utils \
    jq \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

WORKDIR /app

# Install Bun
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:${PATH}"

# Copy package files first for better caching
COPY package.json bun.lock* ./
COPY src-tauri/Cargo.toml src-tauri/Cargo.lock* src-tauri/
COPY src-tauri/build.rs src-tauri/
COPY src-tauri/tauri.conf.json src-tauri/

# Remove benchmark sections from Cargo.toml to reduce dependencies
RUN sed -i '/\[\[bench\]\]/,/harness = false/d' src-tauri/Cargo.toml

# Install frontend dependencies (cached layer with mount cache)
RUN --mount=type=cache,target=/root/.bun/install/cache \
    bun install --frozen-lockfile

# Pre-build Rust dependencies (cached layer with mount cache)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/src-tauri/target \
    cd src-tauri && \
    mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn lib_main() {}" > src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src/ src/
COPY public/ public/
COPY src-tauri/src/ src-tauri/src/
COPY src-tauri/icons/ src-tauri/icons/
COPY src-tauri/capabilities/ src-tauri/capabilities/
COPY *.config.* ./
COPY *.json ./
COPY index.html ./

# Copy existing auto-import files if they exist
COPY auto-imports.d.ts* ./
COPY components.d.ts* ./

# Modify package.json to skip TypeScript checking during build
RUN echo "Modifying build script to skip TypeScript checking..." && \
    jq '.scripts.build = "vite build"' package.json > package-temp.json && \
    mv package-temp.json package.json

# Build the Tauri application with cache mounts
RUN --mount=type=cache,target=/root/.bun/install/cache \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/src-tauri/target \
    echo "Building Tauri app..." && bun run tauri build

# Stage 2: Runtime environment (minimal)
FROM debian:${DEBIAN_VERSION}-slim

RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-0 \
    libgtk-3-0 \
    libayatana-appindicator3-1 \
    librsvg2-2 \
    libssl3 \
    ca-certificates \
    libudev1 \
    libxdo3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash appuser

WORKDIR /app

COPY --from=builder /app/src-tauri/target/release/hmi /app/app
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
  CMD pgrep hmi > /dev/null || exit 1

CMD ["/app/app"]
