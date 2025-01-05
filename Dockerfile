FROM rust:bookworm as builder

RUN cargo new payments_dodo
WORKDIR /payments_dodo
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo build --release

RUN ls -l /payments_dodo/target/release/
RUN ls -l /payments_dodo/target/release/payments_dodo



FROM debian:bookworm-slim
RUN apt-get update && apt install -y openssl
COPY --from=builder /payments_dodo/target/release/payments_dodo /payments_dodo
RUN chmod +x /payments_dodo
CMD ["/payments_dodo"]
EXPOSE 8080
