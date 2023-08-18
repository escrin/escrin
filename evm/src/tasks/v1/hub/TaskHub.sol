// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC165, IERC165} from "@openzeppelin/contracts/utils/introspection/ERC165.sol";

import {ITaskHubV1} from "./ITaskHub.sol";

/// @dev The methods in this contract are not marked with `override` are not guaranteed to be in the next version of the contract.
contract TaskHubV1 is ITaskHubV1, ERC165 {
    function notify() external override {
        emit TasksAvailable(msg.sender, "");
    }

    function notify(bytes32 context) external override {
        emit TasksAvailable(msg.sender, context);
    }

    function supportsInterface(bytes4 interfaceId)
        public
        view
        virtual
        override(ERC165, IERC165)
        returns (bool)
    {
        return interfaceId == type(ITaskHubV1).interfaceId || super.supportsInterface(interfaceId);
    }
}
