import { HardhatUserConfig, task } from 'hardhat/config';
import '@nomicfoundation/hardhat-toolbox';
import 'hardhat-watcher';
import 'hardhat-deploy';
import 'hardhat-deploy-ethers';

const accounts = process.env.PRIVATE_KEY
  ? [process.env.PRIVATE_KEY]
  : process.env.MNEMONIC
  ? { mnemonic: process.env.MNEMONIC }
  : [];

task('accounts').setAction(async (_, { ethers }) => {
  const signers = await ethers.getSigners();
  const balances = await Promise.all(signers.map((s) => ethers.provider.getBalance(s.address)));
  for (let i = 0; i < signers.length; i++) {
    console.log(signers[i].address, ethers.formatEther(balances[i]));
  }
});

const config: HardhatUserConfig = {
  solidity: {
    version: '0.8.18',
    settings: {
      optimizer: {
        enabled: true,
        runs: Math.pow(2, 32) - 1,
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
      // url: 'https://sapphire.oasis.io',
      url: 'http://127.0.0.1:8545',
      chainId: 0x5afe,
      accounts,
    },
    hyperspace: {
      url: 'https://rpc.ankr.com/filecoin_testnet',
      chainId: 3141,
      accounts,
    },
    filecoin: {
      url: 'https://rpc.ankr.com/filecoin',
      chainId: 314,
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
