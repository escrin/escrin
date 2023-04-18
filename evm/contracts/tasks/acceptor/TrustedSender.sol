// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {TaskIdSelector, TaskIdSelectorOps} from "../TaskIdSelector.sol";
import {BaseTaskAcceptorV1} from "./Base.sol";

contract TrustedSenderAcceptor is BaseTaskAcceptorV1 {
    using TaskIdSelectorOps for TaskIdSelector;

    address public immutable taskHub;
    address public immutable trustedSender;

    constructor(address _taskHub, address _trustedSender) {
        taskHub = _taskHub;
        trustedSender = _trustedSender;
    }

    function _acceptTaskResults(
        uint256[] calldata,
        bytes calldata,
        bytes calldata,
        address _submitter
    ) internal virtual override returns (TaskIdSelector memory) {
        if (msg.sender != taskHub || _submitter != trustedSender) return TaskIdSelectorOps.none();
        return TaskIdSelectorOps.all();
    }
}
