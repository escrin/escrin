// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {TaskAcceptor, TaskIdSelectorOps} from "./TaskAcceptor.sol";

contract StaticDelegatedTaskAcceptor is TaskAcceptor {
    address public immutable upstream;

    constructor(address trustedUpstream) {
        upstream = trustedUpstream;
    }

    function _acceptTaskResults(uint256[] calldata, bytes calldata, bytes calldata)
        internal
        virtual
        override
        returns (TaskIdSelector memory sel)
    {
        if (msg.sender != upstream) revert Unauthorized();
        return TaskIdSelectorOps.all();
    }
}

contract DelegatedTaskAcceptor is TaskAcceptor {
    event UpstreamChanged();

    address public upstream;

    constructor(address trustedUpstream) {
        upstream = trustedUpstream;
    }

    function _acceptTaskResults(uint256[] calldata, bytes calldata, bytes calldata)
        internal
        virtual
        override
        returns (TaskIdSelector memory sel)
    {
        if (msg.sender != upstream) revert Unauthorized();
        return TaskIdSelectorOps.all();
    }

    function _setUpstreamTaskAcceptor(address trustedUpstream) internal virtual {
        upstream = trustedUpstream;
    }
}

contract TimelockedDelegatedTaskAcceptor is DelegatedTaskAcceptor {
    event DelayChanged();

    uint64 public delay;

    struct IncomingUpstream {
        address addr;
        uint64 activationTime;
    }

    IncomingUpstream public incomingUpstream;

    struct IncomingDelay {
        uint64 delay;
        uint64 activationTime;
    }

    IncomingDelay public incomingDelay;

    constructor(address upstream, uint64 timelockDelay) DelegatedTaskAcceptor(upstream) {
        delay = timelockDelay;
    }

    function _acceptTaskResults(uint256[] calldata, bytes calldata, bytes calldata)
        internal
        virtual
        override
        returns (TaskIdSelector memory sel)
    {
        if (msg.sender != upstream) revert Unauthorized();
        return TaskIdSelectorOps.all();
    }

    function _setUpstreamTaskAcceptor(address newUpstream) internal override {
        IncomingUpstream storage u = incomingUpstream;
        if (u.addr == newUpstream) {
            if (u.activationTime < block.timestamp) revert Unauthorized();
            upstream = u.addr;
            u.addr = address(0);
            u.activationTime = 0;
        } else {
            u.addr = newUpstream;
            u.activationTime = uint64(block.timestamp) + delay;
        }
        emit UpstreamChanged();
    }

    function _setTaskAcceptorTimelockDelay(uint64 newDelay) internal {
        IncomingDelay storage d = incomingDelay;
        if (d.delay == newDelay) {
            if (d.activationTime < block.timestamp) revert Unauthorized();
            delay = d.delay;
            d.delay = 0;
            d.activationTime = 0;
        } else {
            d.delay = newDelay;
            d.activationTime = uint64(block.timestamp) + delay;
        }
        emit DelayChanged();
    }
}
