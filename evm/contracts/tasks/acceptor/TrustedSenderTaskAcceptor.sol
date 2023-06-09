// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {TaskAcceptorV1} from "./TaskAcceptor.sol";

abstract contract TrustedSenderTaskAcceptorV1 is TaskAcceptorV1 {
    address public immutable trustedSender;

    constructor(address _trustedSender) {
        trustedSender = _trustedSender;
    }

    function _acceptTaskResults(
        uint256[] calldata,
        bytes calldata,
        bytes calldata,
        address _submitter
    ) internal virtual override returns (TaskIdSelector memory sel) {
        sel.quantifier = _isTrustedSender(_submitter) ? Quantifier.All : Quantifier.None;
    }

    function _isTrustedSender(address addr) internal virtual view returns (bool) {
        return addr == trustedSender;
    }
}
