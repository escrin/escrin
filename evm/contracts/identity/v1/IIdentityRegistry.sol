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

    /// Creates a new identity controlled by the specified permitter contract.
    /// @param permitter The address of the contract that grants the identity.
    /// @param pers [optional] Extra entropy used to generate the identity.
    /// @return id The newly created identity's id (store this somewhere).
    function createIdentity(address permitter, bytes calldata pers)
        external
        returns (IdentityId id);

    /// Irrevocably destroys the identity. Must be called by the registrant.
    function destroyIdentity(IdentityId id) external;

    /// Sets the identity's new permitter. Must be called by the registrant.
    function setPermitter(IdentityId id, address permitter) external;

    /// Initiates a transfer to a new registrant. Must be called by the registrant.
    function proposeRegistrationTransfer(IdentityId id, address to) external;

    /// Accepts a pending registration transfer. Must be called by the new registrant.
    function acceptRegistrationTransfer(IdentityId id) external;

    /// Grants an identity's permit to an account. Must be called by the permitter.
    /// @param id The id of the identity to grant.
    /// @param to The address of the permit's recipient.
    /// @param expiry The Unix timestamp at which the permit expires.
    function grantIdentity(IdentityId id, address to, uint64 expiry) external;

    /// Called by the identity's permitter to revoke the identity to the recipient.
    function revokeIdentity(IdentityId id, address from) external;

    /// Returns the permitter associated with the identity.
    function getPermitter(IdentityId id) external view returns (IPermitter);

    /// Returns the permit to the identity held by the provided account, if any.
    function readPermit(address holder, IdentityId identity)
        external
        view
        returns (Permit memory);

    /// Returns the identity's current and proposed registrant(s).
    function getRegistrant(IdentityId id)
        external
        view
        returns (address current, address proposed);
}
