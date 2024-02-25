import { Address, BlockNotFoundError, Hash, TransactionNotFoundError, hexToBigInt } from 'viem';

import {
  IIdentityRegistry as IdentityRegistryAbi,
  IPermitter as PermitterAbi,
} from '@escrin/evm/abi';

import { allocateAccount } from './account.js';
import { getPublicClient, getWalletClient } from './chains.js';

import * as types from './types.js';

export async function handleGetAccount(
  requesterService: string,
  { id }: types.GetAccountParams,
): Promise<types.GetAccountRequest['response']> {
  if (id !== 'worker') throw new Error(`unknown account: ${id}`);
  const { address } = allocateAccount(requesterService);
  return { address };
}

export async function handleAcquireIdentity(
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
