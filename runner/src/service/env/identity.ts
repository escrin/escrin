import { Address, Hash, hexToBigInt } from 'viem';

import {
  IIdentityRegistry as IdentityRegistryAbi,
  IPermitter as PermitterAbi,
} from '@escrin/evm/abi';

import { allocateAccount } from './account.js';
import { getPublicClient, getWalletClient } from './chains.js';

import * as types from './types.js';

export async function handleAcquireIdentity(
  gasKey: Hash,
  requesterService: string,
  opts: types.AcquireIdentityParams,
): Promise<void> {
  const {
    network: { chainId, rpcUrl },
    identity: { registry, id: identityIdHex },
    permit: { lifetime, requiredDuration, permitter, authz } = {},
  } = opts;
  const requester = allocateAccount(requesterService);
  let publicClient = getPublicClient(chainId, rpcUrl);
  const identityId = hexToBigInt(identityIdHex);

  if (requiredDuration) {
    // If the permit key only needs to last a little while and it's already current, reuse it.
    const now = Math.ceil(Date.now() / 1000);
    const { expiry } = await publicClient.readContract({
      address: registry,
      abi: IdentityRegistryAbi,
      functionName: 'readPermit',
      args: [requester.address, identityId],
    });
    if (now + requiredDuration < Number(expiry)) return;
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
  const permitDuration = lifetime ? BigInt(lifetime) : 60n * 60n;
  const context = '0x'; // TODO: include context
  const hash = await gasWallet.writeContract({
    address: permitterAddress,
    abi: PermitterAbi,
    functionName: 'acquireIdentity',
    args: [identityId, requester.address, permitDuration, context, authz ?? '0x'],
  });
  const { status } = await publicClient.waitForTransactionReceipt({ hash });
  if (status !== 'success') throw new Error(`failed to acquire identity in ${hash}`);
}
