import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import "hardhat-watcher";

const config: HardhatUserConfig = {
  solidity: "0.8.18",
  watcher: {
    compile: {
      tasks: ['compile'],
    },
  }
};

export default config;
