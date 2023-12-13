using Workerd = import "/workerd/workerd.capnp";

const tlsOptions :Workerd.TlsOptions = (trustBrowserCas = true);

const runnerWorker :Workerd.Worker = (
  compatibilityDate = "2023-08-01",
  modules = [ (name = "", esModule = embed "../dist/worker/runner.js") ] ,
  bindings = [ (name = "workerd", service = "@workerd") ],
);

const internetServiceName :Text = "internet";

const topServiceName :Text = "@escrin/runner";
const runnerService :Workerd.Service = (name = .topServiceName, worker = .runnerWorker);

const iamServiceName :Text = "@escrin/iam";
const iamWorkerCompatDate :Text = "2023-11-08";
const iamWorkerModules :List(Workerd.Worker.Module) = [ (name = "", esModule = embed "../dist/worker/iam.js") ];
const gasKeyBinding :Workerd.Worker.Binding = (name = "gasKey", fromEnvironment = "GAS_KEY");

const tpmServiceName :Text = "@escrin/tpm";
