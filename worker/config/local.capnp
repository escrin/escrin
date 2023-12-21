using Workerd = import "/workerd/workerd.capnp";

using Base = import "./base.capnp";

const config :Workerd.Config = (
  services = [
    (name = Base.runnerServiceName, worker = .runnerWorker),
    (name = Base.iamServiceName, worker = .iamWorker),
    (name = Base.internetServiceName,
     network = (allow = ["public", "local"], tlsOptions = Base.tlsOptions)),
  ],
  sockets = [ (name = "", address = "*:1057", http = (), service = Base.runnerServiceName) ],
);

const runnerWorker :Workerd.Worker = (
  compatibilityDate = Base.runnerWorkerCompatDate,
  modules =  Base.runnerWorkerModules,
  bindings = [ Base.workerdBinding, (name = "config", json = "{\"tpm\": false}") ],
);

const iamWorker :Workerd.Worker = (
  compatibilityDate = Base.iamWorkerCompatDate,
  modules = Base.iamWorkerModules,
  bindings = [ Base.gasKeyBinding ],
);
