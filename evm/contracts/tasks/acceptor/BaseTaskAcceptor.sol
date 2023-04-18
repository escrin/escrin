// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {ITaskAcceptorV1} from "./ITaskAcceptor.sol";
import {TaskIdSelector} from "./TaskIdSelector.sol";

/// The input task ids were not sorted.
error SubmisionTaskIdsNotSorted();
/// The set of accepted task ids was not sorted.
error AcceptedTaskIdsNotSorted();

abstract contract BaseTaskAcceptorV1 is ITaskAcceptorV1 {
    function acceptTaskResults(
        uint256[] calldata _taskIds,
        bytes calldata _proof,
        bytes calldata _report,
        address _submitter
    ) external virtual returns (TaskIdSelector memory sel) {
        _beforeTaskResultsAccepted(_taskIds, _proof, _report, _submitter);
        if (!_isSorted(_taskIds)) revert SubmisionTaskIdsNotSorted();
        sel = _acceptTaskResults(_taskIds, _proof, _report, _submitter);
        if (!_isSorted(sel.taskIds)) revert AcceptedTaskIdsNotSorted();
        _afterTaskResultsAccepted(_taskIds, _report, _submitter, sel);
    }

    function supportsInterface(bytes4 _interfaceId) public view virtual override returns (bool) {
        return _interfaceId == type(ITaskAcceptorV1).interfaceId;
    }

    function _acceptTaskResults(
        uint256[] calldata _taskIds,
        bytes calldata _proof,
        bytes calldata _report,
        address _submitter
    ) internal virtual returns (TaskIdSelector memory);

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

    function _isSorted(uint256[] memory _input) internal pure returns (bool) {
        for (uint256 i = 1; i < _input.length - 1; ++i)
            if (_input[i] <= _input[i - 1]) return false;
        return true;
    }
}
