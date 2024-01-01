// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {IPermitter} from "./IPermitter.sol";

type IdentityId is uint256;

interface IIdentityRegistry is IERC165 {
    /// The action is disallowed.
    error Unauthorized(); // 82b42900 grQpAA==

    /// The provided contract address does not support the correct interface.
    error InterfaceUnsupported(); // bbaa55aa u6pVqg==

    struct Permit {
        uint64 expiry;
    }

    event RegistrationTransferProposed(IdentityId indexed identityId, address indexed proposed);
    event PermitterChanged(IdentityId indexed identityId);

    event IdentityCreated(IdentityId id);
    event IdentityDestroyed(IdentityId indexed id);
    event IdentityGranted(IdentityId indexed id, address indexed to);
    event IdentityRevoked(IdentityId indexed id, address indexed from);

    function createIdentity(address permitter, bytes calldata pers)
        external
        returns (IdentityId id);

    function destroyIdentity(IdentityId id) external;

    function setPermitter(IdentityId id, address permitter) external;

    function proposeRegistrationTransfer(IdentityId id, address to) external;

    function acceptRegistrationTransfer(IdentityId id) external;

    /// Called by the identity's permitter to grant the identity to the recipient.
    function grantIdentity(IdentityId id, address to, uint64 expiry) external;

    /// Called by the identity's permitter to revoke the identity to the recipient.
    function revokeIdentity(IdentityId id, address from) external;

    function getPermitter(IdentityId id) external view returns (IPermitter);

    function readPermit(address holder, IdentityId identity)
        external
        view
        returns (Permit memory);

    function getRegistrant(IdentityId id)
        external
        view
        returns (address current, address proposed);
}