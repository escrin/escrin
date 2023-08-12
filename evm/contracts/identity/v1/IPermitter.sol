// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {IdentityId} from "./Types.sol";

interface IPermitter is IERC165 {
    function grantPermit(
        IdentityId identity,
        address relayer,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) external returns (bool allow, uint64 expiry);

    function revokePermit(
        IdentityId identity,
        address relayer,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) external returns (bool allow);
}
