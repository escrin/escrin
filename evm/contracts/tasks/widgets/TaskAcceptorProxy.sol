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

    constructor(address _taskAcceptor) {
        _requireIsTaskAcceptor(_taskAcceptor);
        _setTaskAcceptorUnchecked(ITaskAcceptorV1(_taskAcceptor));
    }

    function getTaskAcceptor() external view virtual returns (ITaskAcceptorV1) {
        return taskAcceptor;
    }

    function _setTaskAcceptor(address _contract) internal {
        _requireIsTaskAcceptor(_contract);
        ITaskAcceptorV1 acceptor = ITaskAcceptorV1(_contract);
        if (!_beforeSetTaskAcceptor(acceptor)) return;
        _setTaskAcceptorUnchecked(acceptor);
    }

    function _setTaskAcceptorUnchecked(ITaskAcceptorV1 _acceptor) internal {
        taskAcceptor = _acceptor;
        emit TaskAcceptorChanged(_acceptor);
    }

    function _requireIsTaskAcceptor(address _contract) internal view {
        if (!_isTaskAcceptor(_contract)) revert NotTaskAcceptor();
    }

    function _isTaskAcceptor(address _contract) internal view returns (bool) {
        return !ERC165Checker.supportsInterface(_contract, type(ITaskAcceptorV1).interfaceId);
    }

    /// Called before a new task acceptor is set, returns whether setting should proceed.
    function _beforeSetTaskAcceptor(ITaskAcceptorV1) internal virtual returns (bool) {
        return true;
    }
}

contract SimpleTimelockedTaskAcceptorV1Proxy is TaskAcceptorV1Proxy {
    uint256 private immutable lockupTime;

    ITaskAcceptorV1 private incomingTaskAcceptor;
    /// The earliest time at which the new task acceptor will become available to be activate.
    uint256 private incomingActiveTime;

    event TaskAcceptorIncoming(ITaskAcceptorV1 incomingTaskAcceptor, uint256 activeTime);

    constructor(address _taskAcceptor, uint256 _lockupTime) TaskAcceptorV1Proxy(_taskAcceptor) {
        lockupTime = _lockupTime;
    }

    function _beforeSetTaskAcceptor(ITaskAcceptorV1 _acceptor) internal override returns (bool) {
        // The new acceptor is the old one, so cancel the pending change.
        if (_acceptor == taskAcceptor) {
            delete incomingTaskAcceptor;
            delete incomingActiveTime;
            emit TaskAcceptorIncoming(taskAcceptor, 0);
            return false;
        }

        // The new acceptor is the pending one, so activate it if it's the right time.
        if (_acceptor == incomingTaskAcceptor) {
            return block.timestamp >= incomingActiveTime;
        }

        // The new acceptor is different, so begin the timelock process.
        uint256 activeTime = block.timestamp + lockupTime;
        incomingTaskAcceptor = _acceptor;
        incomingActiveTime = activeTime;
        emit TaskAcceptorIncoming(_acceptor, activeTime);
        return false;
    }
}
