// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {IPermitter} from "./IPermitter.sol";
import {IdentityId} from "./Types.sol";

interface IIdentityRegistry is IERC165 {
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

    function readPermit(address holder, IdentityId identity)
        external
        view
        returns (Permit memory);

    function getRegistrant(IdentityId id)
        external
        view
        returns (address current, address proposed);
}

library Permits {
    function isActive(IIdentityRegistry.Permit memory permit) internal view returns (bool) {
        return permit.expiry > block.timestamp;
    }
}
