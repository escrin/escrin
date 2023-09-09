import { Address, Hash, Hex, isHash, isHex } from 'viem';

import { ApiError } from '../../rpc.js';

export const isU256 = (v: unknown): v is Hex => typeof v === 'string' && isHash(v);
export const isAddress = (v: unknown): v is Hex => typeof v === 'string' && isAddress(v);

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

export type AcquireIdentityRequest = {
  method: 'acquire-identity';
  params: AcquireIdentityParams;
  response: void;
};

export type AcquireIdentityParams = {
  network: Network;
  identity: Identity;
  permit: Partial<{
    ttl: number;
    permitter: Address;
  }>;
  authz: Hex | undefined;
  duration: number | undefined;
};

export function parseAcquireIdentityParams(params: Record<string, unknown>): AcquireIdentityParams {
  const { network, identity, permit, authz, duration } = params;

  if (typeof permit !== 'undefined' && typeof permit !== 'object')
    throw new ApiError(400, 'invalid permit params');
  let permitParams = {};
  if (permit) {
    const { permitter, ttl } = permit as Record<string, unknown>;
    if (permitter !== undefined && !isAddress(permitter))
      throw new ApiError(400, 'invalid permitter');
    if (ttl !== undefined && typeof ttl !== 'number') throw new ApiError(400, 'invalid permit ttl');
    permitParams = {
      ttl,
      permitter,
    };
  }

  if (authz !== undefined && !isHex(authz)) throw new ApiError(400, 'invalid authz');
  if (duration !== undefined && typeof duration !== 'number')
    throw new ApiError(400, 'invalid permit duration');

  return {
    network: parseNetwork(network),
    identity: parseIdentity(identity),
    permit: permitParams,
    authz,
    duration,
  };
}

export type GetKeyRequest = {
  method: 'get-key';
  params: GetKeyParams;
  response: { key: string };
};

export type GetKeyParams = {
  keyId: 'omni';
} & EvmKeyStoreParams;

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
