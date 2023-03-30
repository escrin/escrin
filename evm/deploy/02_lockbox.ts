import { deployments } from 'hardhat';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { DeployFunction } from 'hardhat-deploy/types';

const func: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployer } = await hre.getNamedAccounts();
  const attok = await deployments.get('AttestationToken');
  await hre.deployments.deploy('Lockbox', {
    from: deployer,
    args: [attok.address],
    log: true,
    autoMine: true,
  });
};

func.tags = ['Lockbox'];
func.dependencies = ['AttestationToken'];

export default func;

