import { Cacheable, cacheability } from '../env/index.js';
import * as sapphireKeystore from '../env/keystore/sapphire.js';
import { ApiError, decodeRequest, encodeBase64Bytes, wrapFetch } from '../rpc.js';

export default new (class {
  public readonly fetch = wrapFetch(async (req, env: { gasKey?: string }) => {
    const requester = req.headers.get('x-caller-id');
    if (!requester) throw new ApiError(500, 'escrin-runner did not set x-caller-id header');
    const { method, params } = await decodeRequest(req);

    if (method === 'get-key') {
      const { keyStore, keyId, proof, opts } = params;
      if (!env.gasKey) throw new ApiError(500, 'escrin-runner not configured: missing `gas-key`');
      const key = await getKey(requester, keyStore, keyId, proof, env.gasKey, opts);
      return { key: encodeBase64Bytes(key) };
    }

    throw new ApiError(404, `unknown method: ${method}`);
  });
})();

type Requester = string;
type KeyStore = string;
type KeyId = string;

type KeyCacheKey = `${Requester}.${KeyStore}.${KeyId}`;
const KEY_CACHE: Record<KeyCacheKey, Cacheable<Uint8Array>> = {};

async function getKey(
  requester: string,
  keyStore: string,
  keyId: string,
  proof: string,
  gasKey: string,
  opts?: Record<string, unknown>,
): Promise<Uint8Array> {
  const cacheKey: KeyCacheKey = `${requester}.${keyStore}.${keyId}`;
  const cachedKey = KEY_CACHE[cacheKey];
  if (cachedKey) {
    if (cachedKey[cacheability].expiry > new Date()) return cachedKey;
    delete KEY_CACHE[cacheKey];
  }

  if (!/^sapphire-(local|(main|test)net)$/.test(keyStore))
    throw new ApiError(404, `unknown key store: ${keyStore}`);
  if (keyId !== 'omni') throw new ApiError(404, `unknown key id: ${keyId}`);

  let sapphireGetKeyOpts =
    opts !== undefined
      ? (opts as any) /* TODO */
      : keyStore === 'sapphire-local'
      ? sapphireKeystore.INIT_LOCAL
      : keyStore === 'sapphire-mainnet'
      ? sapphireKeystore.INIT_MAINNET
      : sapphireKeystore.INIT_TESTNET;
  const key = await sapphireKeystore.getKey(keyId, proof, gasKey, sapphireGetKeyOpts);

  if (key.hasOwnProperty(cacheability)) {
    KEY_CACHE[cacheKey] = key;
  }
  return key;
}
