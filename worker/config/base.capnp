using Workerd = import "/workerd/workerd.capnp";

const tlsOptions :Workerd.TlsOptions = (trustBrowserCas = true);

const internetServiceName :Text = "internet";

const runnerServiceName :Text = "@escrin/runner";
const runnerWorkerCompatDate :Text = "2023-11-08";
const runnerWorkerCompatFlags :List(Text) = ["service_binding_extra_handlers"];
const runnerWorkerModules :List(Workerd.Worker.Module) =
  [ (name = "", esModule = embed "../dist/worker/runner.js") ];
const workerdBinding :Workerd.Worker.Binding = (name = "workerd", service = "@workerd");

const iamServiceName :Text = "@escrin/iam";
const iamWorkerCompatDate :Text = "2023-11-08";
const iamWorkerModules :List(Workerd.Worker.Module) = [ (name = "", esModule = embed "../dist/worker/iam.js") ];
const gasKeyBinding :Workerd.Worker.Binding = (name = "gasKey", fromEnvironment = "GAS_KEY");

const tpmServiceName :Text = "@escrin/tpm";
