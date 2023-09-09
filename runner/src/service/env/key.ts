import { wrapPublicClient } from '@oasisprotocol/sapphire-paratime/compat/viem';
import { PrivateKeyAccount, hexToBigInt, toBytes } from 'viem';

import { OmniKeyStore as OmniKeyStoreAbi } from '@escrin/evm/abi';

import { encodeBase64Bytes } from '../../rpc.js';
import { allocateAccount } from './account.js';
import { getPublicClient } from './chains.js';

import * as types from './types.js';

export async function handleGetKey(
  requester: string,
  params: types.GetKeyParams,
): Promise<{ key: string }> {
  const requesterAccount = allocateAccount(requester);
  const key = await getSapphireOmniKey(params, requesterAccount);

  return { key: encodeBase64Bytes(key) };
}

async function getSapphireOmniKey(
  params: types.EvmKeyStoreParams,
  requesterAccount: PrivateKeyAccount,
): Promise<Uint8Array> {
  const { network, identity } = params;

  let publicClient = getPublicClient(network.chainId, network.rpcUrl);

  if (Math.abs(network.chainId - 0x5afe) <= 1) {
    publicClient = wrapPublicClient(publicClient);
  }

  const keyRequest = {
    identity: hexToBigInt(identity.id),
    requester: requesterAccount.address,
    expiry: 5n, // five blocks
  };
  const keyRequestSig = await requesterAccount.signTypedData({
    domain: {
      name: 'OmniKeyStore',
      version: '1',
      chainId: network.chainId,
      verifyingContract: identity.registry,
    },
    types: {
      KeyRequest: [
        { name: 'identity', type: 'uint256' },
        { name: 'requester', type: 'address' },
        { name: 'expiry', type: 'uint256' },
      ],
    },
    primaryType: 'KeyRequest',
    message: keyRequest,
  });
  return toBytes(
    await publicClient.readContract({
      address: identity.registry,
      abi: OmniKeyStoreAbi,
      functionName: 'getKey',
      args: [
        {
          req: keyRequest,
          sig: keyRequestSig,
        },
      ],
    }),
  );
}
