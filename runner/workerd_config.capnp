using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [ (name = "main", worker = .runner) ],
  sockets = [ (name = "http", address = "*:8080", http = (), service = "main") ]
);

const runner :Workerd.Worker = (
  compatibilityDate = "2023-02-28",
  compatibilityFlags = ["web_workers"],
  modules = [
    (name = "worker", esModule = embed "dist/tmm.js")
  ],
  bindings = [
    (name = "gasKey", fromEnvironment = "GAS_KEY")
  ],
);
