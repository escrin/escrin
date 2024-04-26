import * as secp256k1 from '@noble/curves/secp256k1';
import { SetRequired } from 'type-fest';
import {
  Address,
  Hash,
  Hex,
  PrivateKeyAccount,
  PublicClient,
  Transport,
  WalletClient,
  bytesToHex,
  hexToBytes,
} from 'viem';
import { Account } from 'viem/accounts';
import { Chain } from 'viem/chains';

import { ExperimentalSsssHub as SsssHubAbi } from '@escrin/evm/abi';

import { AcquireIdentityParams, EvmKeyStoreParams } from '../env/iam/types.js';

import * as ssssCrypto from './crypto.js';
import { escrin1, fetchAll, throwErrorResponse } from './utils.js';
import * as vss from './vss.js';

const VSS = new vss.Pedersen(secp256k1.secp256k1, secp256k1.hashToCurve);

export async function getSecretVersion(
  publicClient: PublicClient<Transport, Chain>,
  ssssHub: Address,
  identityId: Hash,
  secretName: string,
): Promise<number> {
  return Number(
    await publicClient.readContract({
      address: ssssHub,
      abi: SsssHubAbi,
      functionName: 'versions',
      args: [identityId, secretName],
    }),
  );
}

export async function getSecret(
  name: string,
  version: number,
  params: SetRequired<EvmKeyStoreParams, 'ssss'>,
  requesterAccount: PrivateKeyAccount,
): Promise<Hex> {
  const {
    network: { chainId },
    identity: { registry, id: identityIdHex },
    ssss,
  } = params;

  const { sk: esk, pk: epk } = ssssCrypto.generateEphemeralIdentity();

  type EncSsssResponse = {
    format: { [EncResponseFormat.EncEcdhAes256GcmSiv]: { pk: JsonWebKey; nonce: Hex } };
    data: Hex;
  };

  enum EncResponseFormat {
    EncEcdhAes256GcmSiv = 'enc-ecdh-aes-256-gcm-siv',
  }

  type ShareResponse = {
    share: { index: number; secret: Hex; blinder: Hex };
    commitments: Hex[];
  };

  const results: ShareResponse[] = await fetchAll(
    ssss.urls,
    async (ssssUrl) => {
      const url = `${ssssUrl}/shares/${name}/${chainId}/${registry}/${identityIdHex}?version=${version}`;
      return {
        url,
        method: 'GET',
        headers: {
          ...(await escrin1(requesterAccount, 'GET', url)),
          'requester-pk': bytesToHex(epk),
        },
      };
    },
    async (res) => {
      if (!res.ok) await throwErrorResponse(res);
      const { format, data } = await res.json<EncSsssResponse>();

      if (!(EncResponseFormat.EncEcdhAes256GcmSiv in format))
        throw new Error(`ssss: unsupported share response format`);

      const { pk: ppk, nonce } = format[EncResponseFormat.EncEcdhAes256GcmSiv];
      const cipher = ssssCrypto.deriveSharedCipher(ssssCrypto.Operation.GetShare, esk, ppk, nonce);
      return JSON.parse(new TextDecoder().decode(cipher.decrypt(hexToBytes(data))));
    },
  );

  const shares = [];
  const ccount = new Map<string, number>();
  for (const { share, commitments } of results) {
    const cstr = JSON.stringify(commitments);
    ccount.set(cstr, (ccount.get(cstr) ?? 0) + 1);
    shares.push(share);
  }
  let cmax = 0;
  let commitments;
  // The commitments aren't persisted, so we do consensus over the responses.
  // If there were a trustworthy web3 gateway or light client, the commitments could be looked up from the tx.
  for (const [cstr, count] of ccount.entries()) {
    if (count <= cmax) continue;
    commitments = JSON.parse(cstr);
    cmax = count;
  }
  if (cmax < ssss.quorum) throw new Error('sss: unable to get commitments from SSSSs');

  return VSS.reconstruct(shares, commitments);
}

