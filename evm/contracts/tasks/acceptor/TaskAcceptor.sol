// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {ITaskAcceptorV1, TaskIdSelectorOps} from "./ITaskAcceptor.sol";

/// The input task ids were not sorted.
error SubmisionTaskIdsNotSorted(); // E+1Qrg== 13ed50ae
/// The set of accepted task ids was not sorted.
error AcceptedTaskIdsNotSorted(); // WjXPLQ== 5a35cf2d

abstract contract TaskAcceptorV1 is ITaskAcceptorV1 {
    using TaskIdSelectorOps for TaskIdSelector;

    function acceptTaskResults(
        uint256[] calldata taskIds,
        Proof calldata proof,
        Report calldata report
    ) external virtual returns (TaskIdSelector memory sel) {
        if (!_isSortedSet(taskIds)) revert SubmisionTaskIdsNotSorted();
        _beforeTaskResultsAccepted(taskIds, proof, report, msg.sender);
        sel = _acceptTaskResults(taskIds, proof, report, msg.sender);
        if (!_isSortedSet(sel.taskIds)) revert AcceptedTaskIdsNotSorted();
        _afterTaskResultsAccepted(taskIds, report, msg.sender, sel);
    }

    /// Accepts one or more elements of a task runner's task results submission, returning the set of tasks that were accepted.
    /// @param taskIds a sorted set of taskIds completed in this submission
    /// @param proof some proof of having completed the identified tasks that the acceptor can verify.
    /// @param report Some data provided by the submitter that the requester may or may not trust
    /// @param submitter The account that submitted the task results.
    /// @return A selection of the accepted task results, which may be empty.
    function _acceptTaskResults(
        uint256[] calldata taskIds,
        Proof calldata proof,
        Report calldata report,
        address submitter
    ) internal virtual returns (TaskIdSelector memory);

    /// Runs before tasks are accepted.
    function _beforeTaskResultsAccepted(
        uint256[] calldata, /* taskIds */
        Proof calldata,
        Report calldata,
        address /* submitter */
    ) internal virtual {
        return;
    }

    function _afterTaskResultsAccepted(
        uint256[] calldata, /* taskIds */
        Report calldata,
        address, /* submitter */
        TaskIdSelector memory /* selected */
    ) internal virtual {
        return;
    }

    function _isSortedSet(uint256[] memory input) internal pure returns (bool) {
        for (uint256 i = 1; i < input.length; ++i) {
            if (input[i] <= input[i - 1]) {
                return false;
            }
        }
        return true;
    }
}
