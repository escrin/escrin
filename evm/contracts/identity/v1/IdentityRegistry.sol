// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {Sapphire} from "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";
import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

import {IPermitter} from "./IPermitter.sol";
import {IdentityId, InterfaceUnsupported, Unauthorized} from "./Types.sol";

abstract contract IdentityRegistry {
    using EnumerableSet for EnumerableSet.AddressSet;
    using Permits for Permits.Permit;

    event RegistrationTransferProposed(IdentityId indexed identityId, address indexed proposed);
    event PermitterChanged(IdentityId indexed identityId);

    event IdentityCreated(IdentityId id);
    event IdentityDestroyed(IdentityId indexed id);
    event IdentityAcquired(IdentityId indexed id, address indexed acquirer);
    event IdentityReleased(IdentityId indexed id, address indexed acquirer);

    struct Registration {
        bool registered;
        address registrant;
    }

    mapping(IdentityId => Registration) private registrations;
    mapping(IdentityId => address) private proposedRegistrants;

    mapping(IdentityId => IPermitter) private permitters;

    mapping(IdentityId => EnumerableSet.AddressSet) private permittedAccounts;
    mapping(address => mapping(IdentityId => Permits.Permit)) private permits;

    modifier onlyRegistrant(IdentityId id) {
        if (msg.sender != registrations[id].registrant) revert Unauthorized();
        _;
    }

    modifier onlyPermitted(IdentityId id) {
        if (!permits[msg.sender][id].isCurrent()) revert Unauthorized();
        _;
    }

    function createIdentity(address permitter, bytes calldata pers)
        external
        returns (IdentityId id)
    {
        id = IdentityId.wrap(uint256(bytes32(Sapphire.randomBytes(32, pers))));
        require(!registrations[id].registered, "unlucky");
        registrations[id] = Registration({registered: true, registrant: msg.sender});
        permitters[id] = _requireIsPermitter(permitter);
        _whenIdentityCreated(id, pers);
        emit IdentityCreated(id);
    }

    function destroyIdentity(IdentityId id) external onlyRegistrant(id) {
        delete registrations[id].registrant;
        delete proposedRegistrants[id];
        delete permitters[id];
        EnumerableSet.AddressSet storage permitted = permittedAccounts[id];
        for (uint256 i; i < permitted.length(); i++) {
            address account = permitted.at(i);
            delete permits[account][id];
            permitted.remove(account);
        }
        _whenIdentityDestroyed(id);
        emit IdentityDestroyed(id);
    }

    function setPermitter(IdentityId id, address permitter) external onlyRegistrant(id) {
        permitters[id] = _requireIsPermitter(permitter);
        emit PermitterChanged(id);
    }

    function proposeRegistrationTransfer(IdentityId id, address to) external onlyRegistrant(id) {
        proposedRegistrants[id] = to;
        emit RegistrationTransferProposed(id, to);
    }

    function acceptRegistrationTransfer(IdentityId id) external {
        address proposed = proposedRegistrants[id];
        if (msg.sender != proposed) revert Unauthorized();
        registrations[id].registrant = proposed;
        delete proposedRegistrants[id];
    }

    function acquireIdentity(
        IdentityId id,
        address beneficiary,
        bytes calldata context,
        bytes calldata authz
    ) external {
        (bool allow, uint64 expiry) = permitters[id].grantPermit({
            identity: id,
            requester: msg.sender,
            beneficiary: beneficiary,
            context: context,
            authz: authz
        });
        if (!allow) revert Unauthorized();
        permits[beneficiary][id] = Permits.Permit({allow: allow, expiry: expiry});
        permittedAccounts[id].add(beneficiary);
        emit IdentityAcquired(id, beneficiary);
    }

    function releaseIdentity(
        IdentityId id,
        address beneficiary,
        bytes calldata context,
        bytes calldata authz
    ) external {
        bool allow = permitters[id].revokePermit({
            identity: id,
            requester: msg.sender,
            beneficiary: beneficiary,
            context: context,
            authz: authz
        });
        if (!allow) revert Unauthorized();
        delete permits[beneficiary][id];
        permittedAccounts[id].remove(beneficiary);
        emit IdentityReleased(id, beneficiary);
    }

    function hasIdentity(address account, IdentityId id) external view returns (bool) {
        return permits[account][id].isCurrent();
    }

    function getRegistrant(IdentityId id)
        external
        view
        returns (address current, address proposed)
    {
        current = registrations[id].registrant;
        proposed = proposedRegistrants[id];
    }

    function _requireIsPermitter(address authorizer) internal view returns (IPermitter) {
        if (!ERC165Checker.supportsInterface(authorizer, type(IPermitter).interfaceId)) {
            revert InterfaceUnsupported();
        }
        return IPermitter(authorizer);
    }

    function _whenIdentityCreated(IdentityId id, bytes calldata pers) internal virtual;

    function _whenIdentityDestroyed(IdentityId id) internal virtual;
}

library Permits {
    struct Permit {
        bool allow;
        uint64 expiry;
    }

    function isCurrent(Permit memory p) internal view returns (bool) {
        return p.allow && (p.expiry == 0 || p.expiry > block.timestamp);
    }
}
