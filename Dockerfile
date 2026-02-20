# Stage 1: cargo-chef — 依存クレートのキャッシュ用レシピ生成
FROM rust:1.88-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: 依存クレートのビルド（ソース変更ではキャッシュヒット）
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 3: アプリ本体のビルド（自分のコードだけ再コンパイル）
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY static/ static/
COPY locales/ locales/
COPY migrations/ migrations/
RUN cargo build --release

# Stage 4: 軽量ランタイム
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/quizinart /usr/local/bin/quizinart

ENV ADDRESS=0.0.0.0:1414
ENV RUST_LOG=info

EXPOSE 1414
CMD ["quizinart"]
