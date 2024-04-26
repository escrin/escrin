import {
  Hash,
  PrivateKeyAccount,
  PublicClient,
  Transport,
  WalletClient,
  createPublicClient,
  createWalletClient,
  http,
} from 'viem';
import { Account, privateKeyToAccount } from 'viem/accounts';
import { Chain, foundry, localhost, sapphire, sapphireTestnet } from 'viem/chains';

export function getPublicClient(chainId: number, rpcUrl?: string): PublicClient<Transport, Chain> {
  const chain = getChain(chainId, rpcUrl);
  return createPublicClient({
    chain,
    transport: http(),
  });
}

export function getWalletClient(
  privateKeyOrAccount: Hash | PrivateKeyAccount,
  chainId: number,
  rpcUrl?: string,
): WalletClient<Transport, Chain, Account> {
  const chain = getChain(chainId, rpcUrl);
  const account =
    typeof privateKeyOrAccount === 'string'
      ? privateKeyToAccount(privateKeyOrAccount)
      : privateKeyOrAccount;
  return createWalletClient({ chain, transport: http(), account });
}

export function getChain(chainId: number, rpcUrl?: string): Chain {
  if (rpcUrl) {
    return {
      id: chainId,
      name: 'Custom Network',
      nativeCurrency: { decimals: 18, name: '', symbol: '' },
      rpcUrls: {
        default: { http: [rpcUrl] },
        public: { http: [rpcUrl] },
      },
    };
  }
  if (chainId === 0x5afe) return sapphire;
  if (chainId === 0x5aff) return sapphireTestnet;
  if (chainId === 31337) return foundry;
  if (chainId === 1337) return localhost;
  throw new Error(`the chain with id ${chainId} is unrecognized, so \`rpcUrl\` is required`);
}
