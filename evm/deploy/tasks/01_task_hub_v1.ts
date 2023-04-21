import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { DeployFunction } from 'hardhat-deploy/types';

const func: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployer } = await hre.getNamedAccounts();
  await hre.deployments.deploy('TaskHubV1', {
    from: deployer,
    log: true,
    autoMine: true,
  });
};

func.tags = ['TaskHubV1', 'tasks'];

export default func;
