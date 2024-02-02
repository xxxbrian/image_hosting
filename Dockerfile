FROM rust:latest as builder
WORKDIR /usr/src/image_hosting
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl libssl-dev
COPY --from=builder /usr/src/image_hosting/target/release/image_hosting /usr/local/bin/image_hosting
CMD ["image_hosting"]