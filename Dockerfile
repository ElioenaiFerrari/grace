FROM rust:1.80-slim as builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y musl-tools openssl libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release

# distroless
FROM gcr.io/distroless/cc

COPY --from=builder /usr/src/app/target/release/grace /grace

CMD ["/grace"]