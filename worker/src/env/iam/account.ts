import { Hash } from 'viem';
import { PrivateKeyAccount, generatePrivateKey, privateKeyToAccount } from 'viem/accounts';

const accountKeys = new Map<string, Hash>();

export function allocateAccount(serviceName: string): PrivateKeyAccount {
  return privateKeyToAccount(allocateAccountKey(serviceName));
}

export function allocateAccountKey(serviceName: string): Hash {
  let accountKey = accountKeys.get(serviceName);
  if (!accountKey) {
    const key = generatePrivateKey();
    accountKeys.set(serviceName, key);
    accountKey = key;
  }
  return accountKey;
}
