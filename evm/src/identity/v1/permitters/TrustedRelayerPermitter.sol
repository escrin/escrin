// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {Unauthorized} from "escrin/Types.sol";
import {IIdentityRegistry, IdentityId, Permitter} from "./Permitter.sol";

contract TrustedRelayerPermitter is Permitter {
    address private immutable trustedRelayer_;

    constructor(IIdentityRegistry registry, address trustedRelayer) Permitter(registry) {
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
    ) internal virtual override returns (uint64 expiry) {
        if (!_isTrustedRelayer(msg.sender)) revert Unauthorized();
        uint64 lifetime = _getPermitLifetime(identity, requester, duration, context);
        return uint64(block.timestamp + lifetime);
    }

    function _releaseIdentity(IdentityId, address, bytes calldata, bytes calldata)
        internal
        virtual
        override
    {
        if (!_isTrustedRelayer(msg.sender)) revert Unauthorized();
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
