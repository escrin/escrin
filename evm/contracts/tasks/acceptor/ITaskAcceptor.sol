// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {TaskIdSelector} from "./TaskIdSelector.sol";

/// The contract is did not pass the ITaskAcceptor ERC-165 check.
error NotTaskAcceptor();

interface ITaskAcceptorV1 is IERC165 {
    /// Accepts zero or more tasks results.
    /// @param _taskIds a sorted set of taskIds completed in this submission
    /// @param _report Some data provided by the submitter that the requester may or may not trust
    function acceptTaskResults(
        uint256[] calldata _taskIds,
        bytes calldata _proof,
        bytes calldata _report,
        address _submitter
    ) external returns (TaskIdSelector memory);
}

/// An extension to `ITaskAcceptorV1` that helps task runners know where to find details about how to complete the task.
interface ITaskAcceptanceCriteria is ITaskAcceptorV1 {
    /// @return a string that could be a URI or some abi-encoded data
    function taskAcceptanceCriteria(uint256 _taskId) external view returns (string calldata);
}
