// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ITaskAcceptorV1Proxy} from "../widgets/TaskAcceptorProxy.sol";
import {TaskAcceptorV1} from "./TaskAcceptor.sol";

abstract contract DelegatedTaskAcceptorV1 is TaskAcceptorV1, ITaskAcceptorV1Proxy {
    function _acceptTaskResults(uint256[] calldata, bytes calldata, bytes calldata, address)
        internal
        virtual
        override
        returns (TaskIdSelector memory)
    {
        // solhint-disable-next-line avoid-low-level-calls
        (bool success, bytes memory result) = address(this.getTaskAcceptor()).delegatecall(msg.data);
        if (!success) revert(string(result));
        return abi.decode(result, (TaskIdSelector));
    }
}
