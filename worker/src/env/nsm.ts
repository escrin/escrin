import * as cbor from 'cborg';

import { ApiError, decodeRequest, wrapFetch } from '../rpc.js';

type Env = { nsm: Nsm; gasKey?: string };

type Nsm = {
  request(cborRequest: Uint8Array): Uint8Array;
};

export default new (class {
  public readonly fetch = wrapFetch(async (req, env: Env) => {
    const requester = req.headers.get('x-caller-id');
    if (!requester) throw new ApiError(500, 'escrin-runner did not set x-caller-id header');

    const { method, params } = await decodeRequest(req);

    if (method === ('get-attestation' satisfies AttestationRequest['method'])) {
      if ('userdata' in params && !(params.userdata instanceof Uint8Array))
        throw new ApiError(400, 'invalid userdata');
      return { document: this.#getAttestation(env.nsm, params.userdata as Uint8Array) };
    }

    if (method === ('get-random' satisfies GetRandomRequest['method'])) {
      if (!('bytes' in params) || typeof params.numBytes !== 'number')
        throw new ApiError(400, 'missing or invalid count');
      return { document: this.#getRandom(env.nsm, params.numBytes) };
    }

    throw new ApiError(404, `unknown method: ${method}`);
  });

  #getAttestation(nsm: Nsm, userdata?: Uint8Array): Uint8Array {
    const nonce = this.#getRandom(nsm, 32);
    const { document } = this.#nsm(nsm, {
      Attestation: {
        user_data: userdata,
        nonce,
        public_key: new Uint8Array(),
      },
    });
    if (!(document instanceof Uint8Array)) throw new ApiError(500, 'failed to get attestation');
    return document;
  }

  #getRandom(nsm: Nsm, numBytes: number): Uint8Array {
    if (numBytes < 0) throw new ApiError(400, 'negative amount of bytes requested');
    if (numBytes > 128) throw new ApiError(440, 'too many random bytes requested');
    const bytes = new Uint8Array(numBytes);
    let filled = 0;
    while (filled < numBytes) {
      const { random } = this.#nsm(nsm, { GetRandom: {} });
      bytes.set(random.subarray(0, numBytes - filled), filled);
    }
    return bytes;
  }

  #nsm(nsm: Nsm, params: NsmRequest): any {
    return cbor.decode(nsm.request(cbor.encode(params)));
  }
})();

export type NsmRequest = any;

export type DescribePcrRequest = {
  method: 'describe-pcr';
  params: { index: number };
  response: { lock: boolean; data: Uint8Array };
};

export type ExtendPcrRequest = {
  method: 'extend-pcr';
  params: { data: Uint8Array };
  response: { data: Uint8Array };
};

export type LockPcrRequest = {
  method: 'lock-pcr';
  params: { index: number };
  response: void;
};

export type LockPcrsRequest = {
  method: 'lock-pcrs';
  params: { range: number };
  response: void;
};

export type DescrbeNsmRequest = {
  method: 'describe-nsm';
  params: void;
  response: {
    version: { major: number; minor: number; patch: number };
    moduleId: string;
    maxPcrs: number;
    lockedPcrs: Set<number>;
    digest: 'sha256' | 'sha384' | 'sha512';
  };
};

export type AttestationRequest = {
  method: 'get-attestation';
  params: Partial<{ userdata: Uint8Array }>;
  response: { document: Uint8Array };
};

export type GetRandomRequest = {
  method: 'get-random';
  params: { numBytes: number };
  response: { random: Uint8Array };
  nsmParams: void;
};
