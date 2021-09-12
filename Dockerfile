FROM rust:1.55 as builder

WORKDIR /usr/src/myapp
COPY Cargo.toml .
COPY build.rs .
COPY src src/
COPY proto proto/

RUN rustup component add rustfmt
RUN cargo install --path .

FROM debian:buster-slim
#RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/hello-server /usr/local/bin/hello-server

EXPOSE 6379
CMD ["hello-server"]

