// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {IPermitter} from "../IPermitter.sol";
import {IdentityId} from "../Types.sol";

abstract contract Permitter is IPermitter {
    function grantPermit(
        IdentityId identity,
        address relayer,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) external virtual returns (bool allow, uint64 expiry) {
        _beforeGrantPermit({
            identity: identity,
            relayer: relayer,
            requester: requester,
            context: context,
            authorization: authorization
        });
        (allow, expiry) = _grantPermit({
            identity: identity,
            relayer: relayer,
            requester: requester,
            context: context,
            authorization: authorization
        });
        _afterGrantPermit(identity, requester, context, allow);
    }

    function revokePermit(
        IdentityId identity,
        address relayer,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) external virtual returns (bool allow) {
        _beforeRevokePermit({
            identity: identity,
            relayer: relayer,
            requester: requester,
            context: context,
            authorization: authorization
        });
        allow = _revokePermit({
            identity: identity,
            relayer: relayer,
            requester: requester,
            context: context,
            authorization: authorization
        });
        _afterRevokePermit(identity, requester, context, allow);
    }

    function supportsInterface(bytes4 interfaceId) external pure virtual override returns (bool) {
        return interfaceId == type(IPermitter).interfaceId;
    }

    function _grantPermit(
        IdentityId identity,
        address relayer,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual returns (bool allow, uint64 expiry);

    function _revokePermit(
        IdentityId identity,
        address relayer,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual returns (bool allow);

    function _beforeGrantPermit(
        IdentityId identity,
        address relayer,
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
        address relayer,
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
