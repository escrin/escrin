// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IdentityId, Permitter} from "./Permitter.sol";

abstract contract TrustedRelayerPermitter is Permitter {
    address private immutable trustedRelayer_;

    constructor(address trustedRelayer) {
        trustedRelayer_ = trustedRelayer;
    }

    function getTrustedRelayer() external view returns (address) {
        return trustedRelayer_;
    }

    function _acquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata
    ) internal virtual override returns (bool allow, uint64 expiry) {
        allow = _isTrustedRelayer(msg.sender);
        if (allow) {
            uint64 lifetime = _getPermitLifetime(identity, requester, duration, context);
            expiry = uint64(block.timestamp + lifetime);
        }
    }

    function _releaseIdentity(IdentityId, address, bytes calldata, bytes calldata)
        internal
        virtual
        override
        returns (bool allow)
    {
        return _isTrustedRelayer(msg.sender);
    }

    function _isTrustedRelayer(address addr) internal view virtual returns (bool) {
        return addr == trustedRelayer_;
    }

    function _getPermitLifetime(
        IdentityId,
        address requester,
        uint64 requestedDuration,
        bytes calldata context
    ) internal view virtual returns (uint64) {
        (requester, context);
        return requestedDuration;
    }
}
