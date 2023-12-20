// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {Test} from "forge-std/Test.sol";

import {NE} from "../../../../src/identity/v1/permitters/NitroEnclavePermitter.sol";

contract VerifierContract {
    function verifyAttestationDocument(bytes calldata doc) external view {
        NE.PcrSelector memory pcrs =
            NE.PcrSelector({mask: 0, hash: keccak256(abi.encodePacked(bytes32(0)))});
        NE.verifyAttestationDocument(doc, pcrs, 0);
    }
}

contract NitroEnclaveAttestationVerifierTest is Test {
    VerifierContract v;

    function setUp() public {
        v = new VerifierContract();
    }

    function testVerifyAttestation() public {
        vm.warp(1703101376);
        v.verifyAttestationDocument(
            bytes(vm.readFileBinary("./test/identity/v1/permitters/att_doc_sample.bin"))
        );
    }

    function testFuzz_verifyAttestation(bytes calldata attestationDocument) public {
        vm.expectRevert();
        v.verifyAttestationDocument(attestationDocument);
    }
}
