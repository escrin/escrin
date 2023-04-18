// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {TaskIdSelector} from "../TaskIdSelector.sol";
import {BaseTaskAcceptorV1} from "./Base.sol";
import {TaskAcceptorHaver} from "./Haver.sol";

contract DelegatedTaskAcceptor is BaseTaskAcceptorV1, TaskAcceptorHaver {
    constructor(address _upstream) TaskAcceptorHaver(_upstream) {
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
