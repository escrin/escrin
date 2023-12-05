using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "@escrin/runner", worker = .runner),
    (name = "@escrin/iam", worker = .iam),
    (name = "internet", network =
      (allow = ["public", "local"], tlsOptions = (trustBrowserCas = true))),
  ],
  sockets = [ (name = "http", address = "*:1057", http = (), service = "@escrin/runner") ],
);

const runner :Workerd.Worker = (
  compatibilityDate = "2023-08-01",
  modules = [ (name = "", esModule = embed "dist/worker/runner.js") ] ,
  bindings = [ (name = "workerd", service = "@workerd") ],
);

const iam :Workerd.Worker = (
  compatibilityDate = "2023-08-01",
  modules = [ (name = "", esModule = embed "dist/worker/iam.js") ],
  bindings = [ (name = "gasKey", fromEnvironment = "GAS_KEY") ],
);
