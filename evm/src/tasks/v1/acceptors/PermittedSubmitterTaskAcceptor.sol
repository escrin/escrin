// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

import {InterfaceUnsupported} from "../../../Types.sol";
import {IdentityId, IIdentityRegistry} from "../../../identity/v1/IIdentityRegistry.sol";
import {TaskAcceptor} from "./TaskAcceptor.sol";

abstract contract PermittedSubmitterTaskAcceptor is TaskAcceptor {
    IIdentityRegistry private identityRegistry_;
    IdentityId private trustedIdentity_;

    constructor(address identityRegistry, IdentityId trustedIdentity) {
        identityRegistry_ = _requireIsIdentityRegistry(identityRegistry);
        trustedIdentity_ = trustedIdentity;
    }

    function getTrustedIdentity() external view returns (IIdentityRegistry, IdentityId) {
        return (identityRegistry_, trustedIdentity_);
    }

    function _acceptTaskResults(uint256[] calldata, bytes calldata, bytes calldata)
        internal
        virtual
        override
        returns (TaskIdSelector memory sel)
    {
        sel.quantifier = _isPermittedSubmitter(msg.sender) ? Quantifier.All : Quantifier.None;
    }

    function _setTrustedIdentity(address registry, IdentityId identity) internal {
        identityRegistry_ = _requireIsIdentityRegistry(registry);
        trustedIdentity_ = identity;
    }

    function _isPermittedSubmitter(address submitter) internal view returns (bool) {
        IIdentityRegistry.Permit memory permit =
            identityRegistry_.readPermit(submitter, trustedIdentity_);
        return permit.expiry > block.timestamp;
    }

    function _requireIsIdentityRegistry(address registry)
        private
        view
        returns (IIdentityRegistry)
    {
        if (!ERC165Checker.supportsInterface(registry, type(IIdentityRegistry).interfaceId)) {
            revert InterfaceUnsupported();
        }
        return IIdentityRegistry(registry);
    }
}
