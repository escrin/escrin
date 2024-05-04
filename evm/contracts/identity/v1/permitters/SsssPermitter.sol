// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IIdentityRegistry, IdentityId, Permitter} from "./Permitter.sol";

contract ExperimentalSsssPermitter is Permitter {
    /// The SSSS permitter does not respond directly to acquire/release identity requests.
    error Unsupported(); // kKLK8g==

    event PolicyChange();
    event ApproverChange();

    mapping(IdentityId => bytes32) public policyHashes;
    mapping(IdentityId => bytes32) public approverRoots;

    constructor(address upstream) Permitter(upstream) {}

    modifier onlyRegistrant(IdentityId identity) {
        (address registrant,) = _getIdentityRegistry().getRegistrant(identity);
        if (msg.sender != registrant) revert Unauthorized();
        _;
    }

    function setPolicyHash(IdentityId identity, bytes32 policyHash)
        external
        onlyRegistrant(identity)
    {
        policyHashes[identity] = policyHash;
        emit PolicyChange();
    }

    function setApproversRoot(IdentityId identity, bytes32 approversRoot)
        external
        onlyRegistrant(identity)
    {
        approverRoots[identity] = approversRoot;
        emit ApproverChange();
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
