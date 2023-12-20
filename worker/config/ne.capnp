using Workerd = import "/workerd/workerd.capnp";

using Base = import "./base.capnp";

const config :Workerd.Config = (
  services = [
    Base.runnerService,
    (name = Base.internetServiceName,
     network = (
       allow = ["vsock"],
       proxy = (address = "vsock:2:1057"),
       tlsOptions = Base.tlsOptions)),
    (name = Base.iamServiceName, worker = .iamWorker),
    (name = Base.tpmServiceName, worker = .tpmWorker),
  ],
  sockets = [ (name = "", address = "vsock:-1:1057", http = (), service = Base.topServiceName) ],
);

const iamWorker :Workerd.Worker = (
  compatibilityDate = Base.iamWorkerCompatDate,
  modules = Base.iamWorkerModules,
  bindings = [
    Base.gasKeyBinding,
    (name = "tpm", service = Base.tpmServiceName),
  ],
);

const tpmWorker :Workerd.Worker = (
  compatibilityDate = "2023-12-13",
  modules = [ (name = "", esModule = embed "../dist/worker/tpm.js") ],
  bindings = [ (name = "nsm", nsm = void) ],
);
