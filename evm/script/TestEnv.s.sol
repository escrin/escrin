// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import "forge-std/Script.sol";

import {IdentityId, IdentityRegistry} from "../contracts/identity/v1/IdentityRegistry.sol";
import {ExperimentalSsssHub} from "../contracts/identity/v1/SsssHub.sol";

contract Setup is Script {
    modifier broadcasted() {
        vm.startBroadcast(vm.envUint("PRIVATE_KEY"));
        _;
        vm.stopBroadcast();
    }

    function run() external broadcasted {
        IdentityRegistry registry = new IdentityRegistry();
        ExperimentalSsssHub ssss = new ExperimentalSsssHub(address(registry));
        registry.createIdentity(address(ssss), "");
    }
}
