// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {TaskIdSelector} from "../TaskIdSelector.sol";

/// The contract is did not pass the ITaskAcceptor ERC-165 check.
error NotTaskAcceptor();

interface ITaskAcceptorV1 is IERC165 {
    /// Accepts zero or more tasks results.
    function acceptTaskResults(
        uint256[] calldata _taskIds,
        bytes calldata _proof,
        bytes calldata _report,
        address _submitter
    ) external returns (TaskIdSelector memory);
}
