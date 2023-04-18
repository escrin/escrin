// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

/// The contract is did not pass the ITaskAcceptor ERC-165 check.
error NotTaskAcceptor();
error UnknownQuantifier();

interface ITaskAcceptorV1 is IERC165 {
    struct TaskIdSelector {
        Quantifier quantifier;
        /// A sorted list identifying subset of submitted tasks that will interpereted per the quantifier.
        uint256[] taskIds;
    }

    enum Quantifier {
        Unknown,
        All,
        None,
        Some,
        Excluding
    }

    /// Accepts one or more elements of a task runner's task results submission, returning the seto tasks that were accepted.
    /// @param _taskIds a sorted set of taskIds completed in this submission
    /// @param _proof some proof of having completed the identiied tasks that the acceptor can verify.
    /// @param _report some data provided by the submitter that the requester may or may not trust
    function acceptTaskResults(
        uint256[] calldata _taskIds,
        bytes calldata _proof,
        bytes calldata _report
    ) external returns (TaskIdSelector memory);
}

/// An extension to `ITaskAcceptorV1` that helps task runners know where to find details about how to complete the task.
interface ITaskAcceptanceCriteriaV1 is ITaskAcceptorV1 {
    /// @return a string that could be a URI or some abi-encoded data
    function taskAcceptanceCriteria(uint256 _taskId) external view returns (string calldata);
}

library TaskIdSelectorOps {
    function countSelected(
        ITaskAcceptorV1.TaskIdSelector memory _sel,
        uint256 _totalCount
    ) internal pure returns (uint256 count) {
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.All) return _totalCount;
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.None) return 0;
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.Some) return _sel.taskIds.length;
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.Excluding)
            return _totalCount - _sel.taskIds.length;
        revert UnknownQuantifier();
    }

    /// @param _set a sorted set of task ids
    function selected(
        ITaskAcceptorV1.TaskIdSelector memory _sel,
        uint256[] memory _set
    ) internal pure returns (uint256[] memory) {
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.All) return _set;
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.None) return new uint256[](0);
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.Some) return _sel.taskIds;
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.Excluding) {
            uint256[] memory out = new uint256[](countSelected(_sel, _set.length));
            uint256 selPtr;
            uint256 outPtr;
            for (uint256 setPtr; setPtr < _set.length; ++setPtr) {
                if (_set[setPtr] == _sel.taskIds[selPtr]) continue;
                out[outPtr] = _set[setPtr];
                selPtr++;
                outPtr++;
            }
            return out;
        }
        revert UnknownQuantifier();
    }

    function all() internal pure returns (ITaskAcceptorV1.TaskIdSelector memory sel) {
        sel.quantifier = ITaskAcceptorV1.Quantifier.All;
    }

    function none() internal pure returns (ITaskAcceptorV1.TaskIdSelector memory sel) {
        sel.quantifier = ITaskAcceptorV1.Quantifier.None;
    }
}
