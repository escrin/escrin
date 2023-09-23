// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {Test} from "forge-std/Test.sol";

import {NitroEnclaveAttestationVerifier} from
    "../../../../src/identity/v1/permitters/NitroEnclavePermitter.sol";

contract NitroEnclaveAttestationVerifierTest is Test {
    NitroEnclaveAttestationVerifier v;

    function setUp() public {
        v = new NitroEnclaveAttestationVerifier();
    }

    function testVerifyAttestation() public view {
        v.verifyAttestationDocument(
            bytes(vm.readFileBinary("./test/identity/v1/permitters/att_doc_sample.bin"))
        );
    }
}
