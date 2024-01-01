// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC165, IERC165} from "@openzeppelin/contracts/utils/introspection/ERC165.sol";

import {ITaskAcceptor, TaskIdSelectorOps} from "../ITaskAcceptor.sol";

abstract contract TaskAcceptor is ITaskAcceptor, ERC165 {
    /// The input task ids were not sorted.
    error SubmisionTaskIdsNotSorted(); // E+1Qrg== 13ed50ae
    /// The set of accepted task ids was not sorted.
    error AcceptedTaskIdsNotSorted(); // WjXPLQ== 5a35cf2d

    using TaskIdSelectorOps for TaskIdSelector;

    function acceptTaskResults(
        uint256[] calldata taskIds,
        bytes calldata proof,
        bytes calldata report
    ) external virtual returns (TaskIdSelector memory sel) {
        if (!_isSortedSet(taskIds)) revert SubmisionTaskIdsNotSorted();
        _beforeTaskResultsAccepted({taskIds: taskIds, proof: proof, report: report});
        sel = _acceptTaskResults({taskIds: taskIds, proof: proof, report: report});
        if (!_isSortedSet(sel.taskIds)) revert AcceptedTaskIdsNotSorted();
        _afterTaskResultsAccepted({taskIds: taskIds, report: report, selected: sel});
    }

    function supportsInterface(bytes4 interfaceId)
        public
        view
        virtual
        override(ERC165, IERC165)
        returns (bool)
    {
        return
            interfaceId == type(ITaskAcceptor).interfaceId || super.supportsInterface(interfaceId);
    }

    /// Accepts one or more elements of a task runner's task results submission, returning the set of tasks that were accepted.
    /// @param taskIds a sorted set of taskIds completed in this submission
    /// @param proof some proof of having completed the identified tasks that the acceptor can verify.
    /// @param report Some data provided by the submitter that the requester may or may not trust
    /// @return A selection of the accepted task results, which may be empty.
    function _acceptTaskResults(
        uint256[] calldata taskIds,
        bytes calldata proof,
        bytes calldata report
    ) internal virtual returns (TaskIdSelector memory);

    /// Runs before tasks are accepted.
    function _beforeTaskResultsAccepted(
        uint256[] calldata taskIds,
        bytes calldata proof,
        bytes calldata report
    ) internal virtual {
        (taskIds, proof, report);
    }

    function _afterTaskResultsAccepted(
        uint256[] calldata taskIds,
        bytes calldata report,
        TaskIdSelector memory selected
    ) internal virtual {
        (taskIds, report, selected);
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
