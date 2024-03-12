import { wrapPublicClient } from '@oasisprotocol/sapphire-paratime/compat/viem';
import { Hex, PrivateKeyAccount, hexToBigInt } from 'viem';

import { OmniKeyStore as OmniKeyStoreAbi } from '@escrin/evm/abi';

import { allocateAccount, allocateAccountKey } from './account.js';
import { getPublicClient } from './chains.js';

import * as types from './types.js';

export async function handleGetKey(
  requester: string,
  params: types.GetKeyRequest['params'],
): Promise<types.GetKeyRequest['response']> {
  if (params.keyId === 'omni') {
    const requesterAccount = allocateAccount(requester);
    return { key: await getSapphireOmniKey(params, requesterAccount) };
  }

  if (params.keyId === 'ephemeral-account') {
    return { key: allocateAccountKey(requester) };
  }

  const _exhaustiveCheck: never = params;
  return _exhaustiveCheck;
}

async function getSapphireOmniKey(
  params: types.EvmKeyStoreParams,
  requesterAccount: PrivateKeyAccount,
): Promise<Hex> {
  const { network, identity } = params;

  let publicClient = getPublicClient(network.chainId, network.rpcUrl);

  if (Math.abs(network.chainId - 0x5afe) <= 1) {
    publicClient = wrapPublicClient(publicClient);
  }

  const keyRequest = {
    identity: identity.id,
    requester: requesterAccount.address,
    expiry: BigInt(Math.floor(Date.now() / 1000) + 2 * 60), // two minutes
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
        { name: 'identity', type: 'bytes32' },
        { name: 'requester', type: 'address' },
        { name: 'expiry', type: 'uint256' },
      ],
    },
    primaryType: 'KeyRequest',
    message: keyRequest,
  });
  return publicClient.readContract({
    address: identity.registry,
    abi: OmniKeyStoreAbi,
    functionName: 'getKey',
    args: [
      {
        req: keyRequest,
        sig: keyRequestSig,
      },
    ],
  });
}
