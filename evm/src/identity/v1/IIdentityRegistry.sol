// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {IPermitter} from "./IPermitter.sol";
import {IdentityId} from "./Types.sol";

interface IIdentityRegistry is IERC165 {
    event RegistrationTransferProposed(IdentityId indexed identityId, address indexed proposed);
    event PermitterChanged(IdentityId indexed identityId);

    event IdentityCreated(IdentityId id);
    event IdentityDestroyed(IdentityId indexed id);
    event IdentityAcquired(IdentityId indexed id, address indexed acquirer);
    event IdentityReleased(IdentityId indexed id, address indexed acquirer);

    function createIdentity(address permitter, bytes calldata pers)
        external
        returns (IdentityId id);

    function destroyIdentity(IdentityId id) external;

    function setPermitter(IdentityId id, address permitter) external;

    function proposeRegistrationTransfer(IdentityId id, address to) external;

    function acceptRegistrationTransfer(IdentityId id) external;

    function acquireIdentity(
        IdentityId id,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) external;

    function releaseIdentity(
        IdentityId id,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) external;

    function hasIdentity(address account, IdentityId id) external view returns (bool);

    function getRegistrant(IdentityId id)
        external
        view
        returns (address current, address proposed);
}
