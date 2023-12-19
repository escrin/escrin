import { ApiError, decodeRequest, wrapFetch } from '../../rpc.js';

import { Nsm, NsmBinding } from './nsm.js';
import { AttestationRequest, GetRandomRequest } from './types.js';

type Env = { nsm?: NsmBinding; gasKey?: string };

export default new (class {
  public readonly fetch = wrapFetch(async (req, env: Env) => {
    const requester = req.headers.get('x-caller-id');
    if (!requester) throw new ApiError(500, 'escrin-runner did not set x-caller-id header');

    const { method, params } = await decodeRequest(req);

    const backend = getBackend(env);

    if (method === ('get-attestation' satisfies AttestationRequest['method'])) {
      if ('userdata' in params && !(params.userdata instanceof Uint8Array))
        throw new ApiError(400, 'invalid userdata');
      return {
        document: backend.getAttestation(params.userdata as Uint8Array),
      } as AttestationRequest['response'];
    }

    if (method === ('get-random' satisfies GetRandomRequest['method'])) {
      if (!('bytes' in params) || typeof params.numBytes !== 'number')
        throw new ApiError(400, 'missing or invalid count');
      return {
        random: backend.getRandom(params.numBytes),
      } as GetRandomRequest['response'];
    }

    throw new ApiError(404, `unknown method: ${method}`);
  });
})();

function getBackend(env: Env): {
  getAttestation(userdata: Uint8Array): Uint8Array;
  getRandom(numBytes: number): Uint8Array;
} {
  if (env.nsm) return new Nsm(env.nsm);
  throw new ApiError(500, 'no TPM backend available');
}
