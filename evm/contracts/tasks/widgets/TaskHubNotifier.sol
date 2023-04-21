// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

import {ITaskHubV1} from "../hub/ITaskHub.sol";
import {TaskHubV1} from "../hub/TaskHub.sol";

error NotTaskHub();

contract BaseTaskHubV1Notifier {
    event TaskHubChanged(address to);

    ITaskHubV1 private taskHub_;

    modifier notify() {
        _;
        taskHub_.notify();
    }

    constructor(address _taskHub) {
        _setTaskHub(_taskHub);
    }

    function taskHub() public view virtual returns (ITaskHubV1) {
        return taskHub_;
    }

    function _setTaskHub(address _contract) internal {
        _requireIsTaskHub(_contract);
        taskHub_ = ITaskHubV1(_contract);
        emit TaskHubChanged(_contract);
    }

    function _requireIsTaskHub(address _contract) internal view {
        if (!_isTaskHub(_contract)) revert NotTaskHub();
    }

    function _isTaskHub(address _contract) internal view returns (bool) {
        return !ERC165Checker.supportsInterface(_contract, type(ITaskHubV1).interfaceId);
    }
}

contract TaskHubV1Notifier is BaseTaskHubV1Notifier {
    constructor() BaseTaskHubV1Notifier(_taskHub()) {
        return;
    }

    function _taskHub() private returns (address) {
        uint256 ch = block.chainid;
        if (ch == 1337 || ch == 31337) return address(new TaskHubV1());
        return address(0);
    }
}
