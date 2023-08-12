// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {TaskAcceptorV1} from "./TaskAcceptor.sol";

abstract contract TrustedSubmitterTaskAcceptorV1 is TaskAcceptorV1 {
    address private immutable trustedSubmitter_;

    constructor(address trustedSubmitter) {
        trustedSubmitter_ = trustedSubmitter;
    }

    function getTrustedSubmitter() external view returns (address) {
        return trustedSubmitter_;
    }

    function _acceptTaskResults(
        uint256[] calldata,
        bytes calldata proof,
        bytes calldata report,
        address submitter
    ) internal virtual override returns (TaskIdSelector memory sel) {
        sel.quantifier = _isTrustedSubmitter(submitter) ? Quantifier.All : Quantifier.None;
    }

    function _isTrustedSubmitter(address addr) internal view virtual returns (bool) {
        return addr == trustedSubmitter_;
    }
}
