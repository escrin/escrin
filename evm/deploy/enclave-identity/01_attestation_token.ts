import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { DeployFunction } from 'hardhat-deploy/types';

const func: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployer } = await hre.getNamedAccounts();
  await hre.deployments.deploy('AttestationToken', {
    from: deployer,
    log: true,
    autoMine: true,
  });
};

func.tags = ['AttestationToken', 'enclave-identity'];

export default func;
