import { Environment } from './env/index.js';
import { EscrinRunner, EscrinWorker } from './worker-interface.js';

export class Service {
  public env: Environment = new Environment({});

  private workerInterface: Comlink.Remote<OmitRunner<EscrinWorker>>;

  private scheduledTimeout: ReturnType<typeof setInterval> | number | undefined;

  private isTerminated = false;
  private error: Error | undefined;

  constructor(public readonly name: string, public readonly worker: Worker) {
    // TODO: replace `Environment` dynamic dispatch with concretized `EscrinRunner` to preserve type sanity without destroying performance via `postMessage` roundtrips.
    const svc = this;
    const rnr: EscrinRunner = {
      async getConfig() {
        const handler = svc?.env.get('config', 'getUserConfig');
        return handler ? handler() : {};
      },
      async getOmniKey(store) {
        const handler = svc?.env.get(store, 'getKey'); // TODO: type
        if (!handler) throw new Error(`unrecognized key store: ${store}`);
        let keyBytes = await handler('omni');
        if (keyBytes.item) {
          keyBytes = keyBytes.item;
        }
        if (!keyBytes) throw new Error(`unable to fetch omnikey from ${store}`);
        return keyBytes;
      },
      // async getEthProvider(network) {
      //   const handler = env.get(network, 'get-provider'); // TODO: type
      //   return handler ? handler() : undefined;
      // },
    };
    Comlink.expose(rnr, worker);
    worker.onerror = (e) => {
      console.error('worker encountered an error:', JSON.stringify(e));
      this.error = e.error;
      this.terminate();
    };
    if (this.isTerminated) throw this.error ?? new Error('failed to start service');
  }

  public async notify(): Promise<void> {
    if (this.isTerminated) throw new Error('Worker has already terminated.');
    try {
      await this.workerInterface.tasks();
    } catch (e: any) {
      this.terminate();
      throw e;
    }
  }

  public terminate(): void {
    if (this.isTerminated) return;
    this.isTerminated = true;
    this.workerInterface[Comlink.releaseProxy]();
    clearTimeout(this.scheduledTimeout);
    delete this.scheduledTimeout;
    setImmediate(() => {
      // Wait for promises to settle before destroying the isolate.
      this.worker.terminate();
    });
  }

  public schedule(period: number) {
    if (this.isTerminated) throw new Error('Worker has already terminated.');
    const notifyAndSchedule = async (period: number) => {
      if (this.isTerminated) return;
      console.log('notifying', period)
      await this.notify();
      setTimeout(notifyAndSchedule, period);
    };
    this.scheduledTimeout = setTimeout(notifyAndSchedule, period);
  }
}

type OmitRunner<T> = {
  [K in keyof T]: T[K] extends (rnr: EscrinRunner, ...args: infer Args) => infer R
    ? (...args: Args) => R
    : T[K];
};
