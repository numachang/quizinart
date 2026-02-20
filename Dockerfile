# Build stage
FROM rust:1.85-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY static/ static/
COPY locales/ locales/
COPY migrations/ migrations/

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/quizinart /usr/local/bin/quizinart

ENV ADDRESS=0.0.0.0:1414
ENV RUST_LOG=info

EXPOSE 1414

CMD ["quizinart"]
