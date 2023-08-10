// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {TaskAcceptorV1} from "./TaskAcceptor.sol";

abstract contract TrustedSenderTaskAcceptorV1 is TaskAcceptorV1 {
    address internal immutable trustedSender_;

    constructor(address trustedSender) {
        trustedSender_ = trustedSender;
    }

    function getTrustedSender() external view returns (address) {
        return trustedSender_;
    }

    function _acceptTaskResults(
        uint256[] calldata,
        Proof calldata,
        Report calldata,
        address submitter
    ) internal virtual override returns (TaskIdSelector memory sel) {
        sel.quantifier = _isTrustedSender(submitter) ? Quantifier.All : Quantifier.None;
    }

    function _isTrustedSender(address addr) internal view virtual returns (bool) {
        return addr == trustedSender_;
    }
}
