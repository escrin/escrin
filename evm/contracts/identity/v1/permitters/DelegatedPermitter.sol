// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IPermitterProxy} from "./PermitterProxy.sol";
import {IdentityId, Permitter} from "./Permitter.sol";

// A random number prepended to the delegated context
uint256 constant DELEGATED_CONTEXT_MARKER =
    0x88e6f32f6512c8823c0e55100705d73f8d9ffe0bb853be5114997aa54b6b165c;

struct DelegatedContext {
    uint256 marker;
    address relayer;
    bytes origContext;
}

abstract contract DelegatedPermitter is Permitter, IPermitterProxy {
    function _grantPermit(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual override returns (bool allow, uint64 expiry) {
        return this.getPermitter().grantPermit({
            identity: identity,
            requester: requester,
            context: abi.encode(
                DelegatedContext({
                    marker: DELEGATED_CONTEXT_MARKER,
                    relayer: msg.sender,
                    origContext: context
                })
                ),
            authorization: authorization
        });
    }

    function _revokePermit(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual override returns (bool allow) {
        return this.getPermitter().revokePermit({
            identity: identity,
            requester: requester,
            context: abi.encode(
                DelegatedContext({
                    marker: DELEGATED_CONTEXT_MARKER,
                    relayer: msg.sender,
                    origContext: context
                })
                ),
            authorization: authorization
        });
    }
}
