// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {TaskAcceptorV1Proxy} from "../widgets/TaskAcceptorProxy.sol";
import {BaseTaskAcceptorV1} from "./BaseTaskAcceptor.sol";
import {TaskIdSelector} from "./TaskIdSelector.sol";

abstract contract DelegatedTaskAcceptorV1 is BaseTaskAcceptorV1, TaskAcceptorV1Proxy {
    constructor(address _upstream) TaskAcceptorV1Proxy(_upstream) {
        return;
    }

    function _acceptTaskResults(
        uint256[] calldata _taskIds,
        bytes calldata _proof,
        bytes calldata _report,
        address _submitter
    ) internal virtual override returns (TaskIdSelector memory) {
        return taskAcceptor().acceptTaskResults(_taskIds, _proof, _report, _submitter);
    }
}
