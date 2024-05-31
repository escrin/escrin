// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {Sapphire} from "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";
import {ERC165, IERC165} from "@openzeppelin/contracts/utils/introspection/ERC165.sol";
import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

import {IdentityId, IIdentityRegistry} from "./IIdentityRegistry.sol";
import {IPermitter} from "./IPermitter.sol";

contract IdentityRegistry is IIdentityRegistry, ERC165 {
    struct Registration {
        bool registered;
        address registrant;
    }

    mapping(IdentityId => Registration) private registrations;
    mapping(IdentityId => address) private proposedRegistrants;
    mapping(IdentityId => IPermitter) private permitters;
    mapping(address => mapping(IdentityId => Permit)) private permits;
    mapping(IdentityId => bool) private destroyed;

    modifier onlyRegistrant(IdentityId id) {
        if (msg.sender != registrations[id].registrant) revert Unauthorized();
        _;
    }

    modifier onlyPermitter(IdentityId id) {
        if (msg.sender != address(permitters[id])) revert Unauthorized();
        _;
    }

    function createIdentity(address permitter, bytes calldata pers)
        external
        override
        returns (IdentityId id)
    {
        id = IdentityId.wrap(bytes32(_randomBytes(32, pers)));
        require(!registrations[id].registered, "unlucky");
        registrations[id] = Registration({registered: true, registrant: msg.sender});
        permitters[id] = _requireIsPermitter(permitter);
        _whenIdentityCreated(id, pers);
        emit IdentityCreated(id);
    }

    function destroyIdentity(IdentityId id) external override onlyRegistrant(id) {
        delete registrations[id].registrant;
        delete proposedRegistrants[id];
        delete permitters[id];
        destroyed[id] = true;
        _whenIdentityDestroyed(id);
        emit IdentityDestroyed(id);
    }

    function setPermitter(IdentityId id, address permitter) external override onlyRegistrant(id) {
        permitters[id] = _requireIsPermitter(permitter);
        emit PermitterChanged(id);
    }

    function proposeRegistrationTransfer(IdentityId id, address to)
        external
        override
        onlyRegistrant(id)
    {
        proposedRegistrants[id] = to;
        emit RegistrationTransferProposed(id, to);
    }

    function acceptRegistrationTransfer(IdentityId id) external override {
        address proposed = proposedRegistrants[id];
        if (msg.sender != proposed) revert Unauthorized();
        registrations[id].registrant = proposed;
        delete proposedRegistrants[id];
    }

    function grantIdentity(IdentityId id, address to, uint64 expiry)
        external
        override
        onlyPermitter(id)
    {
        permits[to][id] = Permit({expiry: expiry});
        emit IdentityGranted(id, to);
    }

    function revokeIdentity(IdentityId id, address from) external override onlyPermitter(id) {
        delete permits[from][id];
        emit IdentityRevoked(id, from);
    }

    function getPermitter(IdentityId id) external view override returns (IPermitter) {
        return permitters[id];
    }

    function readPermit(address holder, IdentityId id)
        public
        view
        override
        returns (Permit memory)
    {
        if (destroyed[id]) return Permit({expiry: 0});
        return permits[holder][id];
    }

    function getRegistrant(IdentityId id)
        external
        view
        override
        returns (address current, address proposed)
    {
        current = registrations[id].registrant;
        proposed = proposedRegistrants[id];
    }

    function supportsInterface(bytes4 interfaceId)
        public
        view
        virtual
        override(ERC165, IERC165)
        returns (bool)
    {
        return interfaceId == type(IIdentityRegistry).interfaceId
            || super.supportsInterface(interfaceId);
    }

    function _requireIsPermitter(address permitter) internal view returns (IPermitter) {
        if (!ERC165Checker.supportsInterface(permitter, type(IPermitter).interfaceId)) {
            revert InterfaceUnsupported();
        }
        return IPermitter(permitter);
    }

    function _whenIdentityCreated(IdentityId id, bytes calldata pers) internal virtual {}

    function _whenIdentityDestroyed(IdentityId id) internal virtual {}

    /// Generates pseudorandom bytes.
    /// When run on Sapphire, the random bytes are privte and cryptographically secure.
    function _randomBytes(uint256 count, bytes calldata pers)
        internal
        view
        returns (bytes memory)
    {
        if (block.chainid == 0x5aff || block.chainid == 0x5afe || block.chainid == 0x5afd) {
            return Sapphire.randomBytes(count, pers);
        }
        bool isLocalnet = block.chainid == 1337 || block.chainid == 31337;
        uint256 words = (count + 31) >> 5;
        bytes memory out = new bytes(words << 5);
        bytes memory preSeed = isLocalnet
            ? abi.encodePacked(msg.sender, count, pers)
            : abi.encodePacked(
                msg.sender,
                blockhash(block.number),
                block.timestamp,
                block.prevrandao,
                block.coinbase,
                count,
                pers
            );
        bytes32 seed = keccak256(preSeed);
        for (uint256 i = 0; i < words; i++) {
            if (!isLocalnet) {
                unchecked {
                    seed = keccak256(abi.encodePacked(seed, i, blockhash(block.number - i - 1)));
                }
            }
            assembly ("memory-safe") {
                mstore(add(out, add(32, mul(32, i))), seed)
            }
        }
        assembly ("memory-safe") {
            mstore(out, count)
        }
        return out;
    }
}
