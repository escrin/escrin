import {
  PublicClient,
  Hash,
  Transport,
  WalletClient,
  createPublicClient,
  createWalletClient,
  http,
} from 'viem';
import { Account, privateKeyToAccount } from 'viem/accounts';
import { Chain, foundry, localhost } from 'viem/chains';

export const sapphire = {
  id: 0x5afe,
  name: 'Sapphire',
  network: 'sapphire',
  nativeCurrency: {
    decimals: 18,
    name: 'ROSE',
    symbol: 'ROSE',
  },
  rpcUrls: {
    default: { http: ['https://sapphire.oasis.io'] },
    public: { http: ['https://sapphire.oasis.io'] },
  },
  blockExplorers: {
    default: { name: 'Oasis Explorer', url: 'https://explorer.oasis.io/mainnet/sapphire' },
  },
  contracts: {
    multicall3: {
      address: '0xcA11bde05977b3631167028862bE2a173976CA11',
      blockCreated: 734531,
    },
  },
} as Chain;

export const sapphireTestnet = {
  id: 0x5aff,
  name: 'Sapphire Testnet',
  network: 'sapphire-testnet',
  nativeCurrency: {
    decimals: 18,
    name: 'TEST',
    symbol: 'TEST',
  },
  rpcUrls: {
    default: { http: ['https://testnet.sapphire.oasis.dev'] },
    public: { http: ['https://testnet.sapphire.oasis.dev'] },
  },
  blockExplorers: {
    default: { name: 'Oasis Explorer', url: 'https://explorer.oasis.io/testnet/sapphire' },
  },
  testnet: true,
} as Chain;

export function getPublicClient(chainId: number, rpcUrl?: string): PublicClient<Transport, Chain> {
  const chain = getChain(chainId, rpcUrl);
  return createPublicClient({
    chain,
    transport: http(),
  });
}

export function getWalletClient(
  privateKey: Hash,
  chainId: number,
  rpcUrl?: string,
): WalletClient<Transport, Chain, Account> {
  const chain = getChain(chainId, rpcUrl);
  const gasAccount = privateKeyToAccount(privateKey);
  // TODO: round-robin scheduling
  return createWalletClient({ chain, transport: http(), account: gasAccount });
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
