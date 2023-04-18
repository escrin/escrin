// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {ITaskGeneratorV1} from "./ITaskGenerator.sol";
import {TaskIdSelector} from "./TaskIdSelector.sol";

struct TaskSubmissionV1 {
    uint256[] taskIds;
    bytes report;
}

interface ITaskHubV1 is IERC165 {
    event TasksAvailable(address indexed generator, bytes32 indexed context);

    /// Alerts any listening task runners that there are new tasks available.
    function notify() external;

    /// Called by task runners to submit results to requesters.
    /// @param _taskIds a sorted set of task ids.
    /// @param _report Some data provided by the submitter that the requester may or may not trust. It's provided after payment is verified.
    function submitTaskResults(
        ITaskGeneratorV1 requester,
        uint256[] calldata _taskIds,
        bytes calldata _proof,
        bytes calldata _report
    ) external returns (TaskIdSelector memory);
}
