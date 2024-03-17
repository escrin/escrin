FROM rust:1.76.0-alpine@sha256:e594a9705c4514c0e0b5ed2409f7ec34f20af09a33d242524281247b74196c43 AS builder

WORKDIR /code

COPY ./rust-toolchain.toml ./
RUN rustup show

RUN mkdir -p s4/src src/ && touch s4/src/lib.rs src/lib.rs
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./s4/Cargo.toml ./s4
RUN cargo metadata

RUN apk add --no-cache musl-dev==1.2.4_git20230717-r4

COPY ./ ./
RUN cargo build --locked --release -p ssss

FROM gcr.io/distroless/static-debian12@sha256:6dcc833df2a475be1a3d7fc951de90ac91a2cb0be237c7578b88722e48f2e56f AS ssss

COPY --from=builder /code/target/release/ssss /usr/local/bin/ssss

EXPOSE 1075

ENTRYPOINT ["/usr/local/bin/ssss"]
