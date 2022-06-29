FROM rust:1.61.0 as builder
COPY ./ ./
RUN cargo install sqlx-cli
RUN cargo build --release

FROM rust:1.61.0
WORKDIR /app
RUN apt-get update && apt-get install -y libpq5
COPY --from=builder /target/release ./
EXPOSE 8080

ENTRYPOINT ["/app/pokemon_trading"]