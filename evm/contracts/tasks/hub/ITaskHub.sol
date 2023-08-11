// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

interface ITaskHubV1 is IERC165 {
    event TasksAvailable(address indexed generator, bytes32 indexed context);

    /// Alerts any listening task runners that there are new tasks available.
    function notify() external;

    /// Alerts any listening task runners that there are new tasks available.
    /// @param context Some indexed data to be emitted with the event for listeners to filter on.
    function notify(bytes32 context) external;
}
