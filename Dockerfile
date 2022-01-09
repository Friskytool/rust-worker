FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo install --path .


FROM debian as runner
WORKDIR /app

COPY --from=builder /usr/local/cargo/bin/worker-pod /usr/local/bin/worker-pod

CMD ["worker-pod"]