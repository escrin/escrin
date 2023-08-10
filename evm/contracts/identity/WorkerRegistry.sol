// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {Sapphire} from "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";
import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";
import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

type WorkerId is bytes32;

interface IIdentityAuthorizerV1 is IERC165 {
    function assumeIdentity(WorkerId id, bytes calldata context, bytes calldata authz)
        external
        returns (bool);
}

contract WorkerRegistryV1 {
    /// The identified worker is not registered.
    error NoSuchWorker(); // bd88a936 vYipNg==
    /// The action is disallowed.
    error Unauthorized(); // 82b42900 grQpAA==
    /// The provided contract address does not support the correct interface.
    error InterfaceUnsupported(); // bbaa55aa u6pVqg==

    event WorkerRegistered(WorkerId id);
    event WorkerDeregistered(WorkerId indexed id);

    mapping(WorkerId => address) internal registrants;
    mapping(WorkerId => address) internal proposedRegistrants;
    mapping(WorkerId => IIdentityAuthorizerV1) internal authorizers;

    modifier onlyRegistrant(WorkerId id) {
        if (msg.sender != registrants[id]) revert Unauthorized();
        _;
    }

    function registerWorker(address authorizer, bytes calldata entropy)
        external
        returns (WorkerId id)
    {
        id = _generateWorkerId(entropy);
        require(registrants[id] == address(0), "unlucky");
        registrants[id] = msg.sender;
        authorizers[id] = _checkIsAuthorizer(authorizer);
        emit WorkerRegistered(id);
    }

    function deregisterWorker(WorkerId id) external onlyRegistrant(id) {
        delete registrants[id];
        delete proposedRegistrants[id];
        delete authorizers[id];
        emit WorkerDeregistered(id);
    }

    function setAuthorier(WorkerId id, address authorizer) external onlyRegistrant(id) {
        authorizers[id] = _checkIsAuthorizer(authorizer);
    }

    function proposeRegistrationTransfer(WorkerId id, address to) external onlyRegistrant(id) {
        proposedRegistrants[id] = to;
    }

    function acceptRegistrationTransfer(WorkerId id) external {
        address proposed = proposedRegistrants[id];
        if (msg.sender != proposed) revert Unauthorized();
        registrants[id] = proposed;
        delete proposedRegistrants[id];
    }

    function getAuthorizer(WorkerId id) external view returns (IIdentityAuthorizerV1) {
        IIdentityAuthorizerV1 authorizer = authorizers[id];
        if (address(authorizer) == address(0)) revert NoSuchWorker();
        return authorizer;
    }

    function _generateWorkerId(bytes calldata pers) internal view returns (WorkerId) {
        return WorkerId.wrap(
            block.chainid == 0x5aff || block.chainid == 0x5afe
                ? bytes32(Sapphire.randomBytes(16, pers))
                : keccak256(bytes.concat(bytes32(block.prevrandao), pers))
        );
    }

    function _checkIsAuthorizer(address authorizer) internal view returns (IIdentityAuthorizerV1) {
        if (!ERC165Checker.supportsInterface(authorizer, type(IIdentityAuthorizerV1).interfaceId)) {
            revert InterfaceUnsupported();
        }
        return IIdentityAuthorizerV1(authorizer);
    }
}
