// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

import {InterfaceUnsupported} from "../../Types.sol";
import {IdentityId, IIdentityRegistry} from "../../identity/v1/IIdentityRegistry.sol";
import {TaskAcceptorV1} from "./TaskAcceptor.sol";

abstract contract PermittedSubmitterTaskAcceptorV1 is TaskAcceptorV1 {
    IIdentityRegistry private immutable identityRegistry_;
    IdentityId private immutable trustedIdentity_;

    constructor(address identityRegistry, IdentityId trustedIdentity) {
        identityRegistry_ = _requireIsIdentityRegistry(identityRegistry);
        trustedIdentity_ = trustedIdentity;
    }

    function getTrustedIdentity() external view returns (IIdentityRegistry, IdentityId) {
        return (identityRegistry_, trustedIdentity_);
    }

    function _acceptTaskResults(
        uint256[] calldata,
        bytes calldata,
        bytes calldata,
        address submitter
    ) internal virtual override returns (TaskIdSelector memory sel) {
        sel.quantifier = _isPermittedSubmitter(submitter) ? Quantifier.All : Quantifier.None;
    }

    function _isPermittedSubmitter(address submitter) internal view virtual returns (bool) {
        return identityRegistry_.hasIdentity(submitter, trustedIdentity_);
    }

    function _requireIsIdentityRegistry(address registry)
        internal
        view
        returns (IIdentityRegistry)
    {
        if (!ERC165Checker.supportsInterface(registry, type(IIdentityRegistry).interfaceId)) {
            revert InterfaceUnsupported();
        }
        return IIdentityRegistry(registry);
    }
}
