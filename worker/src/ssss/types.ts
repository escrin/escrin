import { Address, Hex } from 'viem';

export enum EncResponseFormat {
  EncEcdhAes256GcmSiv = 'enc-ecdh-aes-256-gcm-siv',
}

export type EncryptedPayload = {
  format: { [EncResponseFormat.EncEcdhAes256GcmSiv]: SharedCipherDerivationParams<Hex> };
  payload: Hex;
};

export enum Operation {
  GetShare = 'get-share',
  DealShares = 'deal-shares',
}

export type SharedCipherDerivationParams<T = Hex | Uint8Array> = {
  curve: 'P-384';
  pk: T;
  nonce: T;
  recipient_key_id?: string;
};

export type IdentityResponse = {
  ephemeral: EphemeralKey;
  signer: Address;
};

export type EphemeralKey = {
  key_id: string;
  pk: Hex;
  expiry: number;
};

export type SecretShare = {
  meta: {
    index: number;
    commitments: Hex[];
  };
  share: Hex;
  blinder: Hex;
};

type AcqRelIdentityRequest = {
  permitter: Address;
  recipient: Address;
  base_block: number;
  authorization?: Hex;
  context?: Hex;
};

export type AcquireIdentityRequest = AcqRelIdentityRequest & { duration: number };

export type Permit = {
  registry: Address;
  identity: Hex;
  recipient: Address;
  grant: boolean;
  duration: number;
  nonce: Hex;
  pk: Hex;
  baseBlock: bigint;
};
