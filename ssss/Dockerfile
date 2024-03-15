FROM rust:1-bookworm AS builder

WORKDIR /code

COPY ./rust-toolchain.toml ./
RUN rustup show

RUN mkdir -p s4/src src/ && touch s4/src/lib.rs src/lib.rs
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./s4/Cargo.toml ./s4
RUN cargo metadata

ARG TARGET
RUN rustup target add $TARGET

RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get -yy update && \
    apt-get -yy install build-essential musl-tools

COPY ./ ./
RUN cargo build --locked --release -p ssss -p s4 --target $TARGET

FROM gcr.io/distroless/static-debian12 AS s4

ARG TARGET

COPY --from=builder /code/target/$TARGET/release/ssss /usr/local/bin/ssss
COPY --from=builder /code/target/$TARGET/release/s4 /usr/local/bin/s4

ENTRYPOINT ["/usr/local/bin/s4"]

FROM gcr.io/distroless/static-debian12 AS ssss

ARG TARGET

COPY --from=builder /code/target/$TARGET/release/ssss /usr/local/bin/ssss

EXPOSE 1075

ENTRYPOINT ["/usr/local/bin/ssss"]
