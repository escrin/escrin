// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import "forge-std/Script.sol";

import {IdentityId, IdentityRegistry} from "../contracts/identity/v1/IdentityRegistry.sol";
import {SsssPermitter} from "../contracts/identity/v1/permitters/SsssPermitter.sol";

contract Setup is Script {
    modifier broadcasted() {
        vm.startBroadcast(vm.envUint("PRIVATE_KEY"));
        _;
        vm.stopBroadcast();
    }

    function run() external broadcasted {
        IdentityRegistry registry = new IdentityRegistry();
        SsssPermitter ssss = new SsssPermitter(address(registry));
        registry.createIdentity(address(ssss), "");
    }
}
