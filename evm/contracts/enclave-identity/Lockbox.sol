// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

// import "hardhat/console.sol";

import {Sapphire} from "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";

import {AttestationToken, TcbId} from "./AttestationToken.sol";

contract Lockbox {
    error NoAttestationToken();

    AttestationToken public immutable attestationToken;

    mapping(TcbId => bytes32) private lockbox;

    modifier onlyAttested(TcbId _tcbId) {
        if (!attestationToken.isAttested(msg.sender, _tcbId)) revert NoAttestationToken();
        _;
    }

    constructor(AttestationToken _attestationToken) {
        attestationToken = _attestationToken;
    }

    function createKey(TcbId tcbId, bytes calldata pers) external {
        if (lockbox[tcbId] != 0) return;
        lockbox[tcbId] = block.chainid == 0x5aff || block.chainid == 0x5afe
            ? bytes32(Sapphire.randomBytes(32, pers))
            : blockhash(block.number);
    }

    function getKey(TcbId tcbId) external view onlyAttested(tcbId) returns (bytes32) {
        return lockbox[tcbId];
    }
}
