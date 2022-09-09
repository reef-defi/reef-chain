FROM rust:1.63.0 as build

RUN rustup default nightly-2021-05-09
RUN apt-get update && apt-get install -y clang

WORKDIR /build
COPY . /build
RUN make release

FROM ubuntu:20.04

COPY --from=build /build/target/release/reef-node /usr/local/bin
ENTRYPOINT ["reef-node"]
