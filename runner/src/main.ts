import { Body, CryptoKey, Request, ExecutionContext } from '@cloudflare/workers-types/experimental';
import deoxysii from '@oasisprotocol/deoxysii';
import * as sapphire from '@oasisprotocol/sapphire-paratime';
import canonicalize from 'canonicalize';
import { ethers } from 'ethers';
import createKeccakHash from 'keccak';

import { AttestationToken, AttestationTokenFactory, Lockbox, LockboxFactory } from '@escrin/evm';

import { TaskService } from './task-service';
import { decode, encode, memoizeAsync } from './utils';

type Registration = AttestationToken.RegistrationStruct;

export type InitOpts = {
  web3GatewayUrl: string;
  attokAddr: string;
  lockboxAddr: string;
  debug?: Partial<{
    nowrap: boolean;
  }>;
};

export type Box = unknown;

const LATEST_KEY_ID = 1;

// export class ESM implements ESM {
//   private provider: ethers.providers.Provider;
//   private attok: AttestationToken;
//   private lockbox: Lockbox;
//   private gasWallet: ethers.Wallet;
//   private localWallet: ethers.Wallet;

//   constructor(key: CryptoKey, gasKey: string) {
//     this.provider = new ethers.providers.JsonRpcProvider(opts.web3GatewayUrl);
//     this.gasWallet = new ethers.Wallet(gasKey).connect(this.provider);
//     const localWallet = new ethers.Wallet(gasKey).connect(this.provider);
//     // const localWallet = ethers.Wallet.createRandom().connect(this.provider);
//     this.localWallet = opts.debug?.nowrap ? localWallet : sapphire.wrap(localWallet);
//     this.attok = AttestationTokenFactory.connect(opts.attokAddr, this.gasWallet);
//     this.lockbox = LockboxFactory.connect(opts.lockboxAddr, this.localWallet);
//   }

//   private fetchKeySapphire = memoizeAsync(async () => {
//     const oneHourFromNow = Math.floor(Date.now() / 1000) + 60 * 60;
//     let currentBlock = await this.provider.getBlock('latest');
//     const prevBlock = await this.provider.getBlock(currentBlock.number - 1);
//     const registration: Registration = {
//       baseBlockHash: prevBlock.hash,
//       baseBlockNumber: prevBlock.number,
//       expiry: oneHourFromNow,
//       registrant: this.localWallet.address,
//       tokenExpiry: oneHourFromNow,
//     };
//     const quote = await mockQuote(registration);
//     const tcbId = await sendAttestation(this.attok.connect(this.localWallet), quote, registration);
//     return getOrCreateKey(this.lockbox, this.gasWallet, tcbId);
//   });

//   private getCipher = memoizeAsync(async (keyId: number) => {
//     let key;
//     if (keyId === 0) key = Buffer.alloc(deoxysii.KeySize, 42);
//     else if (keyId === 1) key = await this.deriveKey('nftrout/encryption/nfts');
//     else throw new Error(`unknown key: ${keyId}`);
//     return new deoxysii.AEAD(key);
//   });

//   public async encrypt(data: Uint8Array, binding?: unknown): Promise<Box> {
//     const keyId = LATEST_KEY_ID;
//     const cipher = await this.getCipher(keyId);
//     const nonce = new Uint8Array(deoxysii.NonceSize);
//     crypto.getRandomValues(nonce);
//     return {
//       keyId,
//       nonce: encode(nonce),
//       data: encode(cipher.encrypt(nonce, data, bind(binding))),
//     } as Box;
//   }

//   public async decrypt(box: Box, binding?: Uint8Array): Promise<Uint8Array> {
//     const { keyId, nonce, data } = unbox(box);
//     const cipher = await this.getCipher(keyId);
//     return cipher.decrypt(nonce, data, binding);
//   }

