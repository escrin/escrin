// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {ITaskHubV1} from "./ITaskHub.sol";
import {TaskHubHaver} from "./TaskHubHaver.sol";
import {TaskAcceptorHaver} from "./acceptor/Haver.sol";

// import "hardhat/console.sol";

contract TaskGeneratorV1 is TaskAcceptorHaver, TaskHubHaver {
    constructor(
        address _taskHub,
        address _taskAcceptor
    ) TaskHubHaver(_taskHub) TaskAcceptorHaver(_taskAcceptor) {
        return;
    }

    function _notifyTasksAvailable() internal {
        taskHub().notify();
    }
}
