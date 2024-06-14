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
            salt: 0x02a1cc53de2aa4dee9cd7aab6be390fccf16cf4eeae806be5cb401890b1e2ef5
        }();
        ssss = new SsssPermitter{
            salt: 0x2dff154b73951a9e570614c000575518af79837c0a311fb79fa728ba126ad19e
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
