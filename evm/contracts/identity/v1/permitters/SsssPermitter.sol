// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import {EIP712} from "@openzeppelin/contracts/utils/cryptography/EIP712.sol";
import {MerkleProof} from "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";

import {IIdentityRegistry, IdentityId, Permitter} from "./Permitter.sol";

contract SsssPermitter is Permitter, EIP712 {
    /// The SSSS signature could not be verified against the expected permit.
    error InvalidSsssSignature(); // xwXoVA== 0xc705e854
    /// The SSSS signer/weight proof was not acceptable.
    error InvalidProof(); // Cb3jOQ== 0x09bde339
    /// Not enough SSSSs signed the permit.
    error QuorumNotReached(); // qiamkw== 0xaa26a693
    /// The nonce was already used.
    error NonceUsed(); // H21a7w== 0x1f6d5aef

    event PolicyChange(IdentityId identity);
    event ApproverChange(IdentityId identity);

    mapping(IdentityId => bytes32) public policyHashes;
    mapping(IdentityId => bytes32) public approverRoots;
    mapping(IdentityId => mapping(bytes32 => bool)) private burntNonces;

    constructor(address upstream) Permitter(upstream) EIP712("SsssPermitter", "1") {}

    modifier onlyRegistrant(IdentityId identity) {
        (address registrant,) = _getIdentityRegistry().getRegistrant(identity);
        if (msg.sender != registrant) revert Unauthorized();
        _;
    }

    function setPolicyHash(IdentityId identity, bytes32 policyHash)
        external
        onlyRegistrant(identity)
    {
        policyHashes[identity] = policyHash;
        emit PolicyChange(identity);
    }

    function setApproversRoot(IdentityId identity, bytes32 signersRoot, uint256 threshold)
        external
        onlyRegistrant(identity)
    {
        approverRoots[identity] = _calculateApproversRoot(signersRoot, threshold);
        emit ApproverChange(identity);
    }

    function _acquireIdentity(
        IdentityId identity,
        address recipient,
        uint64 duration,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual override returns (uint64) {
        _acqRelIdentity(identity, recipient, duration, true, context, authorization);
        return duration;
    }

    function _releaseIdentity(
        IdentityId identity,
        address recipient,
        bytes calldata context,
        bytes calldata authorization
    ) internal virtual override {
        _acqRelIdentity(identity, recipient, 0, false, context, authorization);
    }

    function _acqRelIdentity(
        IdentityId identity,
        address recipient,
        uint64 duration,
        bool grant,
        bytes calldata context,
        bytes calldata authorization
    ) private {
        // TODO: decode into calldata arrays to save a few thousand gas
        (uint256 threshold, bytes memory nonce, bytes memory pk, uint256 baseBlock) =
            abi.decode(context, (uint256, bytes, bytes, uint256));
        (bytes32[] memory proof, bool[] memory proofFlags, Signature[] memory signatures) =
            abi.decode(authorization, (bytes32[], bool[], Signature[]));

        if (signatures.length < threshold) revert QuorumNotReached();

        bytes32 permitDigest = _hashTypedDataV4(
            keccak256(
                abi.encode(
                    keccak256(
                        "SsssPermit(address registry,bytes32 identity,address recipient,bool grant,uint64 duration,bytes nonce,bytes pk,uint256 baseblock)"
                    ),
                    address(_getIdentityRegistry()),
                    IdentityId.unwrap(identity),
                    recipient,
                    grant,
                    duration,
                    keccak256(nonce),
                    keccak256(pk),
                    baseBlock
                )
            )
        );

        bytes32[] memory leaves = new bytes32[](signatures.length);
        for (uint256 i; i < signatures.length; i++) {
            if (
                ECDSA.recover(permitDigest, signatures[i].r, signatures[i].vs)
                    != signatures[i].signer
            ) {
                revert InvalidSsssSignature();
            }
            leaves[i] = keccak256(bytes.concat(keccak256(abi.encode(signatures[i].signer))));
        }

        bytes32 signersRoot = MerkleProof.processMultiProof(proof, proofFlags, leaves);

        if (_calculateApproversRoot(signersRoot, threshold) != approverRoots[identity]) {
            revert InvalidProof();
        }

        bytes32 nonceHash = keccak256(nonce);
        if (burntNonces[identity][nonceHash]) revert NonceUsed();
        burntNonces[identity][nonceHash] = true;
    }

    struct Signature {
        address signer;
        bytes32 r;
        bytes32 vs;
    }

    function _calculateApproversRoot(bytes32 signersRoot, uint256 threshold)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(abi.encode(signersRoot, threshold));
    }
}
