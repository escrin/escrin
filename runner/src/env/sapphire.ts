import { CryptoKey } from '@cloudflare/workers-types/experimental';

import * as sapphire from '@oasisprotocol/sapphire-paratime';
import { ethers } from 'ethers';
import createKeccakHash from 'keccak';

import { AttestationToken, AttestationTokenFactory, Lockbox, LockboxFactory } from '@escrin/evm';

import { Cacheable, Module, RpcError } from './index.js';

type Registration = AttestationToken.RegistrationStruct;

export type InitOpts = {
  web3GatewayUrl: string;
  attokAddr: string;
  lockboxAddr: string;
  debug?: Partial<{
    nowrap: boolean;
  }>;
};

export const INIT_SAPPHIRE: InitOpts = {
  web3GatewayUrl: 'https://sapphire.oasis.io',
  attokAddr: '0x96c1D1913310ACD921Fc4baE081CcDdD42374C36',
  lockboxAddr: '0x53FE9042cbB6B9773c01F678F7c0439B09EdCeB3',
};

export const INIT_SAPPHIRE_TESTNET: InitOpts = {
  web3GatewayUrl: 'https://testnet.sapphire.oasis.dev',
  attokAddr: '0x960bEAcD9eFfE69e692f727F52Da7DF3601dc80f',
  lockboxAddr: '0x68D4f98E5cd2D8d2C6f03c095761663Bf1aA8442',
};

const lazy = <T extends object>(initializer: () => T): T => {
  let value: T | undefined;
  let initialized = false;

  return new Proxy(
    {},
    {
      get: (_, prop) => {
        if (!initialized) {
          value = initializer();
          initialized = true;
        }
        return Reflect.get(value!, prop);
      },
    },
  ) as T;
};

export default function make(optsOrNet: InitOpts | 'mainnet' | 'testnet', gasKey: string): Module {
  const opts =
    optsOrNet === 'mainnet'
      ? INIT_SAPPHIRE
      : optsOrNet === 'testnet'
      ? INIT_SAPPHIRE_TESTNET
      : optsOrNet;

  const provider = lazy(() => new ethers.JsonRpcProvider(opts.web3GatewayUrl));
  const gasWallet = lazy(() => new ethers.Wallet(gasKey).connect(provider));
  const localWallet = lazy(() => {
    const localWallet = ethers.Wallet.createRandom().connect(provider);
    return opts.debug?.nowrap ? localWallet : sapphire.wrap(localWallet);
  }) as any as ethers.BaseWallet;
  const attok = lazy(() => {
    return AttestationTokenFactory.connect(opts.attokAddr, gasWallet);
  });
  const lockbox = lazy(() => LockboxFactory.connect(opts.lockboxAddr, localWallet));

  return {
    async getKey(id: string): Promise<Cacheable<CryptoKey>> {
      if (id !== 'omni') throw new RpcError(404, `unknown key \`${id}\``);

      const oneHourFromNow = Math.floor(Date.now() / 1000) + 60 * 60;
      let currentBlock = await provider.getBlock('latest');
      if (currentBlock === null) throw new RpcError(500, 'unable to get current block');
      const prevBlock = await provider.getBlock(currentBlock.number - 1);
      if (prevBlock === null) throw new RpcError(500, 'unable to get previous block');
      const registration: Registration = {
        baseBlockHash: prevBlock.hash!,
        baseBlockNumber: prevBlock.number,
        expiry: oneHourFromNow,
        registrant: localWallet.getAddress(),
        tokenExpiry: oneHourFromNow,
      };
      const quote = await mockQuote(registration);
      const tcbId = await sendAttestation(attok.connect(localWallet), quote, registration);

      const key = await getOrCreateKey(lockbox, gasWallet, tcbId);

      return new Cacheable(key, new Date(oneHourFromNow));
    },
  };
}

async function mockQuote(registration: Registration): Promise<Uint8Array> {
  const coder = ethers.AbiCoder.defaultAbiCoder();
  const measurementHash = '0xc275e487107af5257147ce76e1515788118429e0caa17c04d508038da59d5154'; // static random bytes. this is just a key in a key-value store.
  const regTypeDef =
    'tuple(uint256 baseBlockNumber, bytes32 baseBlockHash, uint256 expiry, uint256 registrant, uint256 tokenExpiry)'; // TODO: keep this in sync with the actual typedef
  const regBytesHex = coder.encode([regTypeDef], [registration]);
  const regBytes = Buffer.from(ethers.getBytes(regBytesHex));
  return ethers.getBytes(
    coder.encode(
      ['bytes32', 'bytes32'],
      [measurementHash, createKeccakHash('keccak256').update(regBytes).digest()],
    ),
  );
}

async function sendAttestation(
  attok: AttestationToken,
  quote: Uint8Array,
  reg: Registration,
): Promise<string> {
  const expectedTcbId = await attok.getTcbId(quote);
  if (await attok.isAttested(reg.registrant, expectedTcbId)) return expectedTcbId;
  const tx = await attok.attest(quote, reg, { gasLimit: 10_000_000 });
  const receipt = await tx.wait();
  if (receipt?.status !== 1) throw new Error('attestation tx failed');
  let tcbId = '';
  const attestedTopic = attok.getEvent('Attested').fragment.topicHash;
  for (const log of receipt ?? []) {
    if (log.topics[0] !== attestedTopic) continue;
    tcbId = log.topics[1];
  }
  if (!tcbId) throw new Error('could not retrieve attestation id');
  await waitForConfirmation(attok.runner!.provider!, receipt);
  return tcbId;
}

async function waitForConfirmation(
  provider: ethers.Provider,
  receipt: ethers.ContractTransactionReceipt | null,
): Promise<void> {
  const { chainId } = await provider.getNetwork();
  if (chainId !== 0x5afen && chainId !== 0x5affn) return;
  const getCurrentBlock = () => provider.getBlock('latest');
  let currentBlock = await getCurrentBlock();
  while (currentBlock && receipt && currentBlock.number <= receipt.blockNumber + 1) {
    await new Promise((resolve) => setTimeout(resolve, 3_000));
    currentBlock = await getCurrentBlock();
  }
}

async function getOrCreateKey(
  lockbox: Lockbox,
  gasWallet: ethers.ContractRunner,
  tcbId: string,
): Promise<CryptoKey> {
  let keyHex = await lockbox.getKey(tcbId);
  if (!/^(0x)?0+$/.test(keyHex)) return importKey(keyHex);
  const tx = await lockbox
    .connect(gasWallet)
    .createKey(tcbId, crypto.getRandomValues(new Uint8Array(32)), { gasLimit: 10_000_000 });
  const receipt = await tx.wait();
  await waitForConfirmation(lockbox.runner!.provider!, receipt);
  keyHex = await lockbox.getKey(tcbId);
  return importKey(keyHex);
}

async function importKey(keyHex: string): Promise<CryptoKey> {
  const key = ethers.getBytes(keyHex);
  return crypto.subtle.importKey('raw', key, 'HKDF', false, ['deriveKey', 'deriveBits']);
}
