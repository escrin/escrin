// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

// import "forge-std/console2.sol";

import {
    Sapphire, sha384 as _sha384
} from "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";

import {IdentityId, IIdentityRegistry, Permitter} from "./Permitter.sol";

// Whether to strictly validate attestation doc exons in return for paying up to 30k more gas.
bool constant STRICT = true;

abstract contract BaseNitroEnclavePermitter is Permitter {
    /// The presented attestation document has already been used to acquire an identity using this permitter.
    /// If you want a batch identity acquisition function, please file an issue and it will be made!
    error DocAlreadyUsed(); // HTMAwA==
    /// The attestation is not bound to the request.
    error BindingMismatch(); // Q4xIcw==

    mapping(bytes32 => IdentityId) public burnt;

    constructor(address upstream) Permitter(upstream) {}

    function _acquireIdentity(
        IdentityId identity,
        address requester,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual override returns (uint64 expiry) {
        uint256 maxDuration = _getPermitMaxDuration(identity, requester, context);
        if (maxDuration < uint256(duration)) revert DurationTooLong();
        _processAttestation(identity, requester, context, authorization, false);
        return uint64(block.timestamp + uint256(duration));
    }

    function _releaseIdentity(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual override {
        _processAttestation(identity, requester, context, authorization, true);
    }

    function _processAttestation(
        IdentityId identity,
        address requester,
        bytes calldata context,
        bytes calldata doc,
        bool release
    ) internal {
        uint256 nbf = block.timestamp - _getPermitMaxDuration(identity, requester, context);
        NE.UserData memory userdata = NE.verifyAttestationDocument(doc, _getPCRs(identity), nbf);
        if (IdentityId.unwrap(burnt[userdata.nonce]) != 0) revert DocAlreadyUsed();
        bytes32 expectedBinding = keccak256(
            abi.encode(block.chainid, _getIdentityRegistry(), identity, requester, !release)
        );
        if (userdata.binding != expectedBinding) revert BindingMismatch();
        burnt[userdata.nonce] = identity;
    }

    function _getPCRs(IdentityId identity) internal view virtual returns (NE.PcrSelector memory);

    function _getPermitMaxDuration(IdentityId identity, address requester, bytes calldata context)
        internal
        view
        virtual
        returns (uint256)
    {
        (identity, requester, context) = (identity, requester, context);
        return 30 minutes;
    }
}

contract StaticNitroEnclavePermitter is BaseNitroEnclavePermitter {
    uint16 public immutable pcrMask;
    bytes32 public immutable pcrHash;

    constructor(address upstream, uint16 mask, bytes32 hash) BaseNitroEnclavePermitter(upstream) {
        pcrMask = mask;
        pcrHash = hash;
    }

    function _getPCRs(IdentityId) internal view virtual override returns (NE.PcrSelector memory) {
        return NE.PcrSelector({mask: pcrMask, hash: pcrHash});
    }
}

contract MultiNitroEnclavePermitter is BaseNitroEnclavePermitter {
    mapping(IdentityId => NE.PcrSelector) public pcrs;

    constructor(address upstream) BaseNitroEnclavePermitter(upstream) {}

    function setPCRs(IdentityId identity, NE.PcrSelector calldata pcrSel) external {
        (address registrant,) = IIdentityRegistry(_getIdentityRegistry()).getRegistrant(identity);
        if (msg.sender != registrant) revert Unauthorized();
        pcrs[identity] = pcrSel;
    }

    function _getPCRs(IdentityId identity)
        internal
        view
        virtual
        override
        returns (NE.PcrSelector memory)
    {
        return pcrs[identity];
    }
}

library NE {
    error ContractExpired(); // B5DU4w==

    struct PcrSelector {
        /// A 16-bit flags field, defining which PCRs are included in the hash. Only valid PCRs may be specified.
        /// pcr0 A contiguous measure of the contents of the image file, without the section data.
        /// pcr1 A contiguous measurement of the kernel and boot ramfs data.
        /// pcr2 A contiguous, in-order measurement of the user applications, without the boot ramfs.
        /// pcr3 A measurement of the IAM role assigned to the parent instance. Ensures that the attestation process succeeds only when the parent instance has the correct IAM role.
        /// pcr4 A measurement of the ID of the parent instance. Ensures that the attestation process succeeds only when the parent instance has a specific instance ID.
        /// pcr8 A measurement of the signing certificate specified for the enclave image file. Ensures that the attestation process succeeds only when the enclave was booted from an enclave image file signed by a specific certificate.
        uint16 mask;
        /// The hash of `uint256(mask & 0x11f) || concat(pcr[i] if mask[i] else "" for i in (0, 1, 2, 3, 4, 8))`
        bytes32 hash;
    }

    struct UserData {
        bytes32 binding;
        bytes32 nonce;
    }

    // CN: aws.nitro-enclaves
    bytes constant ROOT_CA_KEY =
        hex"04fc0254eba608c1f36870e29ada90be46383292736e894bfff672d989444b5051e534a4b1f6dbe3c0bc581a32b7b176070ede12d69a3fea211b66e752cf7dd1dd095f6f1370f4170843d9dc100121e4cf63012809664487c9796284304dc53ff4";
    uint256 constant ROOT_CA_EXPIRY = 2519044085;

    function verifyAttestationDocument(bytes calldata doc, PcrSelector memory pcrs, uint256 nbf)
        internal
        view
        returns (UserData memory userdata)
    {
        if (block.timestamp >= ROOT_CA_EXPIRY) revert ContractExpired();

        if (STRICT) {
            // Array(4) - 84
            // Bytes(4) - 44 // 4 bytes of cbor-encoded protected header
            // Map(1) - a1 // protected header
            // Key: Unsigned(1) - 01
            // Value: Signed(-35) - 3822 // P-384
            // Map(0) - a0 // unprotected header
            // Bytes(long) - 59 // payload
            require(bytes8(doc[0:8]) == bytes8(0x84_44_a1_01_3822_a0_59), "invalid doc");
        }
        uint256 payloadSize = uint16(bytes2(doc[8:10]));
        uint256 payloadStart = 10;
        uint256 payloadEnd = payloadStart + payloadSize;
        bytes calldata payload = doc[payloadStart:payloadEnd];

        if (STRICT) {
            // Bytes(short) - 58
            // Unsigned(96) - 60
            require(bytes2(doc[payloadEnd:payloadEnd + 2]) == bytes2(0x58_60));
        }
        uint256 sigStart = payloadEnd + 2;

        bytes calldata pk;
        (pk, userdata) = verifyPayload(payload, pcrs, nbf);
        bytes calldata sig = doc[sigStart:sigStart + 0x60];
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
        bytes memory xTag = _getDerUintHdr(uint8(sig[0]), 48);
        bytes memory yTag = _getDerUintHdr(uint8(sig[48]), 48);
        bytes memory sigDer = abi.encodePacked(
            bytes1(0x30), // SEQUENCE
            bytes1(uint8(96 + xTag.length + yTag.length)), // seq len
            xTag,
            sig[0:48],
            yTag,
            sig[48:96]
        );
        Sig.verifyP384(pk, coseSign1, sigDer);
    }

    function verifyPayload(bytes calldata payload, PcrSelector memory pcrs, uint256 nbf)
        internal
        view
        returns (bytes calldata pk, UserData memory userdata)
    {
        // https://docs.aws.amazon.com/enclaves/latest/user/verify-root.html#doc-spec
        // The key order seems to reliably be module_id, digest, timestamp, pcrs, certificate, cabundle, public_key, user_data, nonce.

        uint256 cursor;

        if (STRICT) {
            // Map(9) - a9
            // Key: Text(9) - 69
            // Value: Text(9) "module_id" - 69_6D6F64756C655F6964
            // Text(var) - 78
            require(
                bytes12(payload[cursor:cursor + 12]) == bytes12(0xa9_69_6d6f64756c655f6964_78),
                "expected module_id"
            );
        }
        cursor += 12;
        uint256 moduleIdLen = uint256(uint8(payload[cursor]));
        cursor += 1 + moduleIdLen;

        if (STRICT) {
            // Key: Text(6) "digest" - 66_646967657374
            // Value: Text(6) "SHA384" - 66_534841333834
            // Key: Text(9) "timestamp" - 69_74696D657374616D70
            // Unsigned - 1b
            require(
                bytes25(payload[cursor:cursor + 25])
                    == bytes25(0x66_646967657374_66_534841333834_69_74696d657374616d70_1b),
                "expected digest & timestamp"
            );
        }
        cursor += 25;
        uint256 timestamp = uint256(uint64(bytes8(payload[cursor:cursor += 8])));
        require(timestamp > nbf, "old attestation");

        cursor += _verifyPCRs(payload[cursor:], pcrs);

        uint256 adv;

        (pk, adv) = _verifyCerts(payload[cursor:]);
        cursor += adv;

        (userdata, adv) = _getUserData(payload[cursor:]);
        cursor += adv;

        require(cursor == payload.length, "unparsed payload");
    }

    function _verifyCerts(bytes calldata input)
        internal
        view
        returns (bytes calldata publicKey, uint256 adv)
    {
        uint256 cursor;

        if (STRICT) {
            // Key: Text(11) "certificate" - 6b_6365727469666963617465
            // Value: Bytes(long) - 59
            require(
                bytes13(input[cursor:cursor + 13]) == bytes13(0x6b_6365727469666963617465_59),
                "expected certificate"
            );
        }
        cursor += 13;
        uint256 certLen = uint16(bytes2(input[cursor:cursor += 2]));
        bytes calldata cert = input[cursor:cursor += certLen];

        if (STRICT) {
            // Key: Text(8) "cabundle" - 68_636162756e646c65
            // Value: Array(n) - 80
            require(
                bytes10(input[cursor:cursor + 10]) & 0xfffffffffffffffffff0
                    == bytes10(0x68_636162756e646c65_80),
                "expected cabundle"
            );
        }
        cursor += 10;
        uint256 cabundleLen = uint256(uint8(input[cursor - 1]) & 0xf);
        bytes calldata pk = input[0:0];

        bytes32 issuerHash = keccak256("aws.nitro-enclaves");
        bytes32 serial;

        for (uint256 i; i < cabundleLen; i++) {
            if (STRICT) {
                // Bytes(long) - 59
                require(bytes1(input[cursor:cursor + 1]) == 0x59, "invalid cabundle");
            }
            cursor += 1;
            uint256 len = uint256(uint16(bytes2(input[cursor:cursor += 2])));

            if (i == 0) {
                // Skip the zeroth (root) cert, which is loaded from storage.
                cursor += len;
                continue;
            }

            bytes calldata icert = input[cursor:cursor += len];
            (serial, issuerHash, pk) = X509.verify(icert, issuerHash, i == 1 ? ROOT_CA_KEY : pk);
        }

        (serial,, pk) = X509.verify(cert, issuerHash, pk);

        return (pk, cursor);
    }

    function _verifyPCRs(bytes calldata input, PcrSelector memory sel)
        internal
        pure
        returns (uint256 adv)
    {
        if (STRICT) {
            // Note: PCR length depends on the digest, but SHA384 is currently the default.
            // Key: Text(4) "pcrs" - 64_70637273
            // Value: Map(16) - b0
            // Key: Unsigned(0) - 00
            // Value: Bytes(short) - 58_30
            require(bytes9(input[0:9]) == bytes9(0x64_70637273_b0_00_58_30), "expected pcrs");
        }
        input = input[9:];
        uint256 mask = uint256(sel.mask) & 0x011f;
        bytes32 pcrHash = keccak256(
            abi.encodePacked(
                mask,
                ((mask >> 0) & 1) == 1 ? input[0 * (48 + 3):0 * (48 + 3) + 48] : input[0:0],
                ((mask >> 1) & 1) == 1 ? input[1 * (48 + 3):1 * (48 + 3) + 48] : input[0:0],
                ((mask >> 2) & 1) == 1 ? input[2 * (48 + 3):2 * (48 + 3) + 48] : input[0:0],
                ((mask >> 3) & 1) == 1 ? input[3 * (48 + 3):3 * (48 + 3) + 48] : input[0:0],
                ((mask >> 4) & 1) == 1 ? input[4 * (48 + 3):4 * (48 + 3) + 48] : input[0:0],
                ((mask >> 8) & 1) == 1 ? input[8 * (48 + 3):8 * (48 + 3) + 48] : input[0:0]
            )
        );
        require(pcrHash == sel.hash, "wrong pcrs");
        return 15 * (48 + 3) + 48 + 9;
    }

    function _getUserData(bytes calldata input)
        internal
        pure
        returns (UserData memory userdata, uint256 adv)
    {
        uint256 cursor;

        if (STRICT) {
            // Key: Text(10) "public_key" - 6a_7075626c69635f6b6579
            require(
                bytes11(input[cursor:cursor + 11]) == bytes11(0x6a_7075626c69635f6b6579),
                "expected public_key"
            );
        }
        cursor += 11;
        (, uint256 pkConsumed) = _consumeOptionalBytes(input[cursor:]);
        cursor += pkConsumed;

        if (STRICT) {
            // Key: Text(9) "user_data" - 69_757365725f64617461
            require(
                bytes10(input[cursor:cursor + 10]) == bytes10(0x69_757365725f64617461),
                "expected user_data"
            );
        }
        cursor += 10;
        (bytes calldata binding, uint256 userdataConsumed) = _consumeOptionalBytes(input[cursor:]);
        cursor += userdataConsumed;
        userdata.binding = bytes32(binding);

        if (STRICT) {
            // Key: Text(5) "nonce" - 65_6e6f6e6365
            require(bytes6(input[cursor:cursor + 6]) == bytes6(0x65_6e6f6e6365), "expected nonce");
        }
        cursor += 6;
        (bytes calldata nonce, uint256 nonceConsumed) = _consumeOptionalBytes(input[cursor:]);
        cursor += nonceConsumed;
        userdata.nonce = bytes32(nonce);

        adv = cursor;
    }

    function _consumeOptionalBytes(bytes calldata input)
        internal
        pure
        returns (bytes calldata data, uint256 adv)
    {
        if (input[0] == 0xf6 /* null */ || input[0] == 0x40 /* empty */ ) return (input[0:0], 1);
        if (input[0] == 0x58) {
            uint256 len = uint256(uint8(bytes1(input[1:2])));
            return (input[2:2 + len], len + 2);
        }
        if (input[0] == 0x59) {
            uint256 len = uint256(uint16(bytes2(input[1:3])));
            return (input[3:3 + len], len + 3);
        }
        revert("expected userdata bytes");
    }

    function _getDerUintHdr(uint8 leadingByte, uint8 len)
        internal
        pure
        returns (bytes memory tag)
    {
        // If the integer is negative, add a zero byte to make it positive.
        if (leadingByte < 0x80) {
            tag = new bytes(2);
            tag[0] = 0x02;
            tag[1] = bytes1(len);
        } else {
            tag = new bytes(3);
            tag[0] = 0x02;
            tag[1] = bytes1(len + 1);
            tag[2] = 0x00;
        }
    }
}

library X509 {
    error CertExpired(); // CPzuqg==
    error CertNotActive(); // 1LHC+A==

    struct Cert {
        TbsMeta tbs;
        bytes signature;
    }

    struct TbsMeta {
        bytes32 serial;
        bytes32 issuerHash;
        bytes32 subjectHash;
    }

    function verify(bytes calldata cert, bytes32 issuerHash, bytes memory issuerPk)
        internal
        view
        returns (bytes32 serial, bytes32 subjectHash, bytes calldata pk)
    {
        bytes calldata tbs;
        bytes calldata sig;
        (serial, subjectHash, tbs, pk, sig) = parseForIss(cert, issuerHash);

        Sig.verifyP384(issuerPk, tbs, sig);
    }

    function parseForIss(bytes calldata cert, bytes32 expectedIssuerHash)
        internal
        view
        returns (
            bytes32 serial,
            bytes32 sub,
            bytes calldata tbs,
            bytes calldata pk,
            bytes calldata sig
        )
    {
        bytes32 iss;

        if (STRICT) {
            // SEQUENCE(var(2)) - 30_82
            require(bytes2(cert[0:2]) == bytes2(0x30_82), "not cert");
            uint256 certLen = uint16(bytes2(cert[2:4]));
            require(cert.length == certLen + 4, "not cert len");
            require(bytes2(cert[4:6]) == bytes2(0x30_8_2), "not tbs");
        }
        uint256 cursor = 6;
        uint256 tbsLen = uint16(bytes2(cert[cursor:cursor += 2]));
        tbs = cert[cursor - 4:cursor += tbsLen];
        (serial, iss, sub, pk) = _parseTbs(tbs);

        if (STRICT) {
            // SEQUENCE(10) - 30_0a
            // OID(8) - 06_08
            // ecdsaWithSHA384 - 2a8648ce3d040303
            require(
                bytes12(cert[cursor:cursor + 12]) == bytes12(0x30_0a_06_08_2A8648CE3D040303),
                "wrong sig"
            );
        }
        cursor += 12;

        if (STRICT) {
            // BITSTRING(n) - 03_nn_00
            require(cert[cursor] == 0x03 && cert[cursor + 2] == 0, "not sig");
        }
        cursor += 3;
        sig = cert[cursor:];

        require(iss == expectedIssuerHash, "wrong issuer");
    }

    function _parseTbs(bytes calldata tbs)
        internal
        view
        returns (bytes32 serial, bytes32 iss, bytes32 sub, bytes calldata pk)
    {
        uint256 cursor = 4; // skip framing that was already checked in `parse`

        if (STRICT) {
            // version_element(3) - a0_03
            // integer(1) - 02_02
            // version 2 - 02
            require(bytes5(tbs[cursor:cursor + 5]) == bytes5(0xa0_03_02_01_02), "not v2 tbs");
        }
        cursor += 5;

        if (STRICT) {
            // INTEGER - 02
            require(bytes1(tbs[cursor:cursor + 1]) == bytes1(0x02), "not serial");
        }
        cursor += 1;
        uint256 serialLen = uint256(uint8(bytes1(tbs[cursor:cursor += 1])));
        serial = keccak256(tbs[cursor:cursor += serialLen]);

        if (STRICT) {
            // SEQUENCE(10) - 30_0a
            // OID(8) - 06_08
            // ecdsaWithSHA384 - 2a8648ce3d040303
            require(
                bytes12(tbs[cursor:cursor + 12]) == bytes12(0x30_0a_06_08_2A8648CE3D040303),
                "wrong sig"
            );
        }
        cursor += 12;

        (bytes calldata issuer, uint256 issuerSeqLen) = _getVarLenSeq(tbs[cursor:]);
        cursor += issuerSeqLen;
        iss = keccak256(_extractCN(issuer));

        if (STRICT) {
            // SEQUENCE - 30
            require(bytes1(tbs[cursor:cursor + 1]) == bytes1(0x30), "not seq");
        }
        cursor += 1;
        uint256 validityLen = uint8(bytes1(tbs[cursor:cursor += 1]));
        _checkValidity(tbs[cursor:cursor += validityLen]);

        (bytes calldata subject, uint256 subjectSeqLen) = _getVarLenSeq(tbs[cursor:]);
        cursor += subjectSeqLen;
        sub = keccak256(_extractCN(subject));

        if (STRICT) {
            // SEQUENCE - 30
            require(bytes1(tbs[cursor:cursor + 1]) == bytes1(0x30), "not seq");
        }
        cursor += 1;
        uint256 spkiLen = uint256(uint8(bytes1(tbs[cursor:cursor += 1])));
        cursor += spkiLen;
        pk = tbs[cursor - 97:cursor];

        // TODO: consider verifying basicConstraints
    }

    function _extractCN(bytes calldata input) internal pure returns (bytes calldata) {
        uint256 cursor;
        while (cursor != input.length) {
            if (STRICT) {
                require(bytes1(input[cursor:cursor + 1]) == bytes1(0x31), "not rdn");
            }
            cursor += 1;
            uint256 rdnLen = uint256(uint8(bytes1(input[cursor:cursor += 1])));
            // SEQUENCE(n)- 30_xx
            // OID(commonName) - 06_03_550403
            if (bytes7(input[cursor:cursor + 7]) & 0xff00ffffffffff != bytes7(0x30_00_0603550403)) {
                cursor += rdnLen;
                continue;
            }
            cursor += 7; // already checked the OID above
            if (STRICT) {
                require(bytes1(input[cursor:cursor + 1]) == bytes1(0x0c), "not cn");
            }
            cursor += 1;
            uint256 cnLen = uint256(uint8(bytes1(input[cursor:cursor += 1])));
            return input[cursor:cursor + cnLen];
        }
        revert("no CN");
    }

    function _checkValidity(bytes calldata validity) internal view {
        uint256 nbf = _parseISO8601(bytes13(validity[2:15]));
        uint256 exp = _parseISO8601(bytes13(validity[17:30]));
        uint256 live = exp - nbf;
        uint256 fuzz = live < 2 days ? 1 days : live >> 2;
        if (block.timestamp > exp + fuzz) revert CertExpired();
        if (block.timestamp < nbf - fuzz) revert CertNotActive();
    }

    /// Parses an ISO-8601 timestamp (YYMMddHHmmssZ) into an approximate unix timestamp.
    function _parseISO8601(bytes13 isodate) internal pure returns (uint256) {
        if (STRICT) {
            require(isodate[12] == 0x5a, "not utc tz");
        }
        uint256 isonums = (uint256(bytes32(isodate)) >> 160) - 0x303030303030303030303030;
        uint256 ss = ((isonums >> 0x08) & 0xff) * 10 + ((isonums >> 0x00) & 0xff);
        uint256 mm = ((isonums >> 0x18) & 0xff) * 10 + ((isonums >> 0x10) & 0xff);
        uint256 hh = ((isonums >> 0x28) & 0xff) * 10 + ((isonums >> 0x20) & 0xff);
        uint256 dd = ((isonums >> 0x38) & 0xff) * 10 + ((isonums >> 0x30) & 0xff);
        uint256 mM = ((isonums >> 0x48) & 0xff) * 10 + ((isonums >> 0x40) & 0xff);
        uint256 yy = ((isonums >> 0x58) & 0xff) * 10 + ((isonums >> 0x50) & 0xff);
        uint256 cd = _cumulativeDays(mM, yy);
        return (
            ss + (1 minutes * mm) + (1 hours * hh) + 1 days * (dd - 1 + cd)
                + (365.25 days * yy + 946702800)
        );
    }

    function _cumulativeDays(uint256 month, uint256 year) internal pure returns (uint256) {
        if (month == 1) return 0;
        uint256 leap = (year & 3) == 0 ? 1 : 0; // anti-leaps and anti-anti-leps are not required as root cert expires before2100
        if (month == 2) return 31;
        if (month == 3) return 59 + leap;
        if (month == 4) return 90 + leap;
        if (month == 5) return 120 + leap;
        if (month == 6) return 151 + leap;
        if (month == 7) return 181 + leap;
        if (month == 8) return 212 + leap;
        if (month == 9) return 243 + leap;
        if (month == 10) return 273 + leap;
        if (month == 11) return 304 + leap;
        if (month == 12) return 334 + leap;
        revert("bad month");
    }

    /// Extracts a sequence that may have 0, 1, or 2 additional length bytes.
    function _getVarLenSeq(bytes calldata input)
        internal
        pure
        returns (bytes calldata, uint256 adv)
    {
        uint256 cursor;
        if (STRICT) {
            require(bytes1(input[cursor:cursor + 1]) == bytes1(0x30), "not seq");
        }
        cursor += 1;
        uint256 len = uint256(uint8(bytes1(input[cursor:cursor += 1])));
        if (len == 0x81) {
            len = uint256(uint8(bytes1(input[cursor:cursor += 1])));
        } else if (len == 0x82) {
            len = uint256(uint16(bytes2(input[cursor:cursor += 2])));
        } else {
            require((len & 0x80) != 0x80, "seq too big");
        }
        return (input[cursor:cursor += len], cursor);
    }
}

library Sig {
    error InvalidSignature(); // i6pXnw==

    function verifyP384(bytes memory pk, bytes memory message, bytes memory sig) internal view {
        if (block.chainid == 1337 || block.chainid == 31337) return;
        if (block.chainid - 0x5afd > 2) revert("no p384");
        bytes memory hash = _sha384(message);
        bytes memory cpk = _compressP384PK(pk);
        if (!Sapphire.verify(Sapphire.SigningAlg.Secp384r1PrehashedSha384, cpk, hash, "", sig)) {
            revert InvalidSignature();
        }
    }

    /// Compresses a p384 public key.
    /// @dev This function is required for operation on old versions of Sapphire.
    function _compressP384PK(bytes memory pk) internal pure returns (bytes memory) {
        if (pk[0] != 0x04) return pk;
        pk[0] = bytes1(0x02 | (uint8(pk[96]) & 0x01));
        assembly ("memory-safe") {
            mstore(pk, 49)
        }
        return pk;
    }
}
