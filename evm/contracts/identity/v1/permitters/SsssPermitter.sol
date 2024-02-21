// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IIdentityRegistry, IdentityId, Permitter} from "./Permitter.sol";

contract UnstableSsssPermitter is Permitter {
    /// The SSSS permitter does not respond directly to acquire/release identity requests.
    error Unsupported();

    event PolicyChange();
    event ApproverChange();

    uint256 public immutable creationBlock;

    constructor(address upstream) Permitter(upstream) {
        creationBlock = block.number;
    }

    function setPolicy(IdentityId identity, bytes calldata /* config */ ) external {
        (address registrant,) = IIdentityRegistry(_getIdentityRegistry()).getRegistrant(identity);
        if (msg.sender != registrant) revert Unauthorized();
        emit PolicyChange();
    }

    function _acquireIdentity(IdentityId, address, uint64, bytes calldata, bytes calldata)
        internal
        virtual
        override
        returns (uint64)
    {
        if (true) revert Unsupported();
        return 0;
    }

    function _releaseIdentity(IdentityId, address, bytes calldata, bytes calldata)
        internal
        virtual
        override
    {
        if (true) revert Unsupported();
    }
}
