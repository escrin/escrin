// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {IdentityId} from "./IIdentityRegistry.sol";

interface IPermitter is IERC165 {
    /// Requests that the permitter trigger the upstream identity registry to grant an identity.
    /// @param identity The identity that the requester wishes to acquire.
    /// @param requester The account to which the identity permit will be issued.
    /// @param duration The requested lifetime of the permit, which may be different from lifetime actually granted.
    /// @param context Non-authentication data provided to the permitter to make its decision.
    /// @param authorization Authentication data provided to the permitter to make its decision.
    /// @return expiry The timestamp at which the permit expires, which may be different from the request timestamp plus the requested duration.
    function acquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) external returns (uint64 expiry);

    /// Requests that the permitter trigger the upstream identity registry to revoke an identity.
    /// @param identity The identity that the requester wishes to acquire.
    /// @param possessor The account that will no longer have the permit.
    /// @param context Non-authentication data provided to the permitter to make its decision.
    /// @param authorization Authentication data provided to the permitter to make its decision.
    function releaseIdentity(
        IdentityId identity,
        address possessor,
        bytes calldata context,
        bytes calldata authorization
    ) external;

    /// Returns the the upstream permitter or identity registry called upon successful identity acquisition/release.
    function upstream() external view returns (address);
}
