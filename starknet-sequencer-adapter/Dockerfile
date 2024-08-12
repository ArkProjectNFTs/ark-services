FROM docker.io/library/rust:1.80.0-alpine as builder
RUN apk add --no-cache musl-dev sqlite-static openssl-dev openssl-libs-static pkgconf git libpq-dev

# Set `SYSROOT` to a dummy path (default is /usr) because pkg-config-rs *always*
# links those located in that path dynamically but we want static linking, c.f.
# https://github.com/rust-lang/pkg-config-rs/blob/54325785816695df031cef3b26b6a9a203bbc01b/src/lib.rs#L613
ENV SYSROOT=/dummy

# The env vars tell libsqlite3-sys to statically link libsqlite3.
ENV SQLITE3_STATIC=1 SQLITE3_LIB_DIR=/usr/lib/

# The env var tells pkg-config-rs to statically link libpq.
ENV LIBPQ_STATIC=1

WORKDIR /wd
COPY . /wd
RUN cargo build --bins --release

FROM scratch
ARG version=unknown
ARG release=unreleased
ENV SKSQADAPTER_NAME=SYNCHRO
ENV SKSQADAPTER_ENVIRONEMENT=dev
ENV SKSQADAPTER_HEALTHCHECK_TOKEN=1a2b3c
ENV SKSQADAPTER_MULTICAST_ADDR=224.0.0.1
ENV SKSQADAPTER_BASE_URL="https://alpha-mainnet.starknet.io/feeder_gateway/get_block?blockNumber="
ENV SKSQADAPTER_MAX_CALLS_PER_MINUTE=180
ENV SKSQADAPTER_THREADS=1
ENV SKSQADAPTER_MONITOR_THREADS=2
ENV SKSQADAPTER_BLOCKS_PER_FILE=100
ENV SKSQADAPTER_PROGRESS_BAR_WIDTH=50
ENV SKSQADAPTER_MULTICAST_ADDR=224.0.0.1
ENV PORT=4001
LABEL name="Starknet Sequencer Adapter" \
      maintainer="mehdi@screenshot.co" \
      vendor="Screenshot Labs" \
      version=${version} \
      release=${release} \
      summary="A Block Sequencer ingest pipeline" \
      description="Give the possibility to ingest starnket data at the max speed"

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /wd/target/release-lto/fast-indexer /

EXPOSE 5002/udp
EXPOSE 4001/tcp

HEALTHCHECK --interval=10s --timeout=3s \
  CMD curl -f -s http://localhost:4001/ || exit 1

CMD ["./fast-indexer"]