//   public async deriveKey(keyId: string, length = 32): Promise<CryptoKey> {
//     crypto.subtle.deriveKey('SHA512-256', this.key, { name: 'hmac', hash: 'SHA-512' }, true, [
//       'unwrapKey',
//     ]);
//     return new Promise(async (resolve, reject) => {
//       const ikm = await this.fetchKeySapphire();
//       hkdf('sha512-256', ikm, '', keyId, length, (err, key) => {
//         if (err) reject(err);
//         else resolve(Buffer.from(key));
//       });
//     });
//     const hmacParams = {
//       name: 'hmac',
//       hash: 'SHA-512-256',
//     };
//     const kdfParams = {
//       info: new TextEncoder().encode(`nftrout/entropy/${chainId}/${tokenId}`),
//       ...hmacParams,
//     };
//     const seedKey = await crypto.subtle.deriveKey(kdfParams, env.agentKey, hmacParams, true, []);
//     const seedMaterial = await crypto.subtle.exportKey('raw', seedKey);
//   }
// }

// function unbox(box: Box): { keyId: string; nonce: Uint8Array; data: Uint8Array } {
//   if (box === null || typeof box !== 'object') throw new NotBoxError('box not object');
//   if (!('keyId' in box) || typeof box.keyId !== 'string') throw new NotBoxError('keyId not string');
//   const keyId = box.keyId;
//   if (!('nonce' in box) || typeof box.nonce !== 'string') throw new NotBoxError('nonce not string');
//   let nonce;
//   try {
//     nonce = decode(box.nonce);
//   } catch (e: any) {
//     throw new NotBoxError(`nonce wrongly encoded: ${e?.message ?? e}`);
//   }
//   if (!('data' in box) || typeof box.data !== 'string') throw new NotBoxError('data not string');
//   let data;
//   try {
//     data = decode(box.data);
//   } catch (e: any) {
//     throw new NotBoxError(`data wrongly encoded: ${e?.message ?? e}`);
//   }
//   return { keyId, nonce, data };
// }

// class NotBoxError extends Error {
//   constructor(message: string) {
//     super(`not a box: ${message}`);
//     this.name = new.target.name;
//     Object.setPrototypeOf(this, new.target.prototype);
//   }
// }

// function bind(prop: unknown): Uint8Array | undefined {
//   if (prop === undefined) return undefined;
//   const c = canonicalize(prop);
//   if (c === undefined) return c;
//   return Buffer.from(c);
// }


class ApiError extends Error {
  constructor(public readonly statusCode: number, message: string) {
    super(message);
    this.name = new.target.name;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

type AgentSpec = {
  script: string;
  type?: 'classic' | 'module';
  name?: string;
  schedule?: string;
  args?: object;
};

async function parseRequestBody(contentType: string | null, body: Body): Promise<AgentSpec> {
  if (!contentType || contentType === 'application/json') {
    try {
      return await body.json();
    } catch {
      throw new ApiError(400, 'the request body could not be parsed as JSON');
    }
  }
  if (
    contentType === 'application/x-www-form-urlencoded' ||
    contentType.startsWith('multipart/form-data')
  ) {
    try {
      const fd = await body.formData();
      return Object.fromEntries(fd.entries()) as any;
    } catch {
      throw new ApiError(400, 'the request body could not be parsed as form data');
    }
  }
  throw new ApiError(400, `unsupported content type: ${contentType}`);
}

export class TaskManager {
  private services: TaskService[] = [];

  async fetch(req: Request, _env: unknown, _ctx: ExecutionContext) {
    let spec: AgentSpec;
    try {
      spec = await parseRequestBody(req.headers.get('content-type'), req);
    } catch (e: unknown) {
      if (e instanceof ApiError) {
        return new ErrorResponse(e.statusCode, e.message);
      }
      throw e;
    }
    const scriptUrl = URL.createObjectURL(new Blob([spec.script], { type: 'application/json' }));
    const worker = new Worker(scriptUrl, { type: spec.type, name: spec.name });
    const svc = new TaskService(worker);
    URL.revokeObjectURL(scriptUrl);
    if (svc.terminated) {
      return new Response('', { status: 400 });
    }
    if (spec.schedule) {
      if (spec.schedule !== '*/5 * * * *') {
        // throw new ApiError(400, 'unsupported cron spec')
        return new Response('', { status: 501 });
      }
      console.log('schedule');
      svc.schedule(5 * 60 * 1000);
    }
    this.services.push(svc);
    return new Response('', { status: 201 });
  }
}

export default new TaskManager();

class ErrorResponse extends Response {
  constructor(status: number, message: string) {
    const resp = JSON.stringify({
      message,
    });
    super(resp, {
      status,
      headers: {
        'content-type': 'application/json',
      },
    });
  }
}
