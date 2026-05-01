FROM rust:1.85-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/timer_with_messages /usr/local/bin/
RUN mkdir -p /data
WORKDIR /data
EXPOSE 3000
CMD ["timer_with_messages"]
