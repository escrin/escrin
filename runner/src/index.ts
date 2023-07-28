import * as Comlink from 'comlink';

import { EscrinRunner, EscrinWorker } from './worker-interface.js';

export { ApiError } from './error.js';
export * from './worker-interface.js';

export interface EscrinCallbacks {
  tasks(rnr: EscrinRunner): Promise<void>;
}

export default function (callbacks: EscrinCallbacks) {
  const svcRnr = Comlink.wrap(self as any) as Comlink.Remote<EscrinRunner & {
    getOmniKey(...args: Parameters<EscrinRunner['getOmniKey']>): Promise<Uint8Array>,
  }>;
  const rnr = {
    async getOmniKey(...args: Parameters<typeof svcRnr['getOmniKey']>): Promise<CryptoKey> {
      const keyBytes = await svcRnr.getOmniKey(...args);
      return crypto.subtle.importKey('raw', keyBytes, 'HKDF', false, ['deriveKey', 'deriveBits']);
    },
    async getConfig(): Promise<Record<string, unknown>> {
      return svcRnr.getConfig();
    }
  };
  const scheduler = (globalThis as any).scheduler;
  Comlink.expose({
    tasks() {
      const p = callbacks.tasks(rnr);
      scheduler.waitUntil(p);
      return p;
    },
  });
}
