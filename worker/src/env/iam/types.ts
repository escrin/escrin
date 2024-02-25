import { Address, Hash, Hex, isAddress as stringIsAddress, isHash, isHex } from 'viem';

import { ApiError } from '../../rpc.js';

export const isU256 = (v: unknown): v is Hex => typeof v === 'string' && isHash(v);
export const isAddress = (v: unknown): v is Hex => typeof v === 'string' && stringIsAddress(v);

export type Network = {
  chainId: number;
  rpcUrl?: string;
};

export function parseNetwork(network: unknown): Network {
  if (typeof network !== 'object' || network === null)
    throw new ApiError(400, 'missing network params');
  const { chainId, rpcUrl } = network as Record<string, unknown>;
  if (!chainId) throw new ApiError(400, 'missing chain id');
  if (typeof chainId !== 'number') throw new ApiError(400, 'invalid chain id');
  if (rpcUrl !== undefined && typeof rpcUrl !== 'string') throw new ApiError(400, 'invalid rpcUrl');
  return { chainId, rpcUrl };
}

export type Identity = {
  registry: Address;
  id: Hash;
};

export function parseIdentity(identity: unknown): { registry: Address; id: Hash } {
  if (typeof identity !== 'object' || !identity) throw new ApiError(400, 'missing identity params');
  const { registry, id } = identity as Record<string, unknown>;
  if (!registry) throw new ApiError(400, 'missing identity registry');
  if (!isAddress(registry)) throw new ApiError(400, 'invalid identity registry');
  if (!id) throw new ApiError(400, 'missing identity id');
  if (!isU256(id)) throw new ApiError(400, 'invalid identity id');
  return { registry, id };
}

export type GetAccountRequest = {
  method: 'get-account';
  params: GetAccountParams;
  response: {
    address: Address;
  };
};

export type GetAccountParams = {
  id: 'worker';
};

export function parseGetAccountParams(params: Record<string, unknown>): GetAccountParams {
  const { id } = params;
  if (id !== 'worker') throw new ApiError(400, 'invalid account id');
  return { id };
}

export type AcquireIdentityRequest = {
  method: 'acquire-identity';
  params: AcquireIdentityParams;
  response: void;
};

export type AcquireIdentityParams = {
  /** The network where the identity is registered. */
  network: Network;
  /** A pointer to the identity to acquire. */
  identity: Identity;

  /** For chained Permitters, the first Permitter in the chain. */
  permitter?: Address;

  /** The address of the recipient of the permit. Default: the worker's ephemeral wallet. */
  recipient?: Address;

  /** Bytes passed directly to the Permitter as the `context` parameter. */
  context?: Hex;
  /** Bytes passed directly to the Permitter as the `authorization` parameter. */
  authorization?: Hex;

  /**
   * The length of time that the permit should last. Default: 24 hours.
   * A longer TTL means fewer renewals and lower gas fees, but slower response to policy changes.
   */
  permitTtl?: number;

  /** Set this to the duration of the permit-requiring critical section to hint to the runner that it can avoid renewing a permit. */
  duration?: number;
};

export function parseAcquireIdentityParams(params: Record<string, unknown>): AcquireIdentityParams {
  const { network, identity, permitter, permitTtl, recipient, context, authorization, duration } =
    params;

  if (recipient !== undefined && !isAddress(recipient))
    throw new ApiError(400, 'invalid recipient');
  if (permitter !== undefined && !isAddress(permitter))
    throw new ApiError(400, 'invalid permitter');
  if (permitTtl !== undefined && typeof permitTtl !== 'number')
    throw new ApiError(400, 'invalid permit ttl');

  if (authorization !== undefined && !isHex(authorization))
    throw new ApiError(400, 'invalid authorization');
  if (context !== undefined && !isHex(context)) throw new ApiError(400, 'invalid context');
  if (duration !== undefined && typeof duration !== 'number')
    throw new ApiError(400, 'invalid permit duration');

  return {
    network: parseNetwork(network),
    identity: parseIdentity(identity),
    permitter,
    recipient,
    context,
    authorization,
    permitTtl,
    duration,
  };
}

export type GetKeyRequest = {
  method: 'get-key';
  params: GetKeyParams;
  response: { key: Hex };
};

export type GetKeyParams =
  | ({
      keyId: 'omni';
    } & EvmKeyStoreParams)
  | {
      keyId: 'ephemeral-account';
    };

export type EvmKeyStoreParams = {
  network: Network;
  identity: {
    registry: Address;
    id: Hash;
  };
};

export function parseGetKeyParams(params: Record<string, unknown>): GetKeyParams {
  const { keyId, keyStore, ...providerParams } = params;
  if (keyId !== 'omni') throw new ApiError(400, `unknown key id ${keyId}`);

  // Only one provider, so no guard is needed.
  const { network, identity } = providerParams;
  return {
    keyId,
    network: parseNetwork(network),
    identity: parseIdentity(identity),
  };
}
