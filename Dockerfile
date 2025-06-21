ARG DEBIAN_VERSION=bookworm

FROM debian:${DEBIAN_VERSION}-slim AS runtime

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

ARG TARGETARCH
COPY docker-artifacts/${TARGETARCH}/tauri-${TARGETARCH}.tar.gz /tmp/app.tar.gz
RUN tar -xzf /tmp/app.tar.gz -C /app && rm /tmp/app.tar.gz

# Copy static icons
COPY src-tauri/icons/ /app/icons/

RUN chown -R appuser:appuser /app
USER appuser

ENV RUST_LOG=info \
    DISPLAY=:0 \
    WAYLAND_DISPLAY=wayland-0 \
    XDG_RUNTIME_DIR=/run/user/1000 \

HEALTHCHECK --interval=60s --timeout=10s --start-period=5s --retries=3 \
  CMD pgrep hmi > /dev/null || exit 1

CMD ["/app/hmi"]
