import { Cacheable, cacheability } from '../env/index.js';
import { INIT_SAPPHIRE, INIT_SAPPHIRE_TESTNET, getKeySapphire } from '../env/keystore/sapphire.js';
import { ApiError, decodeRequest, encodeBase64Bytes, wrapFetch } from '../rpc.js';

export default new (class {
  public readonly fetch = wrapFetch(async (req, env: { gasKey?: string }) => {
    const requester = req.headers.get('host');
    const { method, params } = await decodeRequest(req);

    if (method === 'get-key') {
      if (!requester) throw new ApiError(500, 'escrin-runner did not set host header');
      const { keyStore, keyId, proof } = params;
      if (!env.gasKey) throw new ApiError(500, 'escrin-runner not configured: missing `gas-key`');
      const key = await getKey(requester, keyStore, keyId, proof, env.gasKey);
      return encodeBase64Bytes(key);
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
): Promise<Uint8Array> {
  const cacheKey: KeyCacheKey = `${requester}.${keyStore}.${keyId}`;
  const cachedKey = KEY_CACHE[cacheKey];
  if (cachedKey) {
    if (cachedKey[cacheability].expiry > new Date()) return cachedKey;
    delete KEY_CACHE[cacheKey];
  }

  if (!/^sapphire(-testnet)?$/.test(keyStore))
    throw new ApiError(404, `unknown key store: ${keyStore}`);
  if (keyId !== 'omni') throw new ApiError(404, `unknown key id: ${keyId}`);

  const key = await getKeySapphire(keyId, proof, gasKey, {
    init: keyStore === 'sapphire' ? INIT_SAPPHIRE : INIT_SAPPHIRE_TESTNET,
  });

  if (key.hasOwnProperty(cacheability)) {
    KEY_CACHE[cacheKey] = key;
  }
  return key;
}
