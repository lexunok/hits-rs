FROM rust:1.92-slim-bookworm AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/hits-rs
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim AS runner

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/hits-rs/target/release/hits-rs /usr/local/bin/hits-rs

EXPOSE 8080
CMD ["hits-rs"]
