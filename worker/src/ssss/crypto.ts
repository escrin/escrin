import { siv } from '@noble/ciphers/aes';
import { p384 } from '@noble/curves/p384';
import { hkdf } from '@noble/hashes/hkdf';
import { sha256 } from '@noble/hashes/sha256';
import { bytesToHex, hexToBytes } from 'viem';

import {
  EncryptedPayload,
  EncResponseFormat,
  Operation,
  SharedCipherDerivationParams,
} from './types.js';

const P384 = 'P-384' as const;

export function generateEphemeralIdentity(): EphemeralIdentity {
  const sk = p384.utils.randomPrivateKey();
  const pk = p384.getPublicKey(sk);
  return { curve: P384, sk, pk };
}

export type EphemeralIdentity = {
  curve: typeof P384;
  sk: Uint8Array;
  pk: Uint8Array;
};

function deriveSharedCipher(
  op: Operation,
  sk: Uint8Array,
  { curve, pk: peerPk, nonce }: SharedCipherDerivationParams<Uint8Array>,
): ReturnType<typeof siv> {
  if (curve.toUpperCase() !== P384) throw new Error(`unsupported remote ${curve} PK`);
  const wrappingKey = hkdf(
    sha256,
    'ssss_ecdh_aes-256-gcm-siv',
    op,
    p384.getSharedSecret(sk, peerPk).slice(1),
    32,
  );
  return siv(wrappingKey, nonce);
}

export function encryptPayload(
  payload: object,
  operation: Operation,
  { pk, sk, curve }: EphemeralIdentity,
  peerPk: Uint8Array,
  recipientKeyId: string,
): EncryptedPayload {
  const nonce = crypto.getRandomValues(new Uint8Array(32));
  const cipher = deriveSharedCipher(operation, sk, {
    curve,
    pk: peerPk,
    nonce,
  });
  return {
    format: {
      [EncResponseFormat.EncEcdhAes256GcmSiv]: {
        curve,
        pk: bytesToHex(pk),
        nonce: bytesToHex(nonce),
        recipient_key_id: recipientKeyId,
      },
    },
    payload: bytesToHex(cipher.encrypt(new TextEncoder().encode(JSON.stringify(payload)))),
  };
}

export async function decryptResponse<T>(
  res: Response,
  operation: Operation,
  { sk }: EphemeralIdentity,
): Promise<T> {
  const { format, payload: data } = await res.json<EncryptedPayload>();

  if (!(EncResponseFormat.EncEcdhAes256GcmSiv in format))
    throw new Error(`ssss: unsupported share response format`);

  const { curve, pk: peerPk, nonce } = format[EncResponseFormat.EncEcdhAes256GcmSiv];
  const cipher = deriveSharedCipher(operation, sk, {
    curve,
    pk: hexToBytes(peerPk),
    nonce: hexToBytes(nonce),
  });
  return JSON.parse(new TextDecoder().decode(cipher.decrypt(hexToBytes(data))));
}
