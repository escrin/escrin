import { wrapPublicClient } from '@oasisprotocol/sapphire-paratime/compat/viem';
import { Hex, PrivateKeyAccount } from 'viem';

import { OmniKeyStore as OmniKeyStoreAbi } from '@escrin/evm/abi';

import * as ssss from '../../ssss/index.js';

import { allocateAccount, allocateAccountKey } from './account.js';
import { getPublicClient } from './chains.js';
import * as types from './types.js';

export async function handleGetSecret(
  requesterService: string,
  params: types.GetSecretRequest['params'],
): Promise<types.GetSecretRequest['response']> {
  if (params.secretName === 'omni') {
    if (!('ssss' in params) && !isSapphire(params.network)) {
      throw new Error('no secret storage backend configured');
    }
    const requesterAccount = allocateAccount(requesterService);

    if (isSapphire(params.network)) {
      return { secret: await getSapphireOmniSecret(params, requesterAccount) };
    }

    return {
      secret: await getSsssOmniSecret(params as types.SsssSecretStoreParams, requesterAccount),
    };
  }

  if (params.secretName === 'ephemeral-account') {
    return { secret: allocateAccountKey(requesterService) };
  }

  const _exhaustiveCheck: never = params;
  return _exhaustiveCheck;
}

async function getSsssOmniSecret(
  params: types.SsssSecretStoreParams,
  requesterAccount: PrivateKeyAccount,
): Promise<Hex> {
  return ssss.dealNewSecret(params, requesterAccount);
}

async function getSapphireOmniSecret(
  params: types.EvmSecretStoreParams,
  requesterAccount: PrivateKeyAccount,
): Promise<Hex> {
  const { network, identity } = params;

  let publicClient = getPublicClient(network.chainId, network.rpcUrl);

  if (isSapphire(network)) {
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

const isSapphire = (network: types.Network) => Math.abs(network.chainId - 0x5afe) <= 1;
