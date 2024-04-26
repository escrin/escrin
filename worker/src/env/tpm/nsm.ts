import * as cbor from 'cborg';
import { Hex } from 'viem';

import { ApiError } from '../../rpc.js';

export type NsmBinding = {
  request(cborRequest: Uint8Array): Uint8Array;
};

export class Nsm {
  constructor(private readonly nsm: NsmBinding) {}

  getAttestation(userdata?: Uint8Array): Uint8Array {
    const nonce = this.getRandom(32);
    const { document } = this.communicate<any, { document: Uint8Array }>('Attestation', {
      user_data: userdata,
      nonce,
      public_key: new Uint8Array(),
    });
    if (!(document instanceof Uint8Array)) throw new ApiError(500, 'failed to get attestation');
    return document;
  }

  getRandom(numBytes: number): Uint8Array {
    if (numBytes < 0) throw new ApiError(400, 'negative amount of bytes requested');
    if (numBytes > 128) throw new ApiError(440, 'too many random bytes requested');
    const bytes = new Uint8Array(numBytes);
    let filled = 0;
    while (filled < numBytes) {
      const { random } = this.communicate<void, { random: Uint8Array }>('GetRandom');
      bytes.set(random.subarray(0, numBytes - filled), filled);
      filled += random.length;
    }
    return bytes;
  }

  private communicate<Params, Res>(method: string, params?: Params): Res {
    const req = params !== undefined ? { [method]: params } : method;
    const resBuf = new Uint8Array(this.nsm.request(cbor.encode(req)));
    const res = cbor.decodeFirst(resBuf)[0];
    if (res === undefined) throw new ApiError(500, 'nsm: unexpected empty response');
    if (res.Error) throw new ApiError(500, `nsm: ${res.Error}`);
    if (!(method in res)) throw new ApiError(500, `nsm: missing ${method} response key`);
    return res[method];
  }
}

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
  params: Partial<{ userdata: Hex }>;
  response: { document: Hex };
};

export type RandomBytesRequest = {
  method: 'get-random';
  params: { numBytes: number };
  response: { random: Hex };
};
