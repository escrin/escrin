// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC165, IERC165} from "@openzeppelin/contracts/utils/introspection/ERC165.sol";

import {ITaskHub} from "./ITaskHub.sol";

/// @dev The methods in this contract are not marked with `override` are not guaranteed to be in the next version of the contract.
contract TaskHub is ITaskHub, ERC165 {
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
        return interfaceId == type(ITaskHub).interfaceId || super.supportsInterface(interfaceId);
    }
}
