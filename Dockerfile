FROM rust:1.86-slim as builder

WORKDIR /usr/src/app
COPY . .

# Build with the `http` feature so the published image supports both
# stdio (default) and streamable-HTTP transports. Without this flag the
# binary panics at runtime with "HTTP transport is not enabled. Rebuild
# with the 'http' feature" the moment a user passes `--transport http`.
# See https://github.com/gbrigandi/mcp-server-wazuh/issues/20.
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev build-essential perl make && \
    cargo build --release --features http

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/mcp-server-wazuh /app/mcp-server-wazuh
COPY .env.example /app/.env.example

RUN useradd -m wazuh
USER wazuh

EXPOSE 8000

# Use ENTRYPOINT (rather than CMD) so flags passed via
# `docker run <image> --transport http ...` are appended to the binary
# invocation instead of replacing it. The previous `CMD ["./mcp-server-wazuh"]`
# caused `docker run … --transport http` to fail with
# `exec: "--transport": executable file not found in $PATH`. Issue #20.
ENTRYPOINT ["./mcp-server-wazuh"]
CMD []