export async function dealNewSecret(
  secretName: string,
  params: SetRequired<EvmKeyStoreParams, 'ssss'>,
  publicClient: PublicClient<Transport, Chain>,
  walletClient: WalletClient<Transport, Chain, Account>,
): Promise<Hex> {
  const { identity, ssss } = params;

  const { sk: esk, pk: epk } = ssssCrypto.generateEphemeralIdentity();

  const nonce = crypto.getRandomValues(new Uint8Array(32));

  const ssssCiphers = await fetchAll(
    ssss.urls,
    async (ssssUrl) => {
      return {
        url: `${ssssUrl}/identity`,
        method: 'GET',
      };
    },
    async (res) => {
      if (!res.ok) await throwErrorResponse(res);
      const { ephemeral: pk } = await res.json<{ ephemeral: JsonWebKey }>();
      if (pk.crv !== 'P-384') throw new Error(`unsupported remote ${pk.crv} PK`);
      return ssssCrypto.deriveSharedCipher(ssssCrypto.Operation.DealShares, esk, pk, nonce);
    },
  );

  const { secret, shares: plainShares, commitments } = VSS.generate(ssss.quorum, ssss.urls.length);
  const encShares: Hex[] = [];
  if (ssssCiphers.length !== ssss.urls.length) throw new Error('SSSS and shares mismatch');
  for (let i = 0; i < ssssCiphers.length; i++) {
    const { secret, blinder } = plainShares[i];
    const secretBytes = hexToBytes(secret);
    const blinderBytes = hexToBytes(blinder);
    const shareBytes = new Uint8Array(secretBytes.length + blinderBytes.length);
    shareBytes.set(shareBytes);
    shareBytes.set(blinderBytes, shareBytes.length);
    const cipher = ssssCiphers[i];
    encShares.push(bytesToHex(cipher.encrypt(shareBytes)));
  }

  const hash = await walletClient.writeContract({
    address: ssss.hub,
    abi: SsssHubAbi,
    functionName: 'dealShares',
    args: [
      identity.id,
      secretName,
      1n, // version
      bytesToHex(epk),
      bytesToHex(nonce),
      encShares,
      commitments,
      '0x', // userdata
    ],
  });
  const { status } = await publicClient.waitForTransactionReceipt({ hash, confirmations: 2 });
  if (status !== 'success') throw new Error(`failed to deal shares in ${hash}`);

  return secret;
}

export async function acquireIdentity(
  params: SetRequired<AcquireIdentityParams, 'ssss' | 'recipient' | 'permitTtl'>,
): Promise<{ grants: number; optimisticGrants: number }> {
  const {
    network: { chainId },
    identity: { registry, id: identityIdHex },
    permitter,
    permitTtl,
    recipient,
    context,
    authorization,
    ssss,
  } = params;

  const grants = await fetchAll(
    ssss.urls,
    (ssssUrl) => {
      return {
        url: `${ssssUrl}/permits/${chainId}/${registry}/${identityIdHex}`,
        method: 'POST',
        headers: {
          'content-type': 'application/json',
        },
        body: JSON.stringify({
          recipient,
          permitter,
          duration: permitTtl,
          authorization,
          context,
        } satisfies {
          recipient: Address;
          duration: number;
          permitter?: Address;
          authorization?: Hex;
          context?: Hex;
        }),
      };
    },
    async (res) => {
      if (res.status === 201) return true;
      if (res.status === 202) return false;
      await throwErrorResponse(res);
    },
  );

  if (grants.length < params.ssss.quorum)
    throw new Error('ssss: failed to acquire identity as quorum was not reached');

  let optimisticGrants = 0;
  for (const issued of grants) {
    if (issued) optimisticGrants++;
  }

  return {
    grants: grants.length,
    optimisticGrants,
  };
}
