// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC165, IERC165} from "@openzeppelin/contracts/utils/introspection/ERC165.sol";

import {IPermitter} from "../IPermitter.sol";
import {IdentityId} from "../Types.sol";

abstract contract Permitter is IPermitter, ERC165 {
    function grantPermit(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) external virtual override returns (bool allow, uint64 expiry) {
        _beforeGrantPermit({
            identity: identity,
            requester: requester,
            context: context,
            authorization: authorization
        });
        (allow, expiry) = _grantPermit({
            identity: identity,
            requester: requester,
            context: context,
            authorization: authorization
        });
        _afterGrantPermit(identity, requester, context, allow);
    }

    function revokePermit(
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
        allow = _revokePermit({
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

    function _grantPermit(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual returns (bool allow, uint64 expiry);

    function _revokePermit(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual returns (bool allow);

    function _beforeGrantPermit(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual {}

    function _afterGrantPermit(
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
