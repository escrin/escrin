import { wrapPublicClient } from '@oasisprotocol/sapphire-paratime/compat/viem';
import type { SetRequired } from 'type-fest';
import { Hex, PrivateKeyAccount } from 'viem';

import { OmniKeyStore as OmniKeyStoreAbi } from '@escrin/evm/abi';

import * as ssss from '../../ssss/index.js';

import { allocateAccount, allocateAccountKey } from './account.js';
import { getPublicClient, getWalletClient } from './chains.js';
import * as types from './types.js';

export async function handleGetKey(
  requesterService: string,
  params: types.GetKeyRequest['params'],
): Promise<types.GetKeyRequest['response']> {
  if (params.keyId === 'omni') {
    if (!params.ssss && !isSapphire(params.network)) {
      throw new Error('no secret storage backend configured');
    }
    const requesterAccount = allocateAccount(requesterService);

    if (isSapphire(params.network)) {
      return { key: await getSapphireOmniKey(params, requesterAccount) };
    }

    return {
      key: await getSsssOmniKey(
        params as SetRequired<types.EvmKeyStoreParams, 'ssss'>,
        requesterAccount,
      ),
    };
  }

  if (params.keyId === 'ephemeral-account') {
    return { key: allocateAccountKey(requesterService) };
  }

  const _exhaustiveCheck: never = params;
  return _exhaustiveCheck;
}

async function getSsssOmniKey(
  params: SetRequired<types.EvmKeyStoreParams, 'ssss'>,
  requesterAccount: PrivateKeyAccount,
): Promise<Hex> {
  const { network, identity, ssss: ssssParams } = params;
  const publicClient = getPublicClient(network.chainId, network.rpcUrl);
  const currentVersion = await ssss.getSecretVersion(
    publicClient,
    ssssParams.hub,
    identity.id,
    'omni',
  );
  if (currentVersion > 0) return ssss.getSecret('omni', currentVersion, params, requesterAccount);
  const walletClient = getWalletClient(requesterAccount, network.chainId, network.rpcUrl);
  return ssss.dealNewSecret('omni', params, publicClient, walletClient);
}

async function getSapphireOmniKey(
  params: types.EvmKeyStoreParams,
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
