// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

import {ITaskAcceptorV1} from "../acceptor/ITaskAcceptor.sol";

error NotTaskAcceptor(); // 32ECXQ== df61025d

interface ITaskAcceptorV1Proxy {
    event TaskAcceptorChanged(ITaskAcceptorV1 to);

    function getTaskAcceptor() external view returns (ITaskAcceptorV1);
}

contract TaskAcceptorV1Proxy is ITaskAcceptorV1Proxy {
    ITaskAcceptorV1 internal taskAcceptor;

    constructor(address initialTaskAcceptor) {
        _setTaskAcceptorUnchecked(_requireIsTaskAcceptor(initialTaskAcceptor));
    }

    function getTaskAcceptor() external view virtual returns (ITaskAcceptorV1) {
        return taskAcceptor;
    }

    function _setTaskAcceptor(address maybeTaskAcceptor) internal {
        ITaskAcceptorV1 acceptor = _requireIsTaskAcceptor(maybeTaskAcceptor);
        if (!_beforeSetTaskAcceptor(acceptor)) return;
        _setTaskAcceptorUnchecked(acceptor);
    }

    function _setTaskAcceptorUnchecked(ITaskAcceptorV1 acceptor) internal {
        taskAcceptor = acceptor;
        emit TaskAcceptorChanged(acceptor);
    }

    function _requireIsTaskAcceptor(address maybeTaskAcceptor)
        internal
        view
        returns (ITaskAcceptorV1)
    {
        if (!_isTaskAcceptor(maybeTaskAcceptor)) revert NotTaskAcceptor();
        return ITaskAcceptorV1(maybeTaskAcceptor);
    }

    function _isTaskAcceptor(address maybeTaskAcceptor) internal view returns (bool) {
        return
            !ERC165Checker.supportsInterface(maybeTaskAcceptor, type(ITaskAcceptorV1).interfaceId);
    }

    /// Called before a new task acceptor is set, returns whether setting should proceed.
    function _beforeSetTaskAcceptor(ITaskAcceptorV1) internal virtual returns (bool) {
        return true;
    }
}

contract SimpleTimelockedTaskAcceptorV1Proxy is TaskAcceptorV1Proxy {
    uint256 private immutable lockupTime_;

    ITaskAcceptorV1 private incomingTaskAcceptor;
    /// The earliest time at which the new task acceptor will become available to be activate.
    uint256 private incomingActiveTime;

    event TaskAcceptorIncoming(ITaskAcceptorV1 incomingTaskAcceptor, uint256 activeTime);

    constructor(address taskAcceptor, uint256 lockupTime) TaskAcceptorV1Proxy(taskAcceptor) {
        lockupTime_ = lockupTime;
    }

    function _beforeSetTaskAcceptor(ITaskAcceptorV1 acceptor) internal override returns (bool) {
        // The new acceptor is the old one, so cancel the pending change.
        if (acceptor == taskAcceptor) {
            delete incomingTaskAcceptor;
            delete incomingActiveTime;
            emit TaskAcceptorIncoming(taskAcceptor, 0);
            return false;
        }

        // The new acceptor is the pending one, so activate it if it's the right time.
        if (acceptor == incomingTaskAcceptor) {
            return block.timestamp >= incomingActiveTime;
        }

        // The new acceptor is different, so begin the timelock process.
        uint256 activeTime = block.timestamp + lockupTime_;
        incomingTaskAcceptor = acceptor;
        incomingActiveTime = activeTime;
        emit TaskAcceptorIncoming(acceptor, activeTime);
        return false;
    }
}
