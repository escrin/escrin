import { ExecutionContext, Fetcher, Request } from '@cloudflare/workers-types/experimental';

import { ApiError } from './error.js';

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
  let runnerInterface: EscrinRunnerInterface | undefined;

  return {
    async fetch(req: Request, env: { escrin: Fetcher }, ctx: ExecutionContext) {
      if (!runnerInterface) {
        runnerInterface = new EscrinRunnerInterface(env.escrin);
      }
      const { method, params: _ } = await decodeRequest(req);
      ctx.waitUntil(
        (async () => {
          if (method === 'tasks') {
            return callbacks.tasks(runnerInterface);
          } else {
            throw new ApiError(404, `unrecognized method ${method}`);
          }
        })(),
      );
    },
  };
}

async function decodeRequest(req: Request): Promise<{ method: string; params: any[] }> {
  if (req.headers.get('content-type') !== 'application/json')
    throw new ApiError(400, 'invalid content-type. application/json is required');
  let bodyJson: unknown;
  try {
    bodyJson = await req.json();
  } catch (e: any) {
    throw new ApiError(400, `unable to decode request body: ${e}`);
  }
  if (typeof bodyJson !== 'object' || bodyJson === null)
    throw new ApiError(400, `invalid request body`);
  if (!('method' in bodyJson) || typeof bodyJson.method !== 'string')
    throw new ApiError(400, `invalid request body: missing or invalid method`);
  if (!('params' in bodyJson) || !Array.isArray(bodyJson.params))
    throw new ApiError(400, `invalid request body: missing or invalid params`);
  return {
    method: bodyJson.method,
    params: bodyJson.params,
  };
}

class EscrinRunnerInterface implements EscrinRunner {
  #baseUrl = 'http://runner.escrin'; // A URL is required by the `fetch` interface. The request does not actually get sent to the network.

  constructor(private readonly escrin: Fetcher) {}

  async getOmniKey(keyStore: 'sapphire-mainnet' | 'sapphire-testnet'): Promise<CryptoKey> {
    const res = await this.escrin.fetch(`${this.#baseUrl}/omni-key/${keyStore}`);
    const { key: keyB64 } = await res.json() as { key: string };
    const key = decodeBase64Bytes(keyB64);
    return crypto.subtle.importKey('raw', key, 'HKDF', false, ['deriveKey', 'deriveBits']);
  }
}

function decodeBase64Bytes(base64: string): Uint8Array {
  let binaryStr = atob(base64);
  let len = binaryStr.length;
  let bytes = new Uint8Array(binaryStr.length);

  for (let i = 0; i < len; i++) {
    bytes[i] = binaryStr.charCodeAt(i);
  }

  return bytes;
}
