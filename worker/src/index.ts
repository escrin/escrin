import { ExecutionContext, Fetcher, Request } from '@cloudflare/workers-types/experimental';
import { Address, Hash, Hex, encodeAbiParameters, hexToBytes, keccak256, toHex } from 'viem';

import { ApiError, decodeRequest, rpc, wrapFetch } from './rpc.js';
import * as iamTypes from './env/iam/types.js';
import * as tpmTypes from './env/tpm/types.js';

export { ApiError } from './rpc.js';
export * from './env/iam/types.js';
export * from './env/tpm/types.js';

export interface Runner {
  getConfig(): Promise<Record<string, any>>;

  getAttestation(params: GetAttestationParams): Promise<Attestation>;

  acquireIdentity(params: AcquireIdentityParams): Promise<void>;

  getOmniKey(params: GetOmniKeyParams): Promise<CryptoKey>;
}

export type AcqRelIdentityParams = {
  network: NetworkNameOrNetwork;
  identity: IdentityIdOrIdentity;
  recipient?: Address;
  permitter?: Address;
};

export type GetAttestationParams = AcqRelIdentityParams & {
  purpose?: 'acquire' | 'release';
};

export type Attestation = {
  document: Hex;
};

export type AcquireIdentityParams = AcqRelIdentityParams & {
  permitTtl?: number;
  duration?: number;
  authorization?: Uint8Array | Hex;
  /** @experimental */
  ssss?: iamTypes.SsssParams;
};

export type GetOmniKeyParams = {
  network: NetworkNameOrNetwork;
  identity: IdentityIdOrIdentity;
  /** @experimental */
  ssss?: iamTypes.SsssParams;
};

export type NetworkNameOrNetwork = 'local' | `sapphire-${'testnet' | 'mainnet'}` | iamTypes.Network;
export type IdentityIdOrIdentity = Hash | iamTypes.Identity;

export interface Callbacks {
  tasks(rnr: Runner): Promise<void>;
}

export type WorkerEnv = { config: Record<string, any>; iam: Fetcher; tpm?: Fetcher };

export default function (callbacks: Callbacks) {
  return {
    fetch: wrapFetch(async (req: Request, env: WorkerEnv, ctx: ExecutionContext) => {
      const { method } = await decodeRequest(req);
      ctx.waitUntil(
        (async () => {
          if (method === 'tasks') {
            await callbacks.tasks(new RunnerInterface(env));
          } else {
            throw new ApiError(404, `unrecognized method ${method}`);
          }
        })(),
      );
    }),
    scheduled(_event: ScheduledEvent, env: WorkerEnv, ctx: ExecutionContext) {
      ctx.waitUntil(callbacks.tasks(new RunnerInterface(env)));
    },
  };
}

class RunnerInterface implements Runner {
  #config: Record<string, any>;
  #iam: Fetcher;
  #tpm?: Fetcher;

  constructor(env: WorkerEnv) {
    this.#iam = env.iam;
    this.#tpm = env.tpm;
    this.#config = env.config;
  }

  async getConfig(): Promise<Record<string, any>> {
    return this.#config;
  }

  async getAttestation(params: GetAttestationParams): Promise<Attestation> {
    if (!this.#tpm) throw new ApiError(404, 'no TPM');
    const { network: networkNameOrNetwork, identity: identityIdOrIdentity } = params;
    const network = getNetwork(networkNameOrNetwork);
    const identity = getIdentity(identityIdOrIdentity, network);

    const recipient =
      params.recipient ??
      (await rpc<iamTypes.GetAccountRequest>(this.#iam, 'get-account', { id: 'ephemeral-account' }))
        .address;

    const userdata = keccak256(
      encodeAbiParameters(
        [
          { name: 'chain', type: 'uint64' },
          { name: 'registry', type: 'address' },
          { name: 'identity', type: 'bytes32' },
          { name: 'recipient', type: 'address' },
          { name: 'acquire', type: 'bool' },
        ],
        [
          BigInt(network.chainId),
          identity.registry,
          identity.id,
          recipient,
          !params.purpose || params.purpose === 'acquire',
        ],
      ),
    );
    const { document } = await rpc<tpmTypes.AttestationRequest>(this.#tpm, 'get-attestation', {
      userdata,
    });
    return { document };
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
      ssss,
    } = params;
    const network = getNetwork(networkNameOrNetwork);
    const identity = getIdentity(identityIdOrIdentity, network);
    await rpc<iamTypes.AcquireIdentityRequest>(this.#iam, 'acquire-identity', {
      network,
      identity,
      permitTtl,
      permitter,
      authorization: authorization instanceof Uint8Array ? toHex(authorization) : authorization,
      recipient,
      duration,
      ssss,
    });
  }

  async getOmniKey(params: GetOmniKeyParams): Promise<CryptoKey> {
    const { network: networkNameOrNetwork, identity: identityIdOrIdentity, ssss } = params;
    const network = getNetwork(networkNameOrNetwork);
    const identity = getIdentity(identityIdOrIdentity, network);
    const { key } = await rpc<iamTypes.GetKeyRequest>(this.#iam, 'get-key', {
      keyId: 'omni',
      network,
      identity,
      ssss,
    });
    return crypto.subtle.importKey('raw', hexToBytes(key), 'HKDF', false, [
      'deriveKey',
      'deriveBits',
    ]);
  }
}

function getNetwork(nameOrNetwork: NetworkNameOrNetwork): iamTypes.Network {
  if (typeof nameOrNetwork === 'string')
    throw new Error('unable to infer network parameters, so chainId and rpcUrl are required');
  return nameOrNetwork;
}

function getIdentity(
  idOrIdentity: IdentityIdOrIdentity,
  _network: iamTypes.Network,
): iamTypes.Identity {
  if (typeof idOrIdentity === 'string')
    throw new Error('unable to infer identity registry, so id and registry are required');
  return idOrIdentity;
}
