import { PrivateKeyAccount, generatePrivateKey, privateKeyToAccount } from 'viem/accounts';

const accounts = new Map<string, PrivateKeyAccount>();

export function allocateAccount(serviceName: string): PrivateKeyAccount {
  let account = accounts.get(serviceName);
  if (account) return account;
  account = privateKeyToAccount(generatePrivateKey());
  accounts.set(serviceName, account);
  return account;
}
