import { Address, BlockNotFoundError, Hash, TransactionNotFoundError, hexToBigInt } from 'viem';

import {
  IIdentityRegistry as IdentityRegistryAbi,
  IPermitter as PermitterAbi,
} from '@escrin/evm/abi';

import { ApiError } from '../../rpc.js';

import { allocateAccount } from './account.js';
import { getPublicClient, getWalletClient } from './chains.js';
import * as types from './types.js';

export async function handleGetAccount(
  requesterService: string,
  { id }: types.GetAccountParams,
): Promise<types.GetAccountRequest['response']> {
  if (id !== 'ephemeral-account') throw new Error(`unknown account: ${id}`);
  const { address } = allocateAccount(requesterService);
  return { address };
}

export async function handleAcquireIdentity(
  gasKey: Hash,
  requesterService: string,
  opts: types.AcquireIdentityParams,
): Promise<void> {
  if (opts.sssss) return acquireIdentitySsss(gasKey, requesterService, opts);
  return acquireIdentitySapphire(gasKey, requesterService, opts);
}

async function acquireIdentitySsss(
  _gasKey: Hash,
  _requesterService: string,
  opts: types.AcquireIdentityParams,
): Promise<void> {
  const {
    network: { chainId },
    identity: { registry, id: identityIdHex },
    permitter,
    permitTtl,
    recipient,
    context,
    authorization,
    sssss,
  } = opts;

  if (sssss === undefined) throw new Error('sssss not provided');

  const abort = new AbortController();
  const fetchTimeout = setTimeout(() => abort.abort(), 60 * 1000);
  const permitResults = await Promise.allSettled(
    sssss.urls.map((url) =>
      fetch(`${url}/permits/${chainId}/${registry}/${identityIdHex}`, {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
        },
        body: JSON.stringify({
          permitter,
          recipient,
          duration: permitTtl,
          authorization,
          context,
        }),
        signal: abort.signal,
      }),
    ),
  );
  clearTimeout(fetchTimeout);
  let successCount = 0;
  const errorBodies = [];
  for (let i = 0; i < permitResults.length; i++) {
    const result = permitResults[i];
    const ssss = sssss.urls[i];
    if (result.status === 'rejected') {
      console.warn(`SSSS ${ssss} could not be reached: ${result.reason}`);
    } else if (result.value.status === 201) {
      successCount += 1;
    } else if (result.value.status === 202) {
      console.warn(`SSSS ${ssss} did not optimistcally issue permit`);
    } else if (!result.value.ok && result.value.status !== 401 && result.value.status !== 403) {
      errorBodies.push((async () => [ssss, await result.value.text()] as [string, string])());
    }
  }
  const errors = await Promise.allSettled(errorBodies);
  for (const error of errors) {
    if (error.status === 'rejected') continue;
    const [ssss, msg] = error.value;
    console.warn(`SSSS ${ssss} returned an error: ${msg}`);
  }
  const quorum = sssss?.quorum ?? Math.ceil(sssss.urls.length / 2);
  if (successCount < quorum) throw new ApiError(403, 'quorum not reached');
}

async function acquireIdentitySapphire(
  gasKey: Hash,
  requesterService: string,
  opts: types.AcquireIdentityParams,
): Promise<void> {
  const {
    network: { chainId, rpcUrl },
    identity: { registry, id: identityIdHex },
    permitter,
    permitTtl,
    recipient,
    context,
    authorization,
    duration,
  } = opts;
  const requester = recipient ?? allocateAccount(requesterService).address;
  let publicClient = getPublicClient(chainId, rpcUrl);
  const identityId = hexToBigInt(identityIdHex);

  if (duration) {
    // If the permit key only needs to last a little while and it's already current, reuse it.
    const now = Math.ceil(Date.now() / 1000);
    const { expiry } = await publicClient.readContract({
      address: registry,
      abi: IdentityRegistryAbi,
      functionName: 'readPermit',
      args: [requester, identityId],
    });
    if (now + duration < expiry) return;
  }

  let gasWallet = getWalletClient(gasKey, chainId, rpcUrl);

  let permitterAddress: Address;
  if (permitter) {
    permitterAddress = permitter;
  } else {
    permitterAddress = await publicClient.readContract({
      address: registry,
      abi: IdentityRegistryAbi,
      functionName: 'getPermitter',
      args: [identityId],
    });
  }
  const permitDuration = permitTtl ? BigInt(permitTtl) : 60n * 60n;
  const hash = await gasWallet.writeContract({
    address: permitterAddress,
    abi: PermitterAbi,
    functionName: 'acquireIdentity',
    args: [identityId, requester, permitDuration, context ?? '0x', authorization ?? '0x'],
  });

  let retriesRemaining = 3;
  while (true) {
    try {
      const { status } = await publicClient.waitForTransactionReceipt({ hash, confirmations: 2 });
      if (status !== 'success') throw new Error(`failed to acquire identity in ${hash}`);
      return;
    } catch (e: any) {
      if (e instanceof BlockNotFoundError || e instanceof TransactionNotFoundError) {
        if (retriesRemaining === 0) throw e;
        retriesRemaining--;
        await new Promise((resolve) => setTimeout(resolve, 2_000));
        continue;
      }
      throw e;
    }
  }
}
