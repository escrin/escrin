export interface EscrinWorker {
  tasks(): Promise<void>;
}

export type KmNetwork = 'sapphire-mainnet' | 'sapphire-testnet';
export type StateNetwork = 'sapphire-mainnet' | 'sapphire-testnet';

export interface EscrinRunner {
  getConfig(): Promise<Record<string, unknown>>;
  getKey(store: KmNetwork, ident: string): Promise<CryptoKey>;
  // getEthProvider(network: StateNetwork): Promise<EIP1193Provider>; // TODO: this does not need to be async here
}

export interface EIP1193Provider {
  request: (request: EIP1193Request) => Promise<EIP1193Response>;
}

export interface EIP1193Request {
  method: string;
  params?: any[];
}

export interface EIP1193Response {
  result?: any;
  error?: string;
}
