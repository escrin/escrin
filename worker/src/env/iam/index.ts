import { isHash } from 'viem';

import { ApiError, decodeRequest, wrapFetch } from '../../rpc.js';

import * as identity from './identity.js';
import { handleGetSecret } from './key.js';
import * as types from './types.js';

export type Env = Partial<{
  gasKey?: string;
}>;

export default new (class {
  public readonly fetch = wrapFetch(async (req, env: Env) => {
    const requester = req.headers.get('x-caller-id');
    if (!requester) throw new ApiError(500, 'escrin-runner did not set x-caller-id header');
    const { method, params } = await decodeRequest(req);

    if (method === ('get-account' satisfies types.GetAccountRequest['method'])) {
      return identity.handleGetAccount(requester, types.parseGetAccountParams(params));
    }

    if (method === ('acquire-identity' satisfies types.AcquireIdentityRequest['method'])) {
      const gasKey = env.gasKey ?? (typeof params?.gasKey === 'string' ? params.gasKey : undefined);
      if (!gasKey || !isHash(gasKey)) throw new ApiError(500, 'gas key not configured');
      return identity.handleAcquireIdentity(
        gasKey,
        requester,
        types.parseAcquireIdentityParams(params),
      );
    }

    if (method === ('get-secret' satisfies types.GetSecretRequest['method'])) {
      return handleGetSecret(requester, types.parseGetSecretParams(params));
    }

    throw new ApiError(404, `unknown method: ${method}`);
  });
})();
