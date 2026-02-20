# Stage 1: Build the tap-http binary
FROM rust:1.83-bookworm AS builder

WORKDIR /build

# Copy workspace manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY tap-msg/Cargo.toml tap-msg/Cargo.toml
COPY tap-msg-derive/Cargo.toml tap-msg-derive/Cargo.toml
COPY tap-agent/Cargo.toml tap-agent/Cargo.toml
COPY tap-caip/Cargo.toml tap-caip/Cargo.toml
COPY tap-node/Cargo.toml tap-node/Cargo.toml
COPY tap-http/Cargo.toml tap-http/Cargo.toml
COPY tap-wasm/Cargo.toml tap-wasm/Cargo.toml
COPY tap-mcp/Cargo.toml tap-mcp/Cargo.toml
COPY tap-ivms101/Cargo.toml tap-ivms101/Cargo.toml

# Create stub source files so cargo can resolve the workspace
RUN mkdir -p tap-msg/src tap-msg-derive/src tap-agent/src tap-caip/src \
    tap-node/src tap-http/src tap-wasm/src tap-mcp/src tap-ivms101/src && \
    echo "fn main() {}" > tap-http/src/main.rs && \
    for crate in tap-msg tap-msg-derive tap-agent tap-caip tap-node tap-wasm tap-mcp tap-ivms101; do \
        echo "" > "$crate/src/lib.rs"; \
    done

# Pre-build dependencies (cached unless Cargo.toml/Cargo.lock change)
RUN cargo build --release --package tap-http 2>/dev/null || true

# Copy the actual source code
COPY tap-msg/ tap-msg/
COPY tap-msg-derive/ tap-msg-derive/
COPY tap-agent/ tap-agent/
COPY tap-caip/ tap-caip/
COPY tap-node/ tap-node/
COPY tap-http/ tap-http/
COPY tap-ivms101/ tap-ivms101/

# Touch source files to invalidate the cached stub builds
RUN find tap-msg tap-msg-derive tap-agent tap-caip tap-node tap-http tap-ivms101 \
    -name "*.rs" -exec touch {} +

# Build the release binary
RUN cargo build --release --package tap-http

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates sqlite3 && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN groupadd --gid 1000 tap && \
    useradd --uid 1000 --gid tap --create-home tap

# Create the data directory with proper ownership
RUN mkdir -p /data/tap && chown -R tap:tap /data/tap

# Copy the built binaries
COPY --from=builder /build/target/release/tap-http /usr/local/bin/tap-http
COPY --from=builder /build/target/release/tap-payment-simulator /usr/local/bin/tap-payment-simulator

USER tap

# All persistent state lives under /data/tap
ENV TAP_ROOT=/data/tap
ENV TAP_LOGS_DIR=/data/tap/logs
ENV TAP_HTTP_HOST=0.0.0.0
ENV TAP_HTTP_PORT=8000

EXPOSE 8000

VOLUME ["/data/tap"]

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["tap-http", "--help"]

ENTRYPOINT ["tap-http"]
