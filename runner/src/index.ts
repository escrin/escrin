import { ExecutionContext, Fetcher, Request } from '@cloudflare/workers-types/experimental';

import { ApiError, decodeBase64Bytes, decodeRequest, rpc, wrapFetch } from './rpc.js';

export { ApiError } from './rpc.js';

export type KmNetwork = 'sapphire-mainnet' | 'sapphire-testnet';
export type StateNetwork = 'sapphire-mainnet' | 'sapphire-testnet';

export interface EscrinRunner {
  getConfig(): Promise<Record<string, any>>;
  getOmniKey(store: KmNetwork): Promise<CryptoKey>;
}

export interface EIP1193Provider {
  request: (request: EIP1193Request) => Promise<EIP1193Response>;
}

export interface EIP1193Request {
  method: string;
  params?: any[];
}

export interface EIP1193Response {
  result?: any;
  error?: string;
}

export interface EscrinCallbacks {
  tasks(rnr: EscrinRunner): Promise<void>;
}

export default function (callbacks: EscrinCallbacks) {
  return {
    fetch: wrapFetch(
      async (
        req: Request,
        env: { escrin: Fetcher; config: Record<string, any> },
        ctx: ExecutionContext,
      ) => {
        const { method, params: _ } = await decodeRequest(req);
        ctx.waitUntil(
          (async () => {
            if (method === 'tasks') {
              await callbacks.tasks(new EscrinRunnerInterface(env));
            } else {
              throw new ApiError(404, `unrecognized method ${method}`);
            }
          })(),
        );
      },
    ),
    scheduled(
      _event: ScheduledEvent,
      env: { escrin: Fetcher; config: Record<string, any> },
      ctx: ExecutionContext,
    ) {
      ctx.waitUntil(callbacks.tasks(new EscrinRunnerInterface(env)));
    },
  };
}

class EscrinRunnerInterface implements EscrinRunner {
  #service: Fetcher;
  #config: Record<string, any>;

  constructor(env: { escrin: Fetcher; config: Record<string, any> }) {
    this.#service = env.escrin;
    this.#config = env.config;
  }

  async getConfig(): Promise<Record<string, any>> {
    return this.#config;
  }

  async getOmniKey(keyStore: 'sapphire-mainnet' | 'sapphire-testnet'): Promise<CryptoKey> {
    const { key: keyB64 }: { key: string } = await rpc(this.#service, 'get-key', {
      keyStore,
      keyId: 'omni',
      proof: '',
    });
    const key = decodeBase64Bytes(keyB64);
    return crypto.subtle.importKey('raw', key, 'HKDF', false, ['deriveKey', 'deriveBits']);
  }
}
