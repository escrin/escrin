import { ExecutionContext, Fetcher, Request } from '@cloudflare/workers-types/experimental';
import { StandardMerkleTree } from '@openzeppelin/merkle-tree';
import { Address, Hash, Hex, encodeAbiParameters, hexToBigInt, hexToBytes, toHex } from 'viem';

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

export type GetAttestationParams = {
  network: NetworkNameOrNetwork;
  identity: IdentityIdOrIdentity;
  purpose?: 'acquire' | 'release';
};

export type Attestation = {
  document: Hex;
  context: Hex;
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

export type GetOmniKeyParams = {
  network: NetworkNameOrNetwork;
  identity: IdentityIdOrIdentity;
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

    const tree = StandardMerkleTree.of(
      [
        [
          BigInt(network.chainId),
          identity.registry,
          hexToBigInt(identity.id),
          !params.purpose || params.purpose === 'acquire' ? 1 : 0,
        ],
      ],
      ['uint256', 'address', 'uint256', 'bool'],
    );
    const proof = tree.getProof(0) as Hash[];

    const { document } = await rpc<tpmTypes.AttestationRequest>(this.#tpm, 'get-attestation', {
      userdata: tree.root as Hash,
    });
    return {
      document,
      context: encodeAbiParameters(
        [
          { name: 'context', type: 'string' },
          { name: 'proof', type: 'bytes32[]' },
        ],
        ['nitro', proof],
      ),
    };
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
    await rpc<iamTypes.AcquireIdentityRequest>(this.#iam, 'acquire-identity', {
      network,
      identity,
      permitTtl,
      permitter,
      authorization: authorization instanceof Uint8Array ? toHex(authorization) : authorization,
      recipient,
      duration,
    });
  }

  async getOmniKey(params: GetOmniKeyParams): Promise<CryptoKey> {
    const { network: networkNameOrNetwork, identity: identityIdOrIdentity } = params;
    const network = getNetwork(networkNameOrNetwork);
    const identity = getIdentity(identityIdOrIdentity, network);
    const { key } = await rpc<iamTypes.GetKeyRequest>(this.#iam, 'get-key', {
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
