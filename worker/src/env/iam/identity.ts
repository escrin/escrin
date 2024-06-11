import { Address, BlockNotFoundError, Hash, TransactionNotFoundError } from 'viem';

import {
  IIdentityRegistry as IdentityRegistryAbi,
  IPermitter as PermitterAbi,
} from '@escrin/evm/abi';

import * as ssss from '../../ssss/index.js';
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
  params: types.AcquireIdentityParams,
): Promise<void> {
  if ('ssss' in params) return acquireIdentitySsss(requesterService, params);
  return acquireIdentitySapphire(gasKey, requesterService, params);
}

async function acquireIdentitySsss(
  requesterService: string,
  params: types.SsssAcquireIdentityParams,
): Promise<void> {
  const {
    network: { chainId, rpcUrl },
    identity,
  } = params;
  const publicClient = getPublicClient(chainId, rpcUrl);
  const requesterAccount = allocateAccount(requesterService);
  const walletClient = getWalletClient(requesterAccount, chainId, rpcUrl);
  const permitter =
    params.permitter ??
    (await publicClient.readContract({
      address: identity.registry,
      abi: IdentityRegistryAbi,
      functionName: 'getPermitter',
      args: [identity.id],
    }));
  await ssss.acquireIdentity(
    {
      ...params,
      permitter,
      recipient: params.recipient ?? allocateAccount(requesterService).address,
      permitTtl: params.permitTtl ?? 24 * 60 * 60,
      ssss: params.ssss,
    },
    publicClient,
    walletClient,
  );
  // TODO: consider rethrowing as 403
}

async function acquireIdentitySapphire(
  gasKey: Hash,
  requesterService: string,
  opts: types.AcquireIdentityParams,
): Promise<void> {
  const {
    network: { chainId, rpcUrl },
    identity,
    permitter,
    permitTtl,
    recipient,
    context,
    authorization,
    duration,
  } = opts;
  const requester = recipient ?? allocateAccount(requesterService).address;
  const publicClient = getPublicClient(chainId, rpcUrl);

  if (duration) {
    // If the permit key only needs to last a little while and it's already current, reuse it.
    const now = Math.ceil(Date.now() / 1000);
    const { expiry } = await publicClient.readContract({
      address: identity.registry,
      abi: IdentityRegistryAbi,
      functionName: 'readPermit',
      args: [requester, identity.id],
    });
    if (now + duration < expiry) return;
  }

  const gasWallet = getWalletClient(gasKey, chainId, rpcUrl);

  let permitterAddress: Address;
  if (permitter) {
    permitterAddress = permitter;
  } else {
    permitterAddress = await publicClient.readContract({
      address: identity.registry,
      abi: IdentityRegistryAbi,
      functionName: 'getPermitter',
      args: [identity.id],
    });
  }
  const permitDuration = permitTtl ? BigInt(permitTtl) : 60n * 60n;
  const hash = await gasWallet.writeContract({
    address: permitterAddress,
    abi: PermitterAbi,
    functionName: 'acquireIdentity',
    args: [identity.id, requester, permitDuration, context ?? '0x', authorization ?? '0x'],
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
