// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {ERC165} from "@openzeppelin/contracts/utils/introspection/ERC165.sol";

import {ITaskHubV1, ITaskGeneratorV1, TaskSubmissionV1} from "./ITaskHub.sol";
import {ITaskAcceptorV1} from "./acceptor/ITaskAcceptor.sol";
import {TaskIdSelector, TaskIdSelectorOps} from "./TaskIdSelector.sol";

/// The input task ids were not sorted.
error SubmisionTaskIdsNotSorted();
/// The set of accepted task ids was not sorted.
error AcceptedTaskIdsNotSorted();

/// @dev The methods in this contract are not marked with `override` are not guaranteed to be in the next version of the contract.
contract TaskHubV1 is ITaskHubV1 {
    using TaskIdSelectorOps for TaskIdSelector;

    function notify() external override {
        emit TasksAvailable(msg.sender, "");
    }

    function submitTaskResults(
        ITaskGeneratorV1 _requester,
        uint256[] calldata _taskIds,
        bytes calldata _proof,
        bytes calldata _report
    ) external override returns (TaskIdSelector memory) {
        if (!_isSorted(_taskIds)) revert SubmisionTaskIdsNotSorted();
        ITaskAcceptorV1 acceptor = _requester.taskAcceptor();
        // This doesn't implement commit-reveal, so there's a bit of trust involved.
        // A task runner might not run jobs for a contract that receives results out of band.
        return acceptor.acceptTaskResults(_taskIds, _proof, _report, msg.sender);
    }

    function supportsInterface(bytes4 _interfaceId) public pure override returns (bool) {
        return _interfaceId == type(ITaskHubV1).interfaceId;
    }

    function _isSorted(uint256[] memory _input) internal pure returns (bool) {
        for (uint256 i = 1; i < _input.length - 1; ++i)
            if (_input[i] <= _input[i - 1]) return false;
        return true;
    }
}
