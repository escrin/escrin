import { CryptoKey } from '@cloudflare/workers-types/experimental';

export interface EscrinWorker {
  tasks(): Promise<void>;
}

export interface EscrinRunner {
  getKey(store: 'sapphire-mainnet' | 'sapphire-testnet', ident: string): Promise<CryptoKey>;
}
