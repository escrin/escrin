// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IIdentityRegistry, IdentityId} from "./IdentityRegistry.sol";
import {ExperimentalSsssPermitter} from "./permitters/SsssPermitter.sol";

contract ExperimentalSsssHub is ExperimentalSsssPermitter {
    event SharesDealt();

    constructor(address upstream) ExperimentalSsssPermitter(upstream) {}

    function dealShares(
        IdentityId identity,
        uint64 version,
        bytes calldata /* pk */,
        bytes32 /* nonce */,
        bytes[] calldata /* shares */
    ) external {
        (address registrant,) = IIdentityRegistry(_getIdentityRegistry()).getRegistrant(identity);
        if (msg.sender != registrant) revert Unauthorized();
        emit SharesDealt();
    }
}
