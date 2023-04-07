FROM golang:1-bullseye AS builder

ARG BACALHAU_BRANCH=main

RUN git clone https://github.com/escrin/bacalhau --branch ${BACALHAU_BRANCH} --depth 1 /bacalhau


WORKDIR /bacalhau

RUN make build-bacalhau

FROM gcr.io/distroless/static-debian11

COPY --from=builder /bacalhau/bin/linux_amd64/bacalhau /bacalhau

ENTRYPOINT ["/bacalhau"]
