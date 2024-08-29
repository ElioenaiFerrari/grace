FROM rust:1.80-alpine as builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

# distroless
FROM gcr.io/distroless/cc

COPY --from=builder /usr/src/app/target/release/grace /grace

CMD ["/grace"]