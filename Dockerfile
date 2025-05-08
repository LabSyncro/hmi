ARG DEBIAN_VERSION=bookworm

# Stage 1: Build environment
FROM rust:1.82-slim-${DEBIAN_VERSION} as builder

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
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Install Bun
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:${PATH}"

# First, copy everything except Cargo.lock for dependency preparation
COPY . .
RUN rm -f src-tauri/Cargo.lock

# Remove benchmark sections from Cargo.toml
RUN sed -i '/\[\[bench\]\]/,/harness = false/d' src-tauri/Cargo.toml

# Install frontend dependencies
RUN bun install

# Initialize a fresh Cargo.lock that works with this Rust version
RUN cd src-tauri && rustc --version && cargo update

# Build for Linux
RUN bun run tauri build

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
