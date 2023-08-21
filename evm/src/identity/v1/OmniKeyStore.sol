// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import {EIP712} from "@openzeppelin/contracts/utils/cryptography/EIP712.sol";

import {randomBytes} from "../../Utilities.sol";
import {IdentityId, IdentityRegistry, Permits} from "./IdentityRegistry.sol";
import {Unauthorized} from "./Types.sol";

contract OmniKeyStore is IdentityRegistry, EIP712 {
    using Permits for Permit;

    type Key is bytes32;

    /// The requested key has not been provisioned.
    error KeyNotProvisioned();
    /// The requested key has already been provisioned.
    error KeyAlreadyProvisioned();

    struct SignedKeyRequest {
        KeyRequest req;
        bytes sig;
    }

    struct KeyRequest {
        IdentityId identity;
        address requester;
        uint256 expiry;
    }

    mapping(IdentityId => Key) private primaryKeys;
    mapping(IdentityId => Key) private secondaryKeys;

    constructor() EIP712("OmniKeyStore", "1") {}

    modifier onlyPermitted(SignedKeyRequest calldata signedKeyReq) {
        KeyRequest calldata req = signedKeyReq.req;
        if (block.number >= req.expiry) revert Unauthorized();
        bytes32 typeHash =
            keccak256("KeyRequest(uint256 identity,address requester,uint256 expiry)");
        bytes32 digest = _hashTypedDataV4(keccak256(abi.encode(typeHash, req)));
        address signer = ECDSA.recover(digest, signedKeyReq.sig);
        Permit memory permit = readPermit(req.requester, req.identity);
        if (signer != req.requester || !permit.isActive()) revert Unauthorized();
        _;
    }

    function getKey(SignedKeyRequest calldata signedKeyReq)
        external
        view
        onlyPermitted(signedKeyReq)
        returns (Key)
    {
        return primaryKeys[signedKeyReq.req.identity];
    }

    function getSecondaryKey(SignedKeyRequest calldata signedKeyReq)
        external
        view
        onlyPermitted(signedKeyReq)
        returns (Key)
    {
        Key key = secondaryKeys[signedKeyReq.req.identity];
        if (Key.unwrap(key) == 0) revert KeyNotProvisioned();
        return key;
    }

    function provisionSecondaryKey(IdentityId identityId, bytes calldata pers)
        external
        onlyRegistrant(identityId)
    {
        Key key = secondaryKeys[identityId];
        if (Key.unwrap(key) != 0) revert KeyAlreadyProvisioned();
        secondaryKeys[identityId] = _generateKey(pers);
    }

    function rotateKeys(IdentityId identityId) external onlyRegistrant(identityId) {
        primaryKeys[identityId] = secondaryKeys[identityId];
        secondaryKeys[identityId] = Key.wrap(0);
    }

    function _whenIdentityCreated(IdentityId id, bytes calldata pers) internal override {
        primaryKeys[id] = _generateKey(pers);
    }

    function _whenIdentityDestroyed(IdentityId id) internal override {
        primaryKeys[id] = Key.wrap(0);
        secondaryKeys[id] = Key.wrap(0);
    }

    function _generateKey(bytes calldata pers) internal view returns (Key) {
        return Key.wrap(bytes32(randomBytes(32, pers)));
    }
}
