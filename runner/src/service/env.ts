import { Address, Hex, hexToBigInt } from 'viem';
import { Chain, foundry, localhost } from 'viem/chains';

import * as chains from '../env/keystore/chains.js';
import * as sapphireKeystore from '../env/keystore/sapphire.js';
import { ApiError, decodeRequest, encodeBase64Bytes, wrapFetch } from '../rpc.js';

export default new (class {
  public readonly fetch = wrapFetch(async (req, env: { gasKey?: string }) => {
    const requester = req.headers.get('x-caller-id');
    if (!requester) throw new ApiError(500, 'escrin-runner did not set x-caller-id header');
    const { method, params } = await decodeRequest(req);

    if (method === 'get-key') {
      const { keyStore, keyId, ...opts } = params;

      if (keyId !== 'omni') throw new ApiError(404, `unknown key id: ${keyId}`);

      if (!isKeyStoreOpts(keyStore))
        throw new ApiError(400, `missing or invalid key store: ${JSON.stringify(keyStore)}`);

      let key;
      if (keyStore.kind === 'sapphire') {
        if (!isU256(env.gasKey)) {
          throw new ApiError(500, 'escrin-runner not configured: missing or invalid `gas-key`');
        }
        key = await getSapphireOmniKey(opts, keyStore, env.gasKey as `0x{string}`);
      } else {
        throw new ApiError(400, `unknown key store kind: ${keyStore.kind}`);
      }

      return { key: encodeBase64Bytes(key) };
    }

    throw new ApiError(404, `unknown method: ${method}`);
  });
})();

async function getSapphireOmniKey(
  opts: Record<string, unknown>,
  keyStore: SapphireKeyStoreOpts,
  gasKey: Hex,
): Promise<Uint8Array> {
  const { identity, authz, permitter } = opts;

  if (!isU256(identity)) throw new ApiError(400, 'invalid identity');
  if (authz && !(authz instanceof Uint8Array)) throw new ApiError(400, 'invalid authz');
  if (permitter !== undefined && !isAddress(permitter))
    throw new ApiError(400, 'invalid permitter');

  return await sapphireKeystore.getOmniKey({
    identity: hexToBigInt(identity),
    keyStore: {
      chain: getChain(keyStore.chainId),
      address: keyStore.address,
    },
    authz: (authz as Uint8Array) ?? new Uint8Array(),
    gasKey,
    isSapphire: keyStore.chainId === 0x5afe || keyStore.chainId == 0x5aff,
    overrides: {
      permitter,
    },
  });
}

function isKeyStoreOpts(ks: unknown): ks is KeyStoreOpts {
  return isSapphireKeyStoreOpts(ks);
}

function isSapphireKeyStoreOpts(ks: unknown): ks is SapphireKeyStoreOpts {
  return (
    typeof ks === 'object' &&
    ks !== null &&
    'kind' in ks &&
    ks.kind === 'sapphire' &&
    'chainId' in ks &&
    typeof ks.chainId === 'number' &&
    (!('address' in ks) || isAddress(ks.address))
  );
}

export type KeyStoreOpts = SapphireKeyStoreOpts;
export type SapphireKeyStoreOpts = {
  kind: 'sapphire';
  chainId: number;
  address: Address;
};

function getChain(chainId: number): Chain {
  if (chainId === 0x5afe) return chains.sapphire;
  if (chainId === 0x5aff) return chains.sapphireTestnet;
  if (chainId === 31337) return foundry;
  if (chainId === 1337) return localhost;
  throw new Error(`chainId ${chainId} is not supported by the keystore`);
}

function isHex(n: number, v: unknown): v is Hex {
  return typeof v === 'string' && v.length === n * 2 + 1 && /^0x[0-9a-f]+$/i.test(v);
}
const isU256 = (v: unknown): v is Hex => isHex(32, v);
const isAddress = (v: unknown): v is Hex => isHex(20, v);
