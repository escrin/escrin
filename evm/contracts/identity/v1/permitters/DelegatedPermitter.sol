// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IPermitterProxy} from "./PermitterProxy.sol";
import {IdentityId, Permitter} from "./Permitter.sol";

abstract contract DelegatedPermitter is Permitter, IPermitterProxy {
    function _grantPermit(
        IdentityId identity,
        address relayer,
        address requester,
        bytes calldata context,
        bytes calldata authz
    ) internal virtual override returns (bool allow, uint64 expiry) {
        // solhint-disable-next-line avoid-low-level-calls
        (bool success, bytes memory result) = address(this.getPermitter()).call(msg.data);
        if (!success) revert(string(result));
        return abi.decode(result, (bool, uint64));
    }

    function _revokePermit(
        IdentityId identity,
        address relayer,
        address requester,
        bytes calldata context,
        bytes calldata authz
    ) internal virtual override returns (bool allow) {
        // solhint-disable-next-line avoid-low-level-calls
        (bool success, bytes memory result) = address(this.getPermitter()).call(msg.data);
        if (!success) revert(string(result));
        return abi.decode(result, (bool));
    }
}
