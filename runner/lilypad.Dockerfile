FROM golang:1-bullseye AS builder

RUN apt-get update && \
    apt-get install -y curl && \
    curl -sL https://deb.nodesource.com/setup_18.x | bash - && \
    apt-get install -y nodejs jq

ARG LILYPAD_BRANCH=waterlily

RUN git clone https://github.com/escrin/lilypad --branch ${LILYPAD_BRANCH} --depth 1 /lilypad

WORKDIR /lilypad

RUN echo 'CONTRACT_ADDRESS=0x0000000000000000000000000000000000000000' >> hardhat/.env && \
    echo 'WALLET_PRIVATE_KEY=0000000000000000000000000000000000000000000000000000000000000000' >> hardhat/.env && \
    make bin/lilypad-linux-amd64

FROM gcr.io/distroless/static-debian11

COPY --from=builder /lilypad/bin/lilypad-linux-amd64 /lilypad

ENTRYPOINT ["/lilypad"]
