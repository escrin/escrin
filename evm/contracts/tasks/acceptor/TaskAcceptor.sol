// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {ITaskAcceptorV1, TaskIdSelectorOps} from "./ITaskAcceptor.sol";

/// The input task ids were not sorted.
error SubmisionTaskIdsNotSorted();
/// The set of accepted task ids was not sorted.
error AcceptedTaskIdsNotSorted();

abstract contract TaskAcceptorV1 is ITaskAcceptorV1 {
    using TaskIdSelectorOps for TaskIdSelector;

    function acceptTaskResults(
        uint256[] calldata _taskIds,
        bytes calldata _proof,
        bytes calldata _report
    ) external virtual returns (TaskIdSelector memory sel) {
        if (!_isSortedSet(_taskIds)) revert SubmisionTaskIdsNotSorted();
        _beforeTaskResultsAccepted(_taskIds, _proof, _report, msg.sender);
        sel = _acceptTaskResults(_taskIds, _proof, _report, msg.sender);
        if (!_isSortedSet(sel.taskIds)) revert AcceptedTaskIdsNotSorted();
        _afterTaskResultsAccepted(_taskIds, _report, msg.sender, sel);
    }

    /// Accepts one or more elements of a task runner's task results submission, returning the seto tasks that were accepted.
    /// @param _taskIds a sorted set of taskIds completed in this submission
    /// @param _proof some proof of having completed the identiied tasks that the acceptor can verify.
    /// @param _report Some data provided by the submitter that the requester may or may not trust
    /// @param _submitter The account that submitted the task results.
    /// @return A selection of the accepted task results, which may be empty.
    function _acceptTaskResults(
        uint256[] calldata _taskIds,
        bytes calldata _proof,
        bytes calldata _report,
        address _submitter
    ) internal virtual returns (TaskIdSelector memory);

    /// Runs before tasks are accepted.
    function _beforeTaskResultsAccepted(
        uint256[] calldata /* _taskIds */,
        bytes calldata /* _proof */,
        bytes calldata /* _report */,
        address /* _submitter */
    ) internal virtual {
        return;
    }

    function _afterTaskResultsAccepted(
        uint256[] calldata /* _taskIds */,
        bytes calldata /* _report */,
        address /* _submitter */,
        TaskIdSelector memory /* _selected */
    ) internal virtual {
        return;
    }

    function _isSortedSet(uint256[] memory _input) internal pure returns (bool) {
        for (uint256 i = 1; i < _input.length; ++i) {
            if (_input[i] <= _input[i - 1]) {
                return false;
            }
        }
        return true;
    }
}
