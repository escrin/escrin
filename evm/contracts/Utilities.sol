// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {Sapphire} from "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";

function randomBytes(uint256 count, bytes calldata pers) view returns (bytes memory) {
    if (block.chainid == 0x5aff || block.chainid == 0x5afe) {
        return Sapphire.randomBytes(count, pers);
    }
    uint256 words = (count + 31) >> 5;
    bytes memory out = new bytes(words << 5);
    bytes32 seed = keccak256(
        abi.encodePacked(
            msg.sender,
            blockhash(block.number),
            block.timestamp,
            block.prevrandao,
            block.coinbase,
            count,
            pers
        )
    );
    for (uint256 i = 0; i < words; i++) {
        seed = keccak256(abi.encodePacked(seed, i, blockhash(block.number - i)));
        assembly {
            mstore(add(out, add(32, mul(32, i))), seed)
        }
    }
    assembly {
        mstore(out, count)
    }
    return out;
}
