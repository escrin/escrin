// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {Sapphire} from "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";

import {AttestationToken, TcbId} from "./AttestationToken.sol";

contract Lockbox {
    error NoAttestationToken(); // zUKkBQ== cd42a405

    AttestationToken internal immutable attestationToken_;

    mapping(TcbId => bytes32) private lockbox;

    modifier onlyAttested(TcbId tcbId) {
        if (!attestationToken_.isAttested(msg.sender, tcbId)) revert NoAttestationToken();
        _;
    }

    constructor(AttestationToken attestationToken) {
        attestationToken_ = attestationToken;
    }

    function getAttestationToken() external view returns (AttestationToken) {
        return attestationToken_;
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
