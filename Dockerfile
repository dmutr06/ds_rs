FROM ubuntu

RUN apt-get update && apt-get install -y libopus-dev ffmpeg cmake

FROM rust

WORKDIR /
COPY . .
ENV TOKEN MTAyMjkzNzY0NDg1MTgwMjE4NA.GJxP3r.IPsd21t4vtFZmZtGOu0Z3UhVqHVp3L4LL5VXCU

RUN cargo run --release
