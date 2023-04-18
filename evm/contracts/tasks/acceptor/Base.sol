// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {TaskIdSelector} from "../TaskIdSelector.sol";
import {ITaskAcceptorV1} from "./ITaskAcceptor.sol";

abstract contract BaseTaskAcceptorV1 is ITaskAcceptorV1 {
    function acceptTaskResults(
        uint256[] calldata _taskIds,
        bytes calldata _proof,
        bytes calldata _report,
        address _submitter
    ) external virtual returns (TaskIdSelector memory sel) {
        _beforeTaskResultsAccepted(_taskIds, _proof, _report, _submitter);
        sel = _acceptTaskResults(_taskIds, _proof, _report, _submitter);
        _afterTaskResultsAccepted(_taskIds, _report, _submitter, sel);
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

    function supportsInterface(bytes4 _interfaceId) public view virtual override returns (bool) {
        return _interfaceId == type(ITaskAcceptorV1).interfaceId;
    }
}
