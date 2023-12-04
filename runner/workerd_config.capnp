using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "@escrin/runner", worker = .runner),
    (name = "@escrin/iam", worker = .iam),
    (name = "internet", network = (
        allow = ["public", "local"],
        tlsOptions = ( trustBrowserCas = true )
      )
    )
  ],
  sockets = [ (name = "http", address = "*:1057", http = (), service = "@escrin/runner") ]
);

const runner :Workerd.Worker = (
  compatibilityDate = "2023-11-08",
  modules = [ (name = "runner", esModule = embed "dist/svc/run.js")] ,
  bindings = [ (name = "workerd", service = "@workerd") ],
);

const iam :Workerd.Worker = (
  compatibilityDate = "2023-11-08",
  modules = [ (name = "worker", esModule = embed "dist/svc/iam.js") ],
  bindings = [ (name = "gasKey", fromEnvironment = "GAS_KEY") ],
);
