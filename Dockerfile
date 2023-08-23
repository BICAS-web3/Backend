FROM rust:1.67 as builder

WORKDIR /usr/src/Backend
COPY . .

RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/Backend /usr/local/bin/Backend
CMD ["backend"]