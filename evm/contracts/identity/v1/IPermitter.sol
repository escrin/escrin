// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {IdentityId} from "./Types.sol";

interface IPermitter is IERC165 {
    function grantPermit(
        IdentityId identity,
        address requester,
        address beneficiary,
        bytes calldata context,
        bytes calldata authz
    ) external returns (bool allow, uint64 expiry);

    function revokePermit(
        IdentityId identity,
        address requester,
        address beneficiary,
        bytes calldata context,
        bytes calldata authz
    ) external returns (bool allow);
}
