using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "@escrin/runner", worker = .runner),
    (name = "@escrin/env", worker = .env),
    (name = "internet", network = (
        # allow = ["public", "local"],
        tlsOptions = ( trustBrowserCas = true )
      )
    )
  ],
  sockets = [ (name = "http", address = "*:8080", http = (), service = "@escrin/runner") ]
);

const runner :Workerd.Worker = (
  compatibilityDate = "2023-02-28",
  modules = [
    (name = "runner", esModule = embed "dist/service/escrin-runner.js")
  ],
  bindings = [ (name = "workerd", service = "@workerd") ],
  bindings = [ (name = "mode", fromEnvironment = "ESCRIN_MODE") ],
);

const env :Workerd.Worker = (
  compatibilityDate = "2023-02-28",
  modules = [
    (name = "worker", esModule = embed "dist/service/escrin-env.js"),
  ],
  bindings = [
    (name = "gasKey", fromEnvironment = "GAS_KEY"),
  ],
);
