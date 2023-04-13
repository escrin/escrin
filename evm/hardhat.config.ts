import { HardhatUserConfig, task } from 'hardhat/config';
import '@oasisprotocol/sapphire-hardhat';
import '@nomicfoundation/hardhat-toolbox';
import 'hardhat-watcher';
import 'hardhat-deploy';
import 'hardhat-deploy-ethers';

const accounts = process.env.PRIVATE_KEY
  ? [process.env.PRIVATE_KEY]
  : process.env.MNEMONIC
  ? { mnemonic: process.env.MNEMONIC }
  : [];

task('accounts').setAction(async (_, hre) => {
  const { ethers } = hre;
  const signers = await ethers.getSigners();
  const balances = await Promise.all(signers.map((s) => ethers.provider.getBalance(s.address)));
  for (let i = 0; i < signers.length; i++) {
    let num: string | number;
    try {
      num = balances[i].toNumber();
    } catch {
      num = ethers.utils.formatEther(balances[i]);
    }
    console.log(signers[i].address, num);
  }
});

task('set-trusted-sender')
  .addParam('sender')
  .setAction(async (args, hre) => {
    const { ethers } = hre;
    const attok = await ethers.getContract('AttestationToken');
    const tx = await attok.setTrustedSender(args.sender);
    console.log(tx.hash);
    const receipt = await tx.wait();
    if (receipt.status !== 1) throw new Error('failed');
  });

const config: HardhatUserConfig = {
  solidity: {
    version: '0.8.18',
    settings: {
      optimizer: {
        enabled: true,
      },
      viaIR: true,
    },
  },
  networks: {
    local: {
      url: 'http://127.0.0.1:8545',
    },
    'sapphire-testnet': {
      // url: 'https://testnet.sapphire.oasis.dev',
      url: 'http://127.0.0.1:8545',
      chainId: 0x5aff,
      accounts,
    },
    sapphire: {
      url: 'https://sapphire.oasis.io',
      chainId: 0x5afe,
      accounts,
    },
    hyperspace: {
      url: 'https://rpc.ankr.com/filecoin_testnet',
      chainId: 3141,
      accounts,
    },
  },
  watcher: {
    compile: {
      tasks: ['compile'],
    },
  },
  namedAccounts: {
    deployer: 0,
  },
};

export default config;
