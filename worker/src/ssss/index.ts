import * as secp256k1 from '@noble/curves/secp256k1';
import { StandardMerkleTree } from '@openzeppelin/merkle-tree';
import type { SetRequired } from 'type-fest';
import {
  Address,
  Hex,
  PrivateKeyAccount,
  PublicClient,
  Signature,
  Transport,
  WalletClient,
  bytesToHex,
  encodeAbiParameters,
  hexToBytes,
  signatureToCompactSignature,
} from 'viem';
import { Account } from 'viem/accounts';
import { Chain } from 'viem/chains';

import { SsssPermitter as SsssPermitterAbi } from '@escrin/evm/abi';

import { SsssAcquireIdentityParams, SsssSecretStoreParams } from '../env/iam/types.js';
import * as ssssCrypto from './crypto.js';
import {
  AcquireIdentityRequest,
  IdentityResponse,
  Operation,
  Permit,
  SecretShare,
} from './types.js';
import { escrin1, fetchAll, throwErrorResponse } from './utils.js';
import * as vss from './vss.js';

const VSS = new vss.Pedersen(secp256k1.secp256k1, secp256k1.hashToCurve);

export async function getSecret(
  params: SsssSecretStoreParams,
  requesterAccount: PrivateKeyAccount,
): Promise<Hex> {
  const {
    network: { chainId },
    identity: { registry, id: identityIdHex },
    ssss,
    secretName,
    secretVersion,
  } = params;

  const eid = ssssCrypto.generateEphemeralIdentity();

  type ShareResponse = {
    share: { index: number; secret: Hex; blinder: Hex };
    commitments: Hex[];
  };

  const results: ShareResponse[] = await fetchAll(
    ssss.sssss.map((s) => (typeof s === 'string' ? s : s.url)),
    async (ssssUrl) => {
      const url = `${ssssUrl}/shares/${secretName}/${chainId}/${registry}/${identityIdHex}?version=${secretVersion}`;
      return {
        url,
        method: 'GET',
        headers: {
          ...(await escrin1(requesterAccount, 'GET', url)),
          'requester-pk': bytesToHex(eid.pk),
        },
      };
    },
    async (res) => {
      if (!res.ok) await throwErrorResponse(res);
      return ssssCrypto.decryptResponse(res, Operation.GetShare, eid);
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
  params: SsssSecretStoreParams,
  requesterAccount: PrivateKeyAccount,
): Promise<Hex> {
  const {
    network: { chainId },
    identity: { registry, id: identityIdHex },
    ssss,
    secretName,
    secretVersion,
  } = params;

  const eid = ssssCrypto.generateEphemeralIdentity();

  const ssssPublicKeys = await fetchAll(
    ssss.sssss,
    async (ssssUrl) => {
      return {
        url: `${ssssUrl}/identity`,
        method: 'GET',
      };
    },
    async (res) => {
      if (!res.ok) await throwErrorResponse(res);
      const { ephemeral } = await res.json<IdentityResponse>();
      return ephemeral;
    },
  );

  const { secret, shares, commitments } = VSS.generate(ssss.quorum, ssss.sssss.length);

  const encReqs = shares.map(({ index, secret, blinder }, i) => {
    const { pk: peerPk, key_id: recipientKeyId } = ssssPublicKeys[i];
    return ssssCrypto.encryptPayload(
      {
        meta: {
          index,
          commitments,
        },
        share: secret,
        blinder,
      } satisfies SecretShare,
      Operation.DealShares,
      eid,
      hexToBytes(peerPk),
      recipientKeyId,
    );
  });

  await fetchAll(
    ssss.sssss,
    async (ssssUrl, i) => {
      const url = `${ssssUrl}/shares/${secretName}/${chainId}/${registry}/${identityIdHex}?version=${secretVersion}`;
      return {
        url,
        method: 'POST',
        headers: {
          'content-type': 'application/json',
          ...(await escrin1(requesterAccount, 'POST', url)),
        },
        body: JSON.stringify(encReqs[i]),
      };
    },
    async (res) => {
      if (!res.ok) await throwErrorResponse(res);
    },
  );

  return secret;
}

export async function acquireIdentity(
  params: SetRequired<SsssAcquireIdentityParams, 'permitter' | 'recipient' | 'permitTtl'>,
  publicClient: PublicClient<Transport, Chain>,
  walletClient: WalletClient<Transport, Chain, Account>,
): Promise<{ duration: number }> {
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

  const signersP = [];
  for (const s of ssss.sssss) {
    signersP.push(
      typeof s === 'string'
        ? fetch(`${s}/identity`).then(async (res) => {
            if (!res.ok) await throwErrorResponse(res);
            return (await res.json<IdentityResponse>()).signer;
          })
        : s.signer,
    );
  }
  const signers = await Promise.all(signersP);

  const nonce = params.nonce ?? bytesToHex(crypto.getRandomValues(new Uint8Array(32)));
  const pk = params.pk ?? '0x';

  const baseBlock = Number(await publicClient.getBlockNumber());

  const permits = await fetchAll(
    ssss.sssss,
    async (ssssUrl) => {
      const url = `${ssssUrl}/permits/${chainId}/${registry}/${identityIdHex}`;
      return {
        url,
        method: 'POST',
        headers: {
          'content-type': 'application/json',
          ...(await escrin1(walletClient, 'GET', url)),
        },
        body: JSON.stringify({
          permitter,
          recipient,
          base_block: baseBlock,
          duration: permitTtl,
          context,
          authorization,
        } satisfies AcquireIdentityRequest),
      };
    },
    async (res) => {
      if (!res.ok) await throwErrorResponse(res);
      return res.json<{
        permit: Permit;
        signer: Address;
        signature: Signature;
      }>();
    },
  );

  if (permits.length < ssss.quorum) throw new Error('quorum not reached');

  const tree = StandardMerkleTree.of(
    signers.map((s) => [s]),
    ['address'],
  );
  const proof = tree.getMultiProof(permits.map((p) => [p.signer]));

  const permitsBySigner = new Map<Address, Signature>();
  for (const permit of permits) {
    permitsBySigner.set(permit.signer, permit.signature);
  }
  const signatures = proof.leaves.map(([signer]) => {
    const permit = permitsBySigner.get(signer as `0x${string}`) as Signature;
    const { r, yParityAndS } = signatureToCompactSignature(permit);
    return [signer, r, yParityAndS];
  });

  const hash = await walletClient.writeContract({
    address: permitter,
    abi: SsssPermitterAbi,
    functionName: 'acquireIdentity',
    args: [
      identityIdHex,
      walletClient.account.address,
      BigInt(permitTtl),
      encodeAbiParameters(
        [
          { name: 'threshold', type: 'uint256' },
          { name: 'nonce', type: 'bytes' },
          { name: 'pk', type: 'bytes' },
          { name: 'baseBlock', type: 'uin256' },
        ],
        [BigInt(ssss.quorum), nonce, pk, baseBlock],
      ),
      encodeAbiParameters(
        [
          { name: 'proof', type: 'bytes32[]' },
          { name: 'proofFlags', type: 'bool[]' },
          { name: 'signatures', type: '(address, bytes32, bytes32)[]' },
        ],
        [proof.proof as Array<`0x${string}`>, proof.proofFlags, signatures],
      ),
    ],
  });

  const { status } = await publicClient.waitForTransactionReceipt({ hash });
  if (status !== 'success') throw new Error(`failed to acquire identity in ${hash}`);

  return { duration: permits[0].permit.duration };
}
