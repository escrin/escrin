import { siv } from '@noble/ciphers/aes';
import { p384 } from '@noble/curves/p384';
import { hkdf } from '@noble/hashes/hkdf';
import { sha256 } from '@noble/hashes/sha256';
import { Hex, bytesToHex, hexToBytes } from 'viem';

export function generateEphemeralIdentity(): { sk: Uint8Array; pk: Uint8Array } {
  const sk = p384.utils.randomPrivateKey();
  const pk = p384.getPublicKey(sk);
  return { sk, pk };
}

export enum Operation {
  GetShare = 'get-share',
  DealShares = 'deal-shares',
}

export function deriveSharedCipher(
  op: Operation,
  sk: Hex | Uint8Array,
  peerPk: JsonWebKey,
  nonce: Hex | Uint8Array,
): ReturnType<typeof siv> {
  if (peerPk.crv !== 'P-384') throw new Error(`unsupported remote ${peerPk.crv} PK`);
  const wrappingKey = hkdf(
    sha256,
    'ssss_ecdh_aes-256-gcm-siv',
    op,
    p384
      .getSharedSecret(typeof sk === 'string' ? hexToBytes(sk) : sk, ecJwkToSec1(peerPk))
      .slice(1),
    32,
  );
  return siv(wrappingKey, typeof nonce === 'string' ? hexToBytes(nonce) : nonce);
}

export function ecJwkToSec1(jwk: JsonWebKey): Hex {
  if (jwk.kty !== 'EC') throw new Error('cannot convert non-EC JWK to SEC1');
  if (!jwk.crv || jwk.crv.startsWith('P-')) throw new Error(`unsupported JWK curve: ${jwk.crv}`);
  const curveSize = parseInt(jwk.crv.substring(2), 10) / 8;
  const sec1 = new Uint8Array(2 * curveSize + 1);
  sec1[0] = 4; // uncompressed
  decodeBase64Url(jwk.x!, sec1.subarray(1, 1 + curveSize));
  decodeBase64Url(jwk.y!, sec1.subarray(1 + curveSize));
  return bytesToHex(sec1);
}

export function decodeBase64Url(base64Url: string, arr: Uint8Array): void {
  const padding = '='.repeat((4 - (base64Url.length % 4)) % 4);
  const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/') + padding;
  const rawData = atob(base64);
  for (let i = 0; i < rawData.length; i++) {
    arr[i] = rawData.charCodeAt(i);
  }
}
