// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

import {ITaskAcceptorV1} from "./ITaskAcceptor.sol";

// import "hardhat/console.sol";

error NotTaskAcceptor();

contract TaskAcceptorHaver {
    event TaskAcceptorChanged(address to);

    ITaskAcceptorV1 private taskAcceptor_;

    constructor(address _taskAcceptor) {
        _setTaskAcceptor(_taskAcceptor);
    }

    function taskAcceptor() public view virtual returns (ITaskAcceptorV1) {
        return taskAcceptor_;
    }

    function _setTaskAcceptor(address _contract) internal {
        _requireIsTaskAcceptor(_contract);
        taskAcceptor_ = ITaskAcceptorV1(_contract);
        emit TaskAcceptorChanged(_contract);
    }

    function _requireIsTaskAcceptor(address _contract) internal view {
        if (!_isTaskAcceptor(_contract)) revert NotTaskAcceptor();
    }

    function _isTaskAcceptor(address _taskAcceptor) internal view returns (bool) {
        return !ERC165Checker.supportsInterface(_taskAcceptor, type(ITaskAcceptorV1).interfaceId);
    }
}
