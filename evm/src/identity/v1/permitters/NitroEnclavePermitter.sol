// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import "forge-std/console2.sol";

import {Sapphire} from "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";

contract NitroEnclaveAttestationVerifier {
    // CN: aws.nitro-enclaves
    bytes constant ROOT_CA_KEY =
        hex"04fc0254eba608c1f36870e29ada90be46383292736e894bfff672d989444b5051e534a4b1f6dbe3c0bc581a32b7b176070ede12d69a3fea211b66e752cf7dd1dd095f6f1370f4170843d9dc100121e4cf63012809664487c9796284304dc53ff4";
    uint256 constant ROOT_CA_EXPIRY = 2519044085;

    function verifyAttestationDocument(bytes calldata doc) external view {
        require(block.timestamp >= ROOT_CA_EXPIRY, "root ca expired");

        // Array(4) - 84
        // Bytes(4) - 44 // 4 bytes of cbor-encoded protected header
        // Map(1) - a1 // protected header
        // Key: Unsigned(1) - 01
        // Value: Signed(-35) - 3822 // P-384
        // Map(0) - a0 // unprotected header
        // Bytes(long) - 59 // payload
        require(bytes8(doc[0:8]) == bytes8(0x84_44_a1_01_3822_a0_59), "invalid doc");
        uint256 payloadSize = uint16(bytes2(doc[8:10]));
        uint256 payloadStart = 10;
        uint256 payloadEnd = payloadStart + payloadSize;
        bytes calldata payload = doc[payloadStart:payloadEnd];

        // Bytes(short) - 58
        // Unsigned(96) - 60
        require(bytes2(doc[payloadEnd:payloadEnd + 2]) == bytes2(0x58_60));
        uint256 sigStart = payloadEnd + 2;
        uint256 sigEnd = sigStart + 0x60;

        bytes calldata pk = verifyPayload(payload);
        verifySignature({payload: payload, pk: pk, sig: doc[sigStart:sigEnd]});
    }

    function verifySignature(bytes calldata payload, bytes calldata pk, bytes calldata sig)
        internal
        view
    {
        bytes memory coseSign1 = bytes.concat(
            // COSE Sign1 structure:
            // Array(4) - 84
            // Text(10) "Signature1" - 6a_5369676E617475726531
            // the protected header from before - 44_A1013822
            // Bytes(0) - 40
            // Bytes(long) - 59
            bytes19(0x84_6a_5369676E617475726531_44_A1013822_40_59),
            bytes2(uint16(payload.length)),
            payload
        );
        // require(Sapphire.verifyP384Prehashed(pk, Sapphire.sha384(coseSign1), sig), "invalid sig");
    }

    function verifyPayload(bytes calldata payload) internal view returns (bytes calldata pk) {
        // https://docs.aws.amazon.com/enclaves/latest/user/verify-root.html#doc-spec
        // The key order seems to reliably be module_id, digest, timestamp, pcrs, certificate, cabundle, public_key, user_data, nonce.

        uint256 cursor;

        // Map(9) - a9
        // Key: Text(9) - 69
        // Value: Text(9) "module_id" - 69_6D6F64756C655F6964
        // Text(var) - 78
        require(
            bytes12(payload[cursor:cursor += 12]) == bytes12(0xa9_69_6d6f64756c655f6964_78),
            "expected module_id"
        );
        uint256 moduleIdLen = uint256(uint8(payload[cursor]));
        cursor += 1 + moduleIdLen;

        // Key: Text(6) "digest" - 66_646967657374
        // Value: Text(6) "SHA384" - 66_534841333834
        // Key: Text(9) "timestamp" - 69_74696D657374616D70
        // Unsigned - 1b
        require(
            bytes25(payload[cursor:cursor += 25])
                == bytes25(0x66_646967657374_66_534841333834_69_74696d657374616d70_1b),
            "expected digest & timestamp"
        );
        verifyTimestamp(uint64(bytes8(payload[cursor:cursor += 8])));

        // Note: PCR length depends on the digest, but SHA384 is currently the default.
        // Key: Text(4) "pcrs" - 64_70637273
        // Value: Map(16) - b0
        // Key: Unsigned(0) - 00
        // Value: Bytes(short) - 58_30
        require(
            bytes9(payload[cursor:cursor += 9]) == bytes9(0x64_70637273_b0_00_58_30),
            "expected pcrs"
        );
        cursor += _verifyPCRs(payload[cursor:]);

        // Key: Text(11) "certificate" - 6b_6365727469666963617465
        // Value: Bytes(long) - 59
        require(
            bytes13(payload[cursor:cursor += 13]) == bytes13(0x6b_6365727469666963617465_59),
            "expected certificate"
        );
        (bytes calldata enclavePublicKey, uint256 certsLen) = _verifyCerts(payload[cursor:]);
        cursor += certsLen;

        cursor += _verifyUserData(payload[cursor:]);

        require(cursor == payload.length, "unparsed payload");

        return enclavePublicKey;
    }

    function _verifyPCRs(bytes calldata input) internal view returns (uint256 adv) {
        verifyPCRs(
            input[0 * (48 + 3):0 * (48 + 3) + 48],
            input[1 * (48 + 3):1 * (48 + 3) + 48],
            input[2 * (48 + 3):2 * (48 + 3) + 48],
            input[3 * (48 + 3):3 * (48 + 3) + 48],
            input[4 * (48 + 3):4 * (48 + 3) + 48],
            input[8 * (48 + 3):8 * (48 + 3) + 48]
        );
        return 15 * (48 + 3) + 48;
    }

    function _verifyCerts(bytes calldata input)
        internal
        pure
        returns (bytes calldata publicKey, uint256 adv)
    {
        uint256 cursor;

        // Key: Text(8) "cabundle" - 68_636162756e646c65
        // Value: Array(n) - 80
        require(
            bytes10(input[cursor:cursor += 10]) & 0xfffffffffffffffffff0
                == bytes10(0x68_636162756e646c65_80),
            "expected cabundle"
        );
        uint256 cabundleLen = uint256(uint8(input[cursor - 1]) & 0xf);
        bytes calldata pk = input[0:0];
        for (uint256 i; i < cabundleLen; i++) {
            cursor += 1; // skip the bytes marker (59)
            uint256 len = uint256(uint16(bytes2(input[cursor:cursor += 2])));

            bytes32 issuerHash = keccak256("aws.nitro-enclaves");

            if (i == 1) {
                (bytes calldata issuer, bytes calldata subject, bytes calldata nextPk) =
                    _parseX509(input[cursor:cursor + len]);
                require(keccak256(issuer) == issuerHash, "mismatched issuer");
                issuerHash = keccak256(issuer);
                pk = nextPk;
                // cert = input[cursor:cursor + len];
                // TODO: verify against stored root CA
            } else if (i > 1) {
                // TODO: verify i against i-1
            }
            /// else do nothing because the root ca is loaded from storage
            cursor += len;
        }

        // TODO: verify end entity cert

        return (pk, cursor);
    }

    function _parseX509(bytes calldata cert)
        internal
        pure
        returns (bytes calldata issuer, bytes calldata subject, bytes calldata publicKey)
    {
        uint256 cursor;

        // SEQUENCE(var) - 30 82
        require(bytes2(cert[cursor:cursor += 2]) == bytes2(0x30_82), "not tbs");
        uint256 tbsLen = uint16(bytes2(cert[cursor:cursor += 2]));
        cursor += tbsLen;

        // SEQUENCE(10) - 30_0a
        // OID(8) - 06_08
        // ecdsaWithSHA384 - 2a8648ce3d040303
        require(bytes12(cert[cursor:cursor += 12]) == bytes12(0x30_0a_06_08_2a8648ce3d040303), "not alg id");

        return (cert, cert, cert);
    }

    function _verifyUserData(bytes calldata input) internal view returns (uint256 adv) {
        uint256 cursor;

        // Key: Text(10) "public_key" - 6a_7075626c69635f6b6579
        require(
            bytes11(input[cursor:cursor += 11]) == bytes11(0x6a_7075626c69635f6b6579),
            "expected public_key"
        );
        (bytes calldata publicKey, uint256 pkConsumed) = _consumeOptionalBytes(input[cursor:]);
        cursor += pkConsumed;

        // Key: Text(9) "user_data" - 69_757365725f64617461
        require(
            bytes10(input[cursor:cursor += 10]) == bytes10(0x69_757365725f64617461),
            "expected user_data"
        );
        (bytes calldata userdata, uint256 userdataConsumed) = _consumeOptionalBytes(input[cursor:]);
        cursor += userdataConsumed;

        // Key: Text(5) "nonce" - 65_6e6f6e6365
        require(bytes6(input[cursor:cursor += 6]) == bytes6(0x65_6e6f6e6365), "expected nonce");
        (bytes calldata nonce, uint256 nonceConsumed) = _consumeOptionalBytes(input[cursor:]);
        cursor += nonceConsumed;

        handleUserData(publicKey, userdata, nonce);

        return cursor;
    }

    function _consumeOptionalBytes(bytes calldata input)
        internal
        pure
        returns (bytes calldata data, uint256 adv)
    {
        if (input[0] == 0xf6) return (input[0:0], 1);

        require(input[0] == 0x59, "expected pk/ud/nonce bytes");
        uint256 len = uint256(uint16(bytes2(input[1:3])));
        return (input[3:3 + len], len + 3);
    }

    /// @param timestamp The timestamp at which the attestation doc was generated.
    function verifyTimestamp(uint64 timestamp) internal view virtual {}

    /**
     * @param pcr0 A contiguous measure of the contents of the image file, without the section data.
     * @param pcr1 A contiguous measurement of the kernel and boot ramfs data.
     * @param pcr2 A contiguous, in-order measurement of the user applications, without the boot ramfs.
     * @param pcr3 A contiguous measurement of the IAM role assigned to the parent instance. Ensures that the attestation process succeeds only when the parent instance has the correct IAM role.
     * @param pcr4 A contiguous measurement of the ID of the parent instance. Ensures that the attestation process succeeds only when the parent instance has a specific instance ID.
     * @param pcr8 A measure of the signing certificate specified for the enclave image file. Ensures that the attestation process succeeds only when the enclave was booted from an enclave image file signed by a specific certificate.
     */
    function verifyPCRs(
        bytes calldata pcr0,
        bytes calldata pcr1,
        bytes calldata pcr2,
        bytes calldata pcr3,
        bytes calldata pcr4,
        bytes calldata pcr8
    ) internal view virtual {}

    /**
     * @param publicKey An optional DER-encoded key the attestation consumer can use to encrypt data with
     * @param userdata Additional signed user data, defined by protocol
     * @param nonce An optional cryptographic nonce provided by the attestation consumer as a proof of authenticity
     */
    function handleUserData(bytes calldata publicKey, bytes calldata userdata, bytes calldata nonce)
        internal
        view
        virtual
    {}
}
