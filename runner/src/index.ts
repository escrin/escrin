import { ExecutionContext, Fetcher, Request } from '@cloudflare/workers-types/experimental';

import { ApiError, decodeBase64Bytes, decodeRequest, rpc, wrapFetch } from './rpc.js';

export type KmNetwork = 'sapphire-mainnet' | 'sapphire-testnet';
export type StateNetwork = 'sapphire-mainnet' | 'sapphire-testnet';

export interface EscrinRunner {
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
    fetch: wrapFetch(async (req: Request, env: { escrin: Fetcher }, ctx: ExecutionContext) => {
      const { method, params: _ } = await decodeRequest(req);
      ctx.waitUntil(
        (async () => {
          if (method === 'tasks') {
            await callbacks.tasks(new EscrinRunnerInterface(env.escrin));
          } else {
            throw new ApiError(404, `unrecognized method ${method}`);
          }
        })(),
      );
    }),
  };
}

class EscrinRunnerInterface implements EscrinRunner {
  #service: Fetcher;

  constructor(service: Fetcher) {
    this.#service = service;
  }

  async getOmniKey(keyStore: 'sapphire-mainnet' | 'sapphire-testnet'): Promise<CryptoKey> {
    const { key: keyB64 }: { key: string } = await rpc(this.#service, 'get-key', {
      keyStore: keyStore === 'sapphire-mainnet' ? 'omni-sapphire' : 'omni-sapphire-testnet',
      keyId: 'omni',
      proof: '',
    });
    const key = decodeBase64Bytes(keyB64);
    return crypto.subtle.importKey('raw', key, 'HKDF', false, ['deriveKey', 'deriveBits']);
  }
}
