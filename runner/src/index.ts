import { ExecutionContext, Fetcher, Request } from '@cloudflare/workers-types/experimental';
import { Address, Hash, Hex, hexToBytes, toHex } from 'viem';
import { PrivateKeyAccount, privateKeyToAccount } from 'viem/accounts';

import { ApiError, decodeRequest, rpc, wrapFetch } from './rpc.js';
import * as envTypes from './service/env/types.js';

export { ApiError } from './rpc.js';
export * from './service/env/types.js';

export interface Runner {
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
  recipient?: Address;
  authorization?: Uint8Array | Hex;
  duration?: number;
};

export type NetworkNameOrNetwork = 'local' | `sapphire-${'testnet' | 'mainnet'}` | envTypes.Network;
export type IdentityIdOrIdentity = Hash | envTypes.Identity;

export interface Callbacks {
  tasks(rnr: Runner): Promise<void>;
}

export default function (callbacks: Callbacks) {
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
              await callbacks.tasks(new RunnerInterface(env));
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
      ctx.waitUntil(callbacks.tasks(new RunnerInterface(env)));
    },
  };
}

class RunnerInterface implements Runner {
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
      recipient,
    } = params;
    const network = getNetwork(networkNameOrNetwork);
    const identity = getIdentity(identityIdOrIdentity, network);
    await rpc<envTypes.AcquireIdentityRequest>(this.#service, 'acquire-identity', {
      network,
      identity,
      permitTtl,
      permitter,
      authz: authorization instanceof Uint8Array ? toHex(authorization) : authorization,
      recipient,
      duration,
    });
  }

  async getOmniKey(params: GetOmniKeyParams): Promise<CryptoKey> {
    const { network: networkNameOrNetwork, identity: identityIdOrIdentity } = params;
    const network = getNetwork(networkNameOrNetwork);
    const identity = getIdentity(identityIdOrIdentity, network);
    const { key } = await rpc<envTypes.GetKeyRequest>(this.#service, 'get-key', {
      keyId: 'omni',
      network,
      identity,
    });
    return crypto.subtle.importKey('raw', hexToBytes(key), 'HKDF', false, [
      'deriveKey',
      'deriveBits',
    ]);
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
