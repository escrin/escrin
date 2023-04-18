// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

error UnknownQuantifier();

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

library TaskIdSelectorOps {
    function countSelected(
        TaskIdSelector memory _sel,
        uint256 _totalCount
    ) internal pure returns (uint256 count) {
        if (_sel.quantifier == Quantifier.All) return _totalCount;
        if (_sel.quantifier == Quantifier.None) return 0;
        if (_sel.quantifier == Quantifier.Some) return _sel.taskIds.length;
        if (_sel.quantifier == Quantifier.Excluding) return _totalCount - _sel.taskIds.length;
        revert UnknownQuantifier();
    }

    /// @param _set a sorted set of task ids
    function selected(
        TaskIdSelector memory _sel,
        uint256[] memory _set
    ) internal pure returns (uint256[] memory) {
        if (_sel.quantifier == Quantifier.All) return _set;
        if (_sel.quantifier == Quantifier.None) return new uint256[](0);
        if (_sel.quantifier == Quantifier.Some) return _sel.taskIds;
        if (_sel.quantifier == Quantifier.Excluding) {
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

    function all() internal pure returns (TaskIdSelector memory sel) {
        sel.quantifier = Quantifier.All;
    }

    function none() internal pure returns (TaskIdSelector memory sel) {
        sel.quantifier = Quantifier.None;
    }
}
