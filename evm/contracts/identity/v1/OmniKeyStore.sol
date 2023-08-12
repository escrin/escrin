// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {Sapphire} from "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";

import {IdentityId, IdentityRegistry} from "./IdentityRegistry.sol";

contract OmniKeyStore is IdentityRegistry {
    type Key is bytes32;

    /// The requested key has not been provisioned.
    error KeyNotProvisioned();
    /// The requested key has already been provisioned.
    error KeyAlreadyProvisioned();

    mapping(IdentityId => Key) private primaryKeys;
    mapping(IdentityId => Key) private secondaryKeys;

    function getKey(IdentityId identityId) external view onlyPermitted(identityId) returns (Key) {
        return primaryKeys[identityId];
    }

    function getSecondaryKey(IdentityId identityId)
        external
        view
        onlyPermitted(identityId)
        returns (Key)
    {
        Key key = secondaryKeys[identityId];
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
        return Key.wrap(bytes32(Sapphire.randomBytes(32, pers)));
    }
}
