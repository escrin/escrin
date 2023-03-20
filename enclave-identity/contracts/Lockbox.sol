// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

// import "hardhat/console.sol";

import "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";

import {AttestationToken, AttestationTokenId} from "./AttestationToken.sol";

contract Lockbox {
    error NoAttestationToken();

    AttestationToken public immutable attestationToken;

    mapping(AttestationTokenId => bytes32) private lockbox;

    modifier onlyAttested(AttestationTokenId _attid) {
        if (!attestationToken.isAttested(msg.sender, _attid)) revert NoAttestationToken();
        _;
    }

    constructor(AttestationToken _attestationToken) {
        attestationToken = _attestationToken;
    }

    function createKey(AttestationTokenId attid, bytes calldata pers) external {
        if(lockbox[attid] != 0) return;
        lockbox[attid] = block.chainid == 0x5aff || block.chainid == 0x5afe
            ? bytes32(Sapphire.randomBytes(32, pers))
            : blockhash(block.number);
    }

    function getKey(AttestationTokenId attid) external view onlyAttested(attid) returns (bytes32) {
        return lockbox[attid];
    }
}
