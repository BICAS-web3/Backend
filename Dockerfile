FROM rust:latest as builder
WORKDIR /Backend
COPY . .

RUN cargo build --release

EXPOSE 8282

CMD ["./target/release/backend"]