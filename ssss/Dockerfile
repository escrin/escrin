FROM alpine:3.19.1@sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b AS builder

RUN apk add nix --repository=https://dl-cdn.alpinelinux.org/alpine/edge/community

WORKDIR /code

COPY ./ ./
RUN nix --extra-experimental-features nix-command --extra-experimental-features flakes build .

FROM gcr.io/distroless/static-debian12@sha256:6dcc833df2a475be1a3d7fc951de90ac91a2cb0be237c7578b88722e48f2e56f AS ssss

COPY --from=builder /code/result/ssss /usr/local/bin/ssss

EXPOSE 1075

ENTRYPOINT ["/usr/local/bin/ssss"]
