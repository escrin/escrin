// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {TaskAcceptor, TaskIdSelectorOps} from "./TaskAcceptor.sol";

abstract contract StaticDelegatedTaskAcceptor is TaskAcceptor {
    address internal immutable upstream;

    constructor(address trustedUpstream) {
        upstream = trustedUpstream;
    }

    function getUpstreamTaskAcceptor() external view returns (address) {
        return upstream;
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

abstract contract DelegatedTaskAcceptor is TaskAcceptor {
    event UpstreamChanged(address);

    address internal upstream;

    constructor(address trustedUpstream) {
        _setUpstreamTaskAcceptor(trustedUpstream);
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

    function _setUpstreamTaskAcceptor(address newUpstream) internal virtual {
        upstream = newUpstream;
        emit UpstreamChanged(newUpstream);
    }
}

abstract contract TimelockedDelegatedTaskAcceptor is DelegatedTaskAcceptor {
    event UpstreamIncoming(address);
    event DelayIncoming(uint64);
    event DelayChanged(uint64);

    struct IncomingUpstream {
        address addr;
        uint64 activationTime;
    }

    struct IncomingDelay {
        uint64 delay;
        uint64 activationTime;
    }

    uint64 internal delay;
    IncomingUpstream private incomingUpstream;
    IncomingDelay private incomingDelay;

    constructor(address upstream, uint64 timelockDelay) DelegatedTaskAcceptor(upstream) {
        delay = timelockDelay;
        emit DelayChanged(delay);
    }

    function _setUpstreamTaskAcceptor(address newUpstream) internal override {
        IncomingUpstream storage u = incomingUpstream;
        if (u.addr != newUpstream) {
            (u.addr, u.activationTime) = (newUpstream, uint64(block.timestamp + delay));
            emit UpstreamIncoming(newUpstream);
            return;
        }
        if (u.activationTime > block.timestamp) revert Unauthorized();
        super._setUpstreamTaskAcceptor(newUpstream);
        (u.addr, u.activationTime) = (address(0), 0);
    }

    function _setTaskAcceptorTimelockDelay(uint64 newDelay) internal {
        IncomingDelay storage d = incomingDelay;
        if (d.delay != newDelay) {
            (d.delay, d.activationTime) = (newDelay, uint64(block.timestamp + delay));
            emit DelayIncoming(newDelay);
            return;
        }
        if (d.activationTime > block.timestamp) revert Unauthorized();
        (delay, d.delay, d.activationTime) = (d.delay, 0, 0);
        emit DelayChanged(newDelay);
    }
}
