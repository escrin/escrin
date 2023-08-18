// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {TaskAcceptor} from "./TaskAcceptor.sol";

abstract contract TrustedSubmitterTaskAcceptor is TaskAcceptor {
    address private trustedSubmitter_;

    constructor(address trustedSubmitter) {
        trustedSubmitter_ = trustedSubmitter;
    }

    function getTrustedSubmitter() external view returns (address) {
        return trustedSubmitter_;
    }

    function _acceptTaskResults(uint256[] calldata, bytes calldata, bytes calldata)
        internal
        virtual
        override
        returns (TaskIdSelector memory sel)
    {
        sel.quantifier = _isTrustedSubmitter(msg.sender) ? Quantifier.All : Quantifier.None;
    }

    function _setTrustedSubmitter(address submitter) internal {
        trustedSubmitter_ = submitter;
    }

    function _isTrustedSubmitter(address addr) internal view returns (bool) {
        return addr == trustedSubmitter_;
    }
}
