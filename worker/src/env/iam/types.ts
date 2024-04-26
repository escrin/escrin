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
  id: EphemeralAccount;
};

export function parseGetAccountParams(params: Record<string, unknown>): GetAccountParams {
  const { id } = params;
  if (id !== 'ephemeral-account') throw new ApiError(400, 'invalid account id');
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

  /**
   * For chained Permitters, the first Permitter in the chain.
   * Default: the permitter registered with the identity registry.
   */
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

  /** An override for the gas key used to pay for identity acquisition. */
  gasKey?: Hex;

  /** @experimental */
  ssss?: SsssParams;
};

export type SsssParams = {
  /** The address of the SSSSHub contract. */
  hub: Address;
  /** The M in the M-of-N secret sharing scheme. */
  quorum: number;
  /** The URLs of the SSSSs to be contacted. */
  urls: string[];
};

export function parseAcquireIdentityParams(params: Record<string, unknown>): AcquireIdentityParams {
  const {
    network,
    identity,
    permitter,
    permitTtl,
    recipient,
    context,
    authorization,
    duration,
    ssss,
  } = params;

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
    ssss: checkSsss(ssss),
  };
}

export type GetKeyRequest = {
  method: 'get-key';
  params: GetKeyParams;
  response: { key: Hex };
};

type EphemeralAccount = 'ephemeral-account';

export type GetKeyParams =
  | ({
      keyId: 'omni';
    } & EvmKeyStoreParams)
  | {
      keyId: EphemeralAccount;
    };

export type EvmKeyStoreParams = {
  network: Network;
  identity: {
    registry: Address;
    id: Hash;
  };
  ssss?: SsssParams;
};

export function parseGetKeyParams(params: Record<string, unknown>): GetKeyParams {
  const { keyId, ...providerParams } = params;
  if (keyId !== 'omni') throw new ApiError(400, `unknown key id ${keyId}`);

  // There is only the one provider kind (EVM), so no guard is needed.
  const { network, identity, ssss } = providerParams;
  return {
    keyId,
    network: parseNetwork(network),
    identity: parseIdentity(identity),
    ssss: checkSsss(ssss),
  };
}

function checkSsss(ssss: unknown): SsssParams | undefined {
  if (ssss === undefined || ssss === null) return undefined;
  const { urls, quorum, hub } = ssss as Record<string, unknown>;

  if (!Array.isArray(urls) || urls.length === 0)
    throw new ApiError(400, 'invalid ssss: missing urls');
  const normalizedUrls = urls.map((u) => {
    try {
      const url = new URL(u);
      if (url.protocol !== 'http:' && url.protocol !== 'https:')
        throw new Error(`unsupported protocol: ${url.protocol}`);
      url.pathname = url.pathname.replace(/\/+$/, '');
      if (!url.pathname.endsWith('/v1')) throw new Error('unsupported SSSS API version');
      return url.toString();
    } catch (e: any) {
      throw new ApiError(400, `invalid ssss: invalid ssss url: ${e}`);
    }
  });

  if (typeof quorum !== 'number' || quorum <= 0 || quorum % 1 > 0 || quorum > normalizedUrls.length)
    throw new ApiError(400, 'invalid ssss: invalid quorum');

  if (!isAddress(hub)) throw new ApiError(400, 'invalid ssss: missing hub');

  return { hub, quorum, urls: normalizedUrls };
}
