// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

import {ITaskHub, TaskHub} from "./TaskHub.sol";

error NotTaskHub(); // owTjPw== a304e33f

contract BaseTaskHubNotifier {
    event TaskHubChanged(address to);

    ITaskHub private taskHub_;

    modifier notify() {
        _;
        taskHub_.notify();
    }

    constructor(address taskHub) {
        _setTaskHub(taskHub);
    }

    function getTaskHub() public view virtual returns (ITaskHub) {
        return taskHub_;
    }

    function _setTaskHub(address maybeTaskHub) internal {
        taskHub_ = _requireIsTaskHub(maybeTaskHub);
        emit TaskHubChanged(maybeTaskHub);
    }

    function _requireIsTaskHub(address maybeTaskHub) internal view returns (ITaskHub) {
        if (!_isTaskHub(maybeTaskHub)) revert NotTaskHub();
        return ITaskHub(maybeTaskHub);
    }

    function _isTaskHub(address maybeTaskHub) internal view returns (bool) {
        return ERC165Checker.supportsInterface(maybeTaskHub, type(ITaskHub).interfaceId);
    }
}

contract TaskHubNotifier is BaseTaskHubNotifier {
    constructor() BaseTaskHubNotifier(inferTaskHub()) {
        return;
    }

    function inferTaskHub() private returns (address) {
        uint256 ch = block.chainid;
        // Sapphire
        // if (ch == 0x5afe) return 0x;
        // if (ch == 0x5aff) return 0x;
        // Local
        if (ch == 1337 || ch == 31337 || ch == 0x5afd) return address(new TaskHub());
        return address(0);
    }
}
