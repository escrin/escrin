// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IIdentityRegistry, IdentityId} from "./IdentityRegistry.sol";
import {ExperimentalSsssPermitter} from "./permitters/SsssPermitter.sol";

contract ExperimentalSsssHub is ExperimentalSsssPermitter {
    event SharesDealt();

    mapping(IdentityId => mapping(string => uint256)) public versions;

    constructor(address upstream) ExperimentalSsssPermitter(upstream) {}

    function dealShares(
        IdentityId identity,
        string calldata secretName,
        uint64 version,
        bytes calldata, /* pk */
        bytes32, /* nonce */
        bytes[] calldata, /* shares */
        bytes[] calldata, /* commitments */
        bytes calldata /* userdata */
    ) external {
        versions[identity][secretName] += 1;
        require(version == versions[identity][secretName], "conflict");
        IIdentityRegistry.Permit memory permit =
            _getIdentityRegistry().readPermit(msg.sender, identity);
        if (permit.expiry <= block.timestamp) revert Unauthorized();
        emit SharesDealt();
    }

    // TODO: add gassless deal shares
}
