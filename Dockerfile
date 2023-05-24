FROM ubuntu

RUN apt-get update && apt-get install -y libopus-dev ffmpeg cmake

FROM rust

WORKDIR /
COPY . .

RUN cargo run --release
