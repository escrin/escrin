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

    function _grantPermit(
        IdentityId identity,
        address relayer,
        address requester,
        bytes calldata context,
        bytes calldata authz
    ) internal virtual override returns (bool allow, uint64 expiry) {
        allow = _isTrustedRelayer(relayer);
        if (allow) {
            uint64 lifetime = _getPermitLifetime(identity, requester, context);
            expiry = uint64(block.timestamp + lifetime);
        }
    }

    function _revokePermit(
        IdentityId identity,
        address relayer,
        address requester,
        bytes calldata context,
        bytes calldata authz
    ) internal virtual override returns (bool allow) {
        return _isTrustedRelayer(relayer);
    }

    function _isTrustedRelayer(address addr) internal view virtual returns (bool) {
        return addr == trustedRelayer_;
    }

    function _getPermitLifetime(IdentityId identity, address requester, bytes calldata context)
        internal
        view
        virtual
        returns (uint64)
    {
        return 3 hours; // sane default
    }
}
