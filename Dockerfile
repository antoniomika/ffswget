FROM rustlang/rust:nightly as builder
LABEL maintainer="Antonio Mika <me@antoniomika.me>"

WORKDIR /usr/src
RUN USER=root cargo new ffswget

WORKDIR /usr/src/ffswget
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
RUN rm /usr/src/ffswget/target/release/ffswget* /usr/src/ffswget/target/release/deps/ffswget*
RUN cargo build --release

FROM debian:buster-slim
LABEL maintainer="Antonio Mika <me@antoniomika.me>"

EXPOSE 8000

RUN apt-get update && apt-get install -y openssl ca-certificates
COPY --from=builder /usr/src/ffswget/target/release/ffswget /ffswget

ENTRYPOINT ["/ffswget"]