using Workerd = import "/workerd/workerd.capnp";

const config :Workerd.Config = (
  services = [
    (name = "@escrin/runner", worker = .runner),
    (name = "@escrin/iam", worker = .iam),
    (name = "internet", network =
      (allow = ["public", "local"], tlsOptions = (trustBrowserCas = true)),
    ),
  ],
  sockets = [ (name = "http", address = "*:1057", http = (), service = "@escrin/runner") ],
);

const runner :Workerd.Worker = (
  compatibilityDate = "2023-11-08",
  modules = [ (name = "", esModule = embed "dist/runner.js")] ,
  bindings = [ (name = "workerd", service = "@workerd") ],
);

const iam :Workerd.Worker = (
  compatibilityDate = "2023-11-08",
  modules = [ (name = "", esModule = embed "dist/env/iam.js") ],
  bindings = [ (name = "nsm", nsm = void) ],
);

const escrin :Workerd.Extension = (
  modules = [
    (name = "escrin:nsm", esModule = embed "dist/env/nsm.js"),
    (name = "escrin:iam", esModule = embed "dist/env/iam.js"),
  ],
);
