using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [ (name = "main", worker = .runner) ],
  sockets = [ (name = "http", address = "*:8080", http = (), service = "main") ]
);

const runner :Workerd.Worker = (
  compatibilityDate = "2023-02-28",
  compatibilityFlags = ["web_workers"],
  modules = [
    (name = "worker", esModule = embed "dist/escrin-runner.js"),
  ],
  durableObjectNamespaces = [
    (className = "EscrinSpawner", uniqueKey = "393fc4a381c4adafa76c55683f06ee4a"),
  ],
  durableObjectStorage = (inMemory = void),
  bindings = [
    (name = "gasKey", fromEnvironment = "GAS_KEY"),
    (name = "spawner", durableObjectNamespace = "EscrinSpawner"),
  ],
);
