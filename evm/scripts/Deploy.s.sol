// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import "forge-std/Script.sol";

import {IdentityRegistry as IdentityRegistryV1} from '../src/identity/v1/IdentityRegistry.sol';
import {OmniKeyStore as OmniKeyStoreV1} from '../src/identity/v1/OmniKeyStore.sol';
import {TaskHub as TaskHubV1} from '../src/tasks/v1/hub/TaskHub.sol';

contract DeployV1 is Script {
  function run() external {
    vm.startBroadcast(vm.envUint("PRIVATE_KEY"));
    console2.log("task hub:", _deployTaskHubV1());
    console2.log("identity registry:", _deployIdentityRegistryV1());
    vm.stopBroadcast();
  }
}

contract DeployTaskHubV1 is Script {
  function run() external {
    vm.startBroadcast(vm.envUint("PRIVATE_KEY"));
    console2.log(_deployTaskHubV1());
    vm.stopBroadcast();
  }
}

function _deployTaskHubV1() returns (address) {
  return address(new TaskHubV1());
}

contract DeployIdentityRegistryV1 is Script {
  function run() external {
    vm.startBroadcast(vm.envUint("PRIVATE_KEY"));
    console2.log(_deployIdentityRegistryV1());
    vm.stopBroadcast();
  }
}

function _deployIdentityRegistryV1() returns (address) {
    bool isSapphire = block.chainid >= 0x5afd && block.chainid <= 0x5aff;
    if (isSapphire) return address(new OmniKeyStoreV1());
    return address(new IdentityRegistryV1());
}
