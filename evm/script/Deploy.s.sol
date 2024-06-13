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

    function _deployRegistryAndSsssPermitter()
        internal
        returns (IdentityRegistry registry, SsssPermitter ssss)
    {
        registry = new IdentityRegistry{
            salt: 0xe8b9c6b0f67561f7bd9414113e46cb43f117ad1bef1990f864c327caa35ef2de
        }();
        ssss = new SsssPermitter{
            salt: 0x8c2f1b8b9b47e64b3c1f5dbed4b4c4861e8e5403cf79be45d268251ca51e8783
        }(address(registry));
    }
}

contract SetupNewChain is Setup {
    function run() external broadcasted {
        (IdentityRegistry registry, SsssPermitter ssss) = _deployRegistryAndSsssPermitter();
        console2.log("registry:", address(registry));
        console2.log("ssss permitter:", address(ssss));
    }
}

contract SetupTestEnv is Setup {
    function run() external broadcasted {
        (IdentityRegistry registry, SsssPermitter ssss) = _deployRegistryAndSsssPermitter();
        IdentityId identity = registry.createIdentity(address(ssss), "");
        console2.log("registry:", address(registry));
        console2.log("ssss permitter:", address(ssss));
        console2.log("identity:");
        console2.logBytes32(IdentityId.unwrap(identity));
    }
}
