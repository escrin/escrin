// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {DELEGATED_CONTEXT_MARKER} from "./DelegatedPermitter.sol";
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
        address requester,
        bytes calldata context,
        bytes calldata
    ) internal virtual override returns (bool allow, uint64 expiry) {
        address relayer = msg.sender;
        if (context.length > 159) {
            uint256 maybeMarker;
            assembly {
                maybeMarker := mload(add(context.offset, 64))
            }
            if (maybeMarker == DELEGATED_CONTEXT_MARKER) {
                assembly {
                    relayer := mload(add(context.offset, 96))
                }
            }
        }
        allow = _isTrustedRelayer(relayer);
        if (allow) {
            uint64 lifetime = _getPermitLifetime(identity, requester, context);
            expiry = uint64(block.timestamp + lifetime);
        }
    }

    function _revokePermit(IdentityId, address, bytes calldata, bytes calldata)
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

    function _getPermitLifetime(IdentityId, address, /*requester*/ bytes calldata /*context*/ )
        internal
        view
        virtual
        returns (uint64)
    {
        return 3 hours; // sane default
    }
}
