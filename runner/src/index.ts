import * as Comlink from 'comlink';

import { EscrinRunner, EscrinWorker } from './worker-interface.js';

export { ApiError } from './error.js';
export * from './worker-interface.js';

export interface EscrinCallbacks {
  tasks(rnr: EscrinRunner): Promise<void>;
}

export default function (callbacks: EscrinCallbacks) {
  const rnr = Comlink.wrap(self as any) as Comlink.Remote<EscrinRunner>;
  Comlink.expose({
    async tasks(): Promise<void> {
      return callbacks.tasks(rnr);
    },
  } as EscrinWorker);
}
