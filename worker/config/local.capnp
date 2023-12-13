using Workerd = import "/workerd/workerd.capnp";

using Base = import "./base.capnp";

const config :Workerd.Config = (
  services = [
    Base.runnerService,
    (name = Base.iamServiceName, worker = .iamWorker),
    (name = Base.internetServiceName,
     network = (allow = ["public", "local"], tlsOptions = Base.tlsOptions)),
  ],
  sockets = [ (name = "", address = "*:1057", http = (), service = Base.topServiceName) ],
);

const iamWorker :Workerd.Worker = (
  compatibilityDate = Base.iamWorkerCompatDate,
  modules = Base.iamWorkerModules,
  bindings = [ Base.gasKeyBinding ],
);
