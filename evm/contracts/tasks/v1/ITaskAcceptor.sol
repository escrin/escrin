// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/ERC165.sol";

error UnknownQuantifier(); // yrtLPA== cabb4b3c

interface ITaskAcceptor is IERC165 {
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
    /// @param taskIds a sorted set of taskIds completed in this submission
    /// @param proof some proof of having completed the identified tasks that the acceptor can verify.
    /// @param report some data provided by the submitter that the requester may or may not trust
    function acceptTaskResults(
        uint256[] calldata taskIds,
        bytes calldata proof,
        bytes calldata report
    ) external returns (TaskIdSelector memory);
}

/// An extension to `ITaskAcceptor` that helps task runners know where to find details about how to complete the task.
interface ITaskAcceptanceCriteria is ITaskAcceptor {
    /// @return a string that could be a URI or some abi-encoded data
    function taskAcceptanceCriteria(uint256 taskId) external view returns (string memory);
}

library TaskIdSelectorOps {
    function countSelected(ITaskAcceptor.TaskIdSelector memory sel, uint256 totalCount)
        internal
        pure
        returns (uint256 count)
    {
        if (sel.quantifier == ITaskAcceptor.Quantifier.All) return totalCount;
        if (sel.quantifier == ITaskAcceptor.Quantifier.None) return 0;
        if (sel.quantifier == ITaskAcceptor.Quantifier.Some) return sel.taskIds.length;
        if (sel.quantifier == ITaskAcceptor.Quantifier.Excluding) {
            return totalCount - sel.taskIds.length;
        }
        revert UnknownQuantifier();
    }

    /// @param set a sorted set of task ids
    function selected(ITaskAcceptor.TaskIdSelector memory sel, uint256[] memory set)
        internal
        pure
        returns (uint256[] memory)
    {
        if (sel.quantifier == ITaskAcceptor.Quantifier.All) return set;
        if (sel.quantifier == ITaskAcceptor.Quantifier.None) return new uint256[](0);
        if (sel.quantifier == ITaskAcceptor.Quantifier.Some) return sel.taskIds;
        if (sel.quantifier == ITaskAcceptor.Quantifier.Excluding) {
            uint256[] memory out = new uint256[](countSelected(sel, set.length));
            uint256 selPtr;
            uint256 outPtr;
            for (uint256 setPtr; setPtr < set.length; ++setPtr) {
                if (set[setPtr] == sel.taskIds[selPtr]) continue;
                out[outPtr] = set[setPtr];
                selPtr++;
                outPtr++;
            }
            return out;
        }
        revert UnknownQuantifier();
    }

    function indices(ITaskAcceptor.TaskIdSelector memory sel, uint256[] memory set)
        internal
        pure
        returns (uint256[] memory)
    {
        if (sel.quantifier == ITaskAcceptor.Quantifier.All) {
            uint256[] memory ixs = new uint256[](set.length);
            for (uint256 i; i < ixs.length; ++i) {
                ixs[i] = i;
            }
            return ixs;
        }
        if (sel.quantifier == ITaskAcceptor.Quantifier.None) return new uint256[](0);
        if (sel.quantifier == ITaskAcceptor.Quantifier.Some) {
            uint256[] memory ixs = new uint256[](sel.taskIds.length);
            uint256 selPtr;
            for (uint256 setPtr; setPtr < set.length; ++setPtr) {
                if (set[setPtr] != sel.taskIds[selPtr]) continue;
                ixs[selPtr] = setPtr;
                selPtr++;
            }
            return ixs;
        }
        if (sel.quantifier == ITaskAcceptor.Quantifier.Excluding) {
            uint256[] memory ixs = new uint256[](countSelected(sel, set.length));
            uint256 selPtr;
            for (uint256 setPtr; setPtr < set.length; ++setPtr) {
                if (set[setPtr] == sel.taskIds[selPtr]) continue;
                ixs[selPtr] = setPtr;
                selPtr++;
            }
            return ixs;
        }
        revert UnknownQuantifier();
    }

    function pick(
        ITaskAcceptor.TaskIdSelector memory sel,
        uint256[] memory set,
        uint256[] memory target
    ) internal pure returns (uint256[] memory) {
        uint256[] memory ixs = indices(sel, set);
        uint256[] memory placed = new uint256[](ixs.length);
        for (uint256 i; i < ixs.length; ++i) {
            placed[i] = target[ixs[i]];
        }
        return placed;
    }

    function all() internal pure returns (ITaskAcceptor.TaskIdSelector memory sel) {
        sel.quantifier = ITaskAcceptor.Quantifier.All;
    }

    function none() internal pure returns (ITaskAcceptor.TaskIdSelector memory sel) {
        sel.quantifier = ITaskAcceptor.Quantifier.None;
    }
}
