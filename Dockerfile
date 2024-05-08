FROM rust:bookworm as builder

RUN apt-get update && \
    apt-get install -y protobuf-compiler && \
    apt-get install -y build-essential && \
    apt-get install -y pkg-config && \
    apt-get install -y libpq-dev && \
    apt-get install -y openssl && \
    apt clean

WORKDIR /app

copy . .

RUN cargo build -p Zchronod


FROM debian:bookworm-slim

RUN apt update && apt install -y openssl

WORKDIR /app

COPY --from=builder /app/target/debug/Zchronod ./Zchronod

COPY ./zchronod/config-tempelete.yaml ./config-tempelete.yaml

EXPOSE 8080

CMD ["./Zchronod", "--config", "./config-tempelete.yaml"]
