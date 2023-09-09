import { ExecutionContext, Fetcher, Request } from '@cloudflare/workers-types/experimental';
import { Address, Hash, Hex, toHex } from 'viem';

import { ApiError, decodeBase64Bytes, decodeRequest, rpc, wrapFetch } from './rpc.js';
import type * as envTypes from './service/env/types.js';

export { ApiError } from './rpc.js';

export type KmNetwork = `sapphire-${'local' | 'testnet' | 'mainnet'}`;
export type StateNetwork = `sapphire-${'local' | 'testnet' | 'mainnet'}`;

export interface EscrinRunner {
  getConfig(): Promise<Record<string, any>>;

  getOmniKey(params: GetOmniKeyParams): Promise<CryptoKey>;

  acquireIdentity(params: AcquireIdentityParams): Promise<void>;
}

export type GetOmniKeyParams = {
  network: NetworkNameOrNetwork;
  identity: IdentityIdOrIdentity;
};

export type AcquireIdentityParams = {
  network: NetworkNameOrNetwork;
  identity: IdentityIdOrIdentity;
  permitter?: Address;
  permitTtl?: number;
  authorization?: Uint8Array | Hex;
  duration?: number;
};

type NetworkNameOrNetwork =
  | 'local'
  | `sapphire-${'testnet' | 'mainnet'}`
  | { chainId: number; rpcUrl: string };
type IdentityIdOrIdentity = Hash | { registry: Address; id: Hash };

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

  async acquireIdentity(params: AcquireIdentityParams): Promise<void> {
    const {
      network: networkNameOrNetwork,
      identity: identityIdOrIdentity,
      permitter,
      permitTtl,
      authorization,
      duration,
    } = params;
    const network = getNetwork(networkNameOrNetwork);
    const identity = getIdentity(identityIdOrIdentity, network);
    await rpc<envTypes.AcquireIdentityRequest>(this.#service, 'acquire-identity', {
      network,
      identity,
      permit: {
        ttl: permitTtl,
        permitter,
      },
      authz: authorization instanceof Uint8Array ? toHex(authorization) : authorization,
      duration,
    });
  }

  async getOmniKey(params: GetOmniKeyParams): Promise<CryptoKey> {
    const { network: networkNameOrNetwork, identity: identityIdOrIdentity } = params;
    const network = getNetwork(networkNameOrNetwork);
    const identity = getIdentity(identityIdOrIdentity, network);
    const { key: keyB64 } = await rpc<envTypes.GetKeyRequest>(this.#service, 'get-key', {
      keyId: 'omni',
      network,
      identity,
    });
    const key = decodeBase64Bytes(keyB64);
    return crypto.subtle.importKey('raw', key, 'HKDF', false, ['deriveKey', 'deriveBits']);
  }
}

function getNetwork(nameOrNetwork: NetworkNameOrNetwork): envTypes.Network {
  if (typeof nameOrNetwork === 'string')
    throw new Error('unable to infer network parameters, so chainId and rpcUrl are required');
  return nameOrNetwork;
}

function getIdentity(
  idOrIdentity: IdentityIdOrIdentity,
  _network: envTypes.Network,
): envTypes.Identity {
  if (typeof idOrIdentity === 'string')
    throw new Error('unable to infer identity registry, so id and registry are required');
  return idOrIdentity;
}
