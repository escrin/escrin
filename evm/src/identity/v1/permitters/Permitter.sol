// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC165, IERC165} from "@openzeppelin/contracts/utils/introspection/ERC165.sol";

import {IPermitter} from "../IPermitter.sol";
import {IdentityId} from "../Types.sol";

abstract contract Permitter is IPermitter, ERC165 {
    function acquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) external virtual override returns (bool allow, uint64 expiry) {
        _beforeAcquireIdentity({
            identity: identity,
            requester: requester,
            duration: duration,
            context: context,
            authorization: authorization
        });
        (allow, expiry) = _acquireIdentity({
            identity: identity,
            requester: requester,
            duration: duration,
            context: context,
            authorization: authorization
        });
        _afterAcquireIdentity(identity, requester, context, allow);
    }

    function releaseIdentity(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) external virtual override returns (bool allow) {
        _beforeRevokePermit({
            identity: identity,
            requester: requester,
            context: context,
            authorization: authorization
        });
        allow = _releaseIdentity({
            identity: identity,
            requester: requester,
            context: context,
            authorization: authorization
        });
        _afterRevokePermit(identity, requester, context, allow);
    }

    function supportsInterface(bytes4 interfaceId)
        public
        view
        virtual
        override(ERC165, IERC165)
        returns (bool)
    {
        return interfaceId == type(IPermitter).interfaceId || super.supportsInterface(interfaceId);
    }

    function _acquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual returns (bool allow, uint64 expiry);

    function _releaseIdentity(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual returns (bool allow);

    function _beforeAcquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual {}

    function _afterAcquireIdentity(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bool decision
    ) internal virtual {}

    function _beforeRevokePermit(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual {}

    function _afterRevokePermit(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bool decision
    ) internal virtual {}
}
