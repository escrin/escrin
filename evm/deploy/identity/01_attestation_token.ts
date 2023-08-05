import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { DeployFunction } from 'hardhat-deploy/types';

const func: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployer } = await hre.getNamedAccounts();
  let trustedSender = deployer;
  if (hre.network.live) {
    trustedSender = process.env.ATTESTATION_TOKEN_TRUSTED_SENDER ?? '';
    if (!trustedSender) throw new Error('ATTESTATION_TOKEN_TRUSTED_SENDER not set');
  }
  await hre.deployments.deploy('AttestationToken', {
    from: deployer,
    args: [trustedSender],
    log: true,
    autoMine: true,
  });
};

func.tags = ['AttestationToken', 'identity'];

export default func;
