import {
  getDefaultCipher,
  wrapPublicClient,
  wrapWalletClient,
} from '@oasisprotocol/sapphire-paratime/compat/viem';
import {
  Address,
  Chain,
  PrivateKeyAccount,
  createPublicClient,
  createWalletClient,
  http,
  toBytes,
  toHex,
} from 'viem';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';

import { OmniKeyStore as OmniKeyStoreAbi, IPermitter as IPermitterAbi } from '@escrin/evm/abi';

export type GetKeyOpts = {
  identity: bigint;
  keyStore: {
    chain: Chain;
    address: Address;
  };
  authz: Uint8Array;
  gasKey: `0x${string}`;
  isSapphire: boolean;
  overrides?: Partial<{
    permitter: Address;
  }>;
};

const appAccountCache = new Map<string, PrivateKeyAccount>();

export async function getOmniKey({
  identity,
  keyStore: { chain, address: keyStoreAddr },
  authz,
  gasKey,
  isSapphire,
  overrides,
}: GetKeyOpts): Promise<Uint8Array> {
  const clientConfig = { chain, transport: http() };
  let publicClient = createPublicClient(clientConfig);

  const gasAccount = privateKeyToAccount(gasKey);
  let gasWallet = createWalletClient({ ...clientConfig, account: gasAccount }); // TODO: round-robin scheduling

  const contextKey = JSON.stringify([chain.id, keyStoreAddr, isSapphire]);
  let appAccount = appAccountCache.get(contextKey)!;
  if (appAccount === undefined) {
    const appKey = generatePrivateKey();
    appAccount = privateKeyToAccount(appKey);
    appAccountCache.set(contextKey, appAccount);
  }

  if (isSapphire) {
    const cipher = await getDefaultCipher(publicClient);
    publicClient = wrapPublicClient(publicClient, { cipher });
    gasWallet = wrapWalletClient(gasWallet, { cipher });
  }

  const currentPermit = await publicClient.readContract({
    address: keyStoreAddr,
    abi: OmniKeyStoreAbi,
    functionName: 'readPermit',
    args: [appAccount.address, identity],
  });
  const bufferTime = 2 * 60; // two minutes
  if (Number(currentPermit.expiry) - bufferTime <= Date.now() / 1000) {
    let permitterAddr = overrides?.permitter;
    if (!permitterAddr) {
      permitterAddr = await publicClient.readContract({
        address: keyStoreAddr,
        abi: OmniKeyStoreAbi,
        functionName: 'getPermitter',
        args: [identity],
      });
    }
    const permitDuration = 60n * 60n; // one hour
    const context = '0x'; // TODO: include context
    const hash = await gasWallet.writeContract({
      address: permitterAddr,
      abi: IPermitterAbi,
      functionName: 'acquireIdentity',
      args: [identity, appAccount.address, permitDuration, context, toHex(authz)],
    });
    const { status } = await publicClient.waitForTransactionReceipt({ hash });
    if (status !== 'success') throw new Error(`failed to acquire identity in ${hash}`);
  }

  const keyRequest = {
    identity,
    requester: appAccount.address,
    expiry: 5n, // five blocks
  };
  const keyRequestSig = await appAccount.signTypedData({
    domain: {
      name: 'OmniKeyStore',
      version: '1',
      chainId: chain.id,
      verifyingContract: keyStoreAddr,
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
  const keyHex = await publicClient.readContract({
    address: keyStoreAddr,
    abi: OmniKeyStoreAbi,
    functionName: 'getKey',
    args: [
      {
        req: keyRequest,
        sig: keyRequestSig,
      },
    ],
  });
  return toBytes(keyHex);
}
