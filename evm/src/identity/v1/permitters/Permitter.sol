// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC165, IERC165} from "@openzeppelin/contracts/utils/introspection/ERC165.sol";
import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

import {InterfaceUnsupported} from "escrin/Types.sol";
import {IIdentityRegistry} from "../IIdentityRegistry.sol";
import {IPermitter} from "../IPermitter.sol";
import {IdentityId} from "../Types.sol";

abstract contract Permitter is IPermitter, ERC165 {
    IIdentityRegistry public immutable identityRegistry;

    constructor(address registry) {
        if (!ERC165Checker.supportsInterface(registry, type(IIdentityRegistry).interfaceId)) {
            revert InterfaceUnsupported();
        }
        identityRegistry = IIdentityRegistry(registry);
    }

    function acquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) external virtual override returns (uint64 expiry) {
        _beforeAcquireIdentity({
            identity: identity,
            requester: requester,
            duration: duration,
            context: context,
            authorization: authorization
        });
        expiry = _acquireIdentity({
            identity: identity,
            requester: requester,
            duration: duration,
            context: context,
            authorization: authorization
        });
        identityRegistry.grantIdentity(identity, requester, expiry);
        _afterAcquireIdentity(identity, requester, context);
    }

    function releaseIdentity(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) external virtual override {
        _beforeRevokePermit({
            identity: identity,
            requester: requester,
            context: context,
            authorization: authorization
        });
        _releaseIdentity({
            identity: identity,
            requester: requester,
            context: context,
            authorization: authorization
        });
        identityRegistry.revokeIdentity(identity, requester);
        _afterRevokePermit(identity, requester, context);
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

    /// Authorizes the identity acquisition request, returning the expiry if approved or reverting if denied.
    function _acquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual returns (uint64 expiry);

    /// Authorizes the identity release request, reverting if denied.
    function _releaseIdentity(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual;

    function _beforeAcquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual {}

    function _afterAcquireIdentity(IdentityId identity, address requester, bytes calldata context)
        internal
        virtual
    {}

    function _beforeRevokePermit(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual {}

    function _afterRevokePermit(IdentityId identity, address requester, bytes calldata context)
        internal
        virtual
    {}
}
