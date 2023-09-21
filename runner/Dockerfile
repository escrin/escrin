FROM node:20 AS runner-builder

WORKDIR /escrin

RUN npm install -g pnpm

ARG WORKERD_TAG

RUN curl -sSL \
  https://github.com/escrin/workerd/releases/download/${WORKERD_TAG}/workerd-linux-64.gz \
  | gunzip > workerd
RUN chmod a+x workerd

COPY ./ ./

RUN pnpm --filter "@escrin/worker" install --frozen-lockfile
RUN pnpm --filter "@escrin/worker" build
RUN ./workerd compile runner/workerd_config.capnp > escrin-runner

FROM gcr.io/distroless/cc:latest

COPY --from=runner-builder /escrin/escrin-runner /escrin-runner

ENTRYPOINT ["/escrin-runner", "--verbose"]
