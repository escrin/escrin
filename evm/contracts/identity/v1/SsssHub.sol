// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IIdentityRegistry, IdentityId} from "./IdentityRegistry.sol";
import {ExperimentalSsssPermitter} from "./permitters/SsssPermitter.sol";

contract ExperimentalSsssHub is ExperimentalSsssPermitter {
    event SharesDealt();

    constructor(address upstream) ExperimentalSsssPermitter(upstream) {}

    function dealShares(
        IdentityId identity,
        uint64, /* version */
        bytes calldata, /* pk */
        bytes32, /* nonce */
        bytes[] calldata /* shares */
    ) external {
        IIdentityRegistry.Permit memory permit =
            _getIdentityRegistry().readPermit(msg.sender, identity);
        if (permit.expiry <= block.timestamp) revert Unauthorized();
        emit SharesDealt();
    }
}
