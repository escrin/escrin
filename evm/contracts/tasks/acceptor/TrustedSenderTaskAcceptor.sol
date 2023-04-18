// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {TaskAcceptorV1, TaskIdSelectorOps} from "./TaskAcceptor.sol";

abstract contract TrustedSenderAcceptor is TaskAcceptorV1 {
    using TaskIdSelectorOps for TaskIdSelector;

    address public immutable trustedSender;

    constructor(address _trustedSender) {
        trustedSender = _trustedSender;
    }

    function _acceptTaskResults(
        uint256[] calldata,
        bytes calldata,
        bytes calldata,
        address _submitter
    ) internal virtual override returns (TaskIdSelector memory) {
        if (_submitter != trustedSender) return TaskIdSelectorOps.none();
        return TaskIdSelectorOps.all();
    }
}
