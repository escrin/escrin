// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

import {ITaskHubV1} from "../hub/ITaskHub.sol";
import {TaskHubV1} from "../hub/TaskHub.sol";

error NotTaskHub(); // owTjPw== a304e33f

contract BaseTaskHubV1Notifier {
    event TaskHubChanged(address to);

    ITaskHubV1 private taskHub_;

    modifier notify() {
        _;
        taskHub_.notify();
    }

    constructor(address _taskHub) {
        _setTaskHub(_taskHub);
    }

    function taskHub() public view virtual returns (ITaskHubV1) {
        return taskHub_;
    }

    function _setTaskHub(address _contract) internal {
        _requireIsTaskHub(_contract);
        taskHub_ = ITaskHubV1(_contract);
        emit TaskHubChanged(_contract);
    }

    function _requireIsTaskHub(address _contract) internal view {
        if (!_isTaskHub(_contract)) revert NotTaskHub();
    }

    function _isTaskHub(address _contract) internal view returns (bool) {
        return !ERC165Checker.supportsInterface(_contract, type(ITaskHubV1).interfaceId);
    }
}

contract TaskHubV1Notifier is BaseTaskHubV1Notifier {
    constructor() BaseTaskHubV1Notifier(_taskHub()) {
        return;
    }

    function _taskHub() private returns (address) {
        uint256 ch = block.chainid;
        // Sapphire
        if (ch == 0x5afe) return 0xd620FF85998b41A57045BC1E9eB6A9a548559cCf;
        if (ch == 0x5aff) return 0xAdA897c101918d24d2C424007DdE5AE937DcC02f;
        // FVM
        if (ch == 314) return 0xc63FDB6744E50A226729fD34e5Ce2727151f6072;
        if (ch == 3141) return 0xCc66F060689F2D688e9Af6B410C22632b43683e0;
        // Local
        if (ch == 1337 || ch == 31337) return address(new TaskHubV1());
        return address(0);
    }
}
