// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC165, IERC165} from "@openzeppelin/contracts/utils/introspection/ERC165.sol";
import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

import {IdentityId, IIdentityRegistry} from "../IIdentityRegistry.sol";
import {IPermitter} from "../IPermitter.sol";

abstract contract Permitter is IPermitter, ERC165 {
    /// The provided contract address does not support the correct interface.
    error InterfaceUnsupported(); // bbaa55aa u6pVqg==

    /// The action is disallowed.
    error Unauthorized(); // 82b42900 grQpAA==
    /// The requested duration of the permit was too long.
    error DurationTooLong(); // lSn1Bg==

    enum UpstreamKind {
        Unknown,
        Registry,
        Permitter
    }

    bytes32 private immutable upstreamAndKind;

    constructor(address upstreamRegistryOrPermitter) {
        if (!ERC165Checker.supportsERC165(upstreamRegistryOrPermitter)) {
            revert InterfaceUnsupported();
        }
        UpstreamKind upstreamKind;
        if (
            ERC165Checker.supportsERC165InterfaceUnchecked(
                upstreamRegistryOrPermitter, type(IIdentityRegistry).interfaceId
            )
        ) {
            upstreamKind = UpstreamKind.Registry;
        } else if (
            ERC165Checker.supportsERC165InterfaceUnchecked(
                upstreamRegistryOrPermitter, type(IPermitter).interfaceId
            )
        ) {
            upstreamKind = UpstreamKind.Permitter;
        } else {
            revert InterfaceUnsupported();
        }
        upstreamAndKind =
            bytes32(uint256(bytes32(bytes20(upstreamRegistryOrPermitter))) | uint8(upstreamKind));
    }

    function acquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) external virtual override returns (uint64 expiry) {
        _beforeAcquireIdentity({
            identity: identity,
            requester: requester,
            duration: duration,
            context: context,
            authorization: authorization
        });
        expiry = _acquireIdentity({
            identity: identity,
            requester: requester,
            duration: duration,
            context: context,
            authorization: authorization
        });
        (UpstreamKind upstreamKind, address up) = _upstream();
        if (upstreamKind == UpstreamKind.Registry) {
            IIdentityRegistry(up).grantIdentity(identity, requester, expiry);
        } else if (upstreamKind == UpstreamKind.Permitter) {
            expiry = IPermitter(up).acquireIdentity(
                identity, requester, duration, context, authorization
            );
        } else {
            revert InterfaceUnsupported();
        }
        _afterAcquireIdentity(identity, requester, context);
    }

    function releaseIdentity(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) external virtual override {
        _beforeReleaseIdentity({
            identity: identity,
            requester: requester,
            context: context,
            authorization: authorization
        });
        _releaseIdentity({
            identity: identity,
            requester: requester,
            context: context,
            authorization: authorization
        });
        (UpstreamKind upstreamKind, address up) = _upstream();
        if (upstreamKind == UpstreamKind.Registry) {
            IIdentityRegistry(up).revokeIdentity(identity, requester);
        } else if (upstreamKind == UpstreamKind.Permitter) {
            IPermitter(up).releaseIdentity(identity, requester, context, authorization);
        } else {
            revert InterfaceUnsupported();
        }
        _afterReleaseIdentity(identity, requester, context);
    }

    function upstream() external view virtual override returns (address) {
        (, address addr) = _upstream();
        return addr;
    }

    function supportsInterface(bytes4 interfaceId)
        public
        view
        virtual
        override(ERC165, IERC165)
        returns (bool)
    {
        return interfaceId == type(IPermitter).interfaceId || super.supportsInterface(interfaceId);
    }

    function _upstream() internal view returns (UpstreamKind, address) {
        bytes32 up = upstreamAndKind;
        address addr = address(bytes20(up));
        UpstreamKind kind = UpstreamKind(uint256(up) & 0xff);
        return (kind, addr);
    }

    function _getIdentityRegistry() internal view returns (IIdentityRegistry) {
        (UpstreamKind kind, address up) = _upstream();
        bool isRegistry = kind == UpstreamKind.Registry;
        while (!isRegistry) {
            up = IPermitter(up).upstream();
            isRegistry = ERC165Checker.supportsInterface(up, type(IIdentityRegistry).interfaceId);
        }
        return IIdentityRegistry(up);
    }

    /// Authorizes the identity acquisition request, returning the expiry if approved or reverting if denied.
    function _acquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual returns (uint64 expiry);

    /// Authorizes the identity release request, reverting if denied.
    function _releaseIdentity(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual;

    function _beforeAcquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual {}

    function _afterAcquireIdentity(IdentityId identity, address requester, bytes calldata context)
        internal
        virtual
    {}

    function _beforeReleaseIdentity(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual {}

    function _afterReleaseIdentity(IdentityId identity, address requester, bytes calldata context)
        internal
        virtual
    {}
}
