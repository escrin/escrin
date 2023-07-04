import * as Comlink from 'comlink';

import { Environment } from './env';
import { EscrinRunner, EscrinWorker } from './worker-interface';

export class Service {
  public env: Environment = new Environment({});

  private workerInterface: Comlink.Remote<EscrinWorker>;

  private scheduledTimeout: ReturnType<typeof setInterval> | number | undefined;

  private isTerminated = false;
  private error: Error | undefined;

  constructor(public readonly name: string, public readonly worker: Worker) {
    this.workerInterface = Comlink.wrap(worker);
    // TODO: replace `Environment` dynamic dispatch with concretized `EscrinRunner` to preserve type sanity without destroying performance via `postMessage` roundtrips.
    const env = this.env;
    const context: EscrinRunner = {
      getKey(store, ident) {
        const handler = env.get(store, 'get-key') as any; // TODO: type
        return handler(ident);
      },
    };
    Comlink.expose(context, worker);
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
      await this.notify();
      setTimeout(notifyAndSchedule, period);
    };
    this.scheduledTimeout = setTimeout(notifyAndSchedule, period);
  }
}
