import { keccak_256 } from '@noble/hashes/sha3';
import * as sapphire from '@oasisprotocol/sapphire-paratime';
import { ethers } from 'ethers';

import { AttestationToken, AttestationTokenFactory, Lockbox, LockboxFactory } from '@escrin/evm';

import { Cacheable, cacheability } from '../index.js';

type Registration = AttestationToken.RegistrationStruct;

export type InitOpts = {
  web3GatewayUrl: string;
  attokAddr: string;
  lockboxAddr: string;
  network?: ethers.Network;
  isSapphire: boolean;
};

export const INIT_MAINNET: InitOpts = {
  web3GatewayUrl: 'https://sapphire.oasis.io',
  attokAddr: '0x96c1D1913310ACD921Fc4baE081CcDdD42374C36',
  lockboxAddr: '0x53FE9042cbB6B9773c01F678F7c0439B09EdCeB3',
  network: new ethers.Network('sapphire-mainnet', 0x5afe),
  isSapphire: true,
};

export const INIT_TESTNET: InitOpts = {
  web3GatewayUrl: 'https://testnet.sapphire.oasis.dev',
  attokAddr: '0x960bEAcD9eFfE69e692f727F52Da7DF3601dc80f',
  lockboxAddr: '0x68D4f98E5cd2D8d2C6f03c095761663Bf1aA8442',
  network: new ethers.Network('sapphire-testnet', 0x5aff),
  isSapphire: true,
};

export const INIT_LOCAL: InitOpts = {
  web3GatewayUrl: 'http://127.0.0.1:8545',
  attokAddr: '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512',
  lockboxAddr: '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0',
  network: new ethers.Network('sapphire-local', 31337),
  isSapphire: false,
};

export async function getKey(
  _keyId: 'omni',
  _proof: string,
  gasKey: string,
  opts: InitOpts,
): Promise<Cacheable<Uint8Array>> {
  const provider = new ethers.JsonRpcProvider(opts.web3GatewayUrl, undefined, {
    staticNetwork: opts.network,
  });
  const gasWallet = new ethers.Wallet(gasKey, provider);
  let localWallet = new ethers.Wallet(gasKey, provider);
  // const localWallet = ethers.Wallet.createRandom().connect(provider);
  if (opts.isSapphire) {
    localWallet = sapphire.wrap(localWallet);
  }
  const attok = AttestationTokenFactory.connect(opts.attokAddr, gasWallet);
  const lockbox = LockboxFactory.connect(opts.lockboxAddr, localWallet);

  const oneHourFromNow = Math.floor(Date.now() / 1000) + 60 * 60;

  let currentBlock = await provider.getBlock('latest');
  if (currentBlock === null) throw new Error('unable to get current block');

  const prevBlock = await provider.getBlock(currentBlock.number - 1);
  if (prevBlock === null) throw new Error('unable to get previous block');

  const registration: Registration = {
    baseBlockHash: prevBlock.hash!,
    baseBlockNumber: prevBlock.number,
    expiry: oneHourFromNow,
    registrant: await localWallet.getAddress(),
    tokenExpiry: oneHourFromNow,
  };
  const quote = await mockQuote(registration);
  const tcbId = await sendAttestation(
    attok.connect(localWallet),
    quote,
    registration,
    opts.isSapphire,
  );
  const key = await getOrCreateKey(lockbox, gasWallet, tcbId, opts.isSapphire);

  Object.defineProperty(key, cacheability, {
    value: new Date(oneHourFromNow),
    enumerable: false,
    writable: false,
    configurable: true,
  });

  return key as Cacheable<Uint8Array>;
}

async function mockQuote(registration: Registration): Promise<Uint8Array> {
  const coder = ethers.AbiCoder.defaultAbiCoder();
  const measurementHash = '0xc275e487107af5257147ce76e1515788118429e0caa17c04d508038da59d5154'; // static random bytes. this is just a key in a key-value store.
  const regTypeDef =
    'tuple(uint256 baseBlockNumber, bytes32 baseBlockHash, uint256 expiry, uint256 registrant, uint256 tokenExpiry)'; // TODO: keep this in sync with the actual typedef
  const regBytesHex = coder.encode([regTypeDef], [registration]);
  const regBytes = ethers.getBytes(regBytesHex);
  return ethers.getBytes(
    coder.encode(
      ['bytes32', 'bytes32'],
      [measurementHash, keccak_256.create().update(regBytes).digest()],
    ),
  );
}

async function sendAttestation(
  attok: AttestationToken,
  quote: Uint8Array,
  reg: Registration,
  isSapphire: boolean,
): Promise<string> {
  const expectedTcbId = await attok.getTcbId(quote, {
    from: reg.registrant,
  });
  if (await attok.isAttested(reg.registrant, expectedTcbId)) return expectedTcbId;
  const tx = await attok.attest(quote, reg, { gasLimit: 10_000_000 });
  const receipt = await tx.wait();
  if (receipt?.status !== 1) throw new Error('attestation tx failed');
  let tcbId = '';
  const attestedTopic = attok.getEvent('Attested').fragment.topicHash;
  for (const log of receipt ?? []) {
    if (log.topics[0] !== attestedTopic) continue;
    tcbId = log.topics[2];
    if (tcbId !== expectedTcbId) throw new Error('received unexpected TCB ID');
  }
  if (!tcbId) throw new Error('could not retrieve attestation id');
  await waitForConfirmation(attok.runner!.provider!, receipt, isSapphire);
  return tcbId;
}

async function waitForConfirmation(
  provider: ethers.Provider,
  receipt: ethers.ContractTransactionReceipt | null,
  isSapphire: boolean,
): Promise<void> {
  if (!isSapphire) return;
  let currentBlock = await provider.getBlock('latest');
  while (currentBlock && receipt && currentBlock.number <= receipt.blockNumber + 1) {
    await (globalThis as any).scheduler.wait(6_000);
    currentBlock = await provider.getBlock('latest');
  }
}

async function getOrCreateKey(
  lockbox: Lockbox,
  gasWallet: ethers.Wallet,
  tcbId: string,
  isSapphire: boolean,
): Promise<Uint8Array> {
  let keyHex = await lockbox.getKey(tcbId, { from: gasWallet.getAddress() });
  if (!/^(0x)?0+$/.test(keyHex)) return ethers.getBytes(keyHex);
  const tx = await lockbox
    .connect(gasWallet)
    .createKey(tcbId, crypto.getRandomValues(new Uint8Array(32)), { gasLimit: 10_000_000 });
  const receipt = await tx.wait();
  await waitForConfirmation(lockbox.runner!.provider!, receipt, isSapphire);
  keyHex = await lockbox.getKey(tcbId);
  return ethers.getBytes(keyHex);
}
