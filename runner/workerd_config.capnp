using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "@escrin/runner", worker = .runner),
    (name = "@escrin/env", worker = .env),
  ],
  sockets = [ (name = "http", address = "*:8080", http = (), service = "@escrin/runner") ]
);

const runner :Workerd.Worker = (
  compatibilityDate = "2023-02-28",
  modules = [
    (name = "runner", esModule = embed "dist/service/escrin-runner.js"),
  ],
  durableObjectNamespaces = [
    (className = "Waker", uniqueKey = "393fc4a381c4adafa76c55683f06ee4a"),
  ],
  durableObjectStorage = (inMemory = void),
  bindings = [
    (name = "workerd", service = "@workerd"),
    (name = "waker", durableObjectNamespace = "Waker"),
  ],
);

const env :Workerd.Worker = (
  compatibilityDate = "2023-02-28",
  modules = [
    (name = "worker", esModule = embed "dist/serice/escrin-env.js"),
  ],
  bindings = [
    (name = "gasKey", fromEnvironment = "GAS_KEY"),
  ],
);
