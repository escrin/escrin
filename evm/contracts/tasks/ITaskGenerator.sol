// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {ITaskAcceptorV1} from "./acceptor/ITaskAcceptor.sol";

interface ITaskGeneratorV1 is IERC165 {
    function taskAcceptor() external view returns (ITaskAcceptorV1);
}
