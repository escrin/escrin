// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

error UnknownQuantifier(); // yrtLPA== cabb4b3c

interface ITaskAcceptorV1 {
    struct TaskIdSelector {
        Quantifier quantifier;
        /// A sorted list identifying subset of submitted tasks that will interpreted per the quantifier.
        uint256[] taskIds;
    }

    enum Quantifier {
        Unknown,
        All,
        None,
        Some,
        Excluding
    }

    /// Accepts one or more elements of a task runner's task results submission, returning the set of tasks that were accepted.
    /// @param _taskIds a sorted set of taskIds completed in this submission
    /// @param _proof some proof of having completed the identified tasks that the acceptor can verify.
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
    function taskAcceptanceCriteria(uint256 _taskId) external view returns (string memory);
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

    function indices(
        ITaskAcceptorV1.TaskIdSelector memory _sel,
        uint256[] memory _set
    ) internal pure returns (uint256[] memory) {
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.All) {
            uint256[] memory ixs = new uint256[](_set.length);
            for (uint256 i; i < ixs.length; ++i) ixs[i] = i;
            return ixs;
        }
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.None) return new uint256[](0);
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.Some) {
            uint256[] memory ixs = new uint256[](_sel.taskIds.length);
            uint256 selPtr;
            for (uint256 setPtr; setPtr < _set.length; ++setPtr) {
                if (_set[setPtr] != _sel.taskIds[selPtr]) continue;
                ixs[selPtr] = setPtr;
                selPtr++;
            }
            return ixs;
        }
        if (_sel.quantifier == ITaskAcceptorV1.Quantifier.Excluding) {
            uint256[] memory ixs = new uint256[](countSelected(_sel, _set.length));
            uint256 selPtr;
            for (uint256 setPtr; setPtr < _set.length; ++setPtr) {
                if (_set[setPtr] == _sel.taskIds[selPtr]) continue;
                ixs[selPtr] = setPtr;
                selPtr++;
            }
            return ixs;
        }
        revert UnknownQuantifier();
    }

    function pick(
        ITaskAcceptorV1.TaskIdSelector memory _sel,
        uint256[] memory _set,
        uint256[] memory _target
    ) internal pure returns (uint256[] memory) {
        uint256[] memory ixs = indices(_sel, _set);
        uint256[] memory placed = new uint256[](ixs.length);
        for (uint256 i; i < ixs.length; ++i) {
            placed[i] = _target[ixs[i]];
        }
        return placed;
    }

    function all() internal pure returns (ITaskAcceptorV1.TaskIdSelector memory sel) {
        sel.quantifier = ITaskAcceptorV1.Quantifier.All;
    }

    function none() internal pure returns (ITaskAcceptorV1.TaskIdSelector memory sel) {
        sel.quantifier = ITaskAcceptorV1.Quantifier.None;
    }
}
