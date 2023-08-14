// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

import {IPermitter} from "../IPermitter.sol";
import {InterfaceUnsupported} from "../Types.sol";

interface IPermitterProxy {
    event PermitterChanged(IPermitter to);

    function getPermitter() external view returns (IPermitter);
}

contract PermitterProxy is IPermitterProxy {
    IPermitter internal permitter_;

    constructor(address initialPermitter) {
        _setPermitterUnchecked(_requireIsPermitter(initialPermitter));
    }

    function getPermitter() external view virtual returns (IPermitter) {
        return permitter_;
    }

    function _setPermitter(address maybePermitter) internal {
        IPermitter permitter = _requireIsPermitter(maybePermitter);
        if (!_beforeSetPermitter(permitter)) return;
        _setPermitterUnchecked(permitter);
    }

    function _setPermitterUnchecked(IPermitter permitter) internal {
        permitter_ = permitter;
        emit PermitterChanged(permitter);
    }

    function _requireIsPermitter(address maybePermitter) internal view returns (IPermitter) {
        if (!_isPermitter(maybePermitter)) revert InterfaceUnsupported();
        return IPermitter(maybePermitter);
    }

    function _isPermitter(address maybePermitter) internal view returns (bool) {
        return ERC165Checker.supportsInterface(maybePermitter, type(IPermitter).interfaceId);
    }

    /// Called before a new permitter is set, returns whether setting should proceed.
    function _beforeSetPermitter(IPermitter) internal virtual returns (bool) {
        return true;
    }
}

contract SimpleTimelockedPermitterProxy is PermitterProxy {
    uint256 private immutable lockupTime_;

    IPermitter private incomingPermitter;
    /// The earliest time at which the new permitter will become available to be activate.
    uint256 private incomingActiveTime;

    event PermitterIncoming(IPermitter incomingPermitter, uint256 activeTime);

    constructor(address permitter, uint256 lockupTime) PermitterProxy(permitter) {
        lockupTime_ = lockupTime;
    }

    function _beforeSetPermitter(IPermitter permitter) internal override returns (bool) {
        // The new permitter is the old one, so cancel the pending change.
        if (permitter == permitter) {
            delete incomingPermitter;
            delete incomingActiveTime;
            emit PermitterIncoming(permitter, 0);
            return false;
        }

        // The new permitter is the pending one, so activate it if it's the right time.
        if (permitter == incomingPermitter) {
            return block.timestamp >= incomingActiveTime;
        }

        // The new permitter is different, so begin the timelock process.
        uint256 activeTime = block.timestamp + lockupTime_;
        incomingPermitter = permitter;
        incomingActiveTime = activeTime;
        emit PermitterIncoming(permitter, activeTime);
        return false;
    }
}
