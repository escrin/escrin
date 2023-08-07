// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {Sapphire} from "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";
import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";
import {ERC165Checker} from "@openzeppelin/contracts/utils/introspection/ERC165Checker.sol";

type WorkerId is bytes32;

interface IIdentityAuthorizerV1 is IERC165 {
    function assumeIdentity(
        WorkerId _id,
        bytes calldata _context,
        bytes calldata _authz
    ) external returns (bool);
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

    modifier onlyRegistrant(WorkerId _id) {
        if (msg.sender != registrants[_id]) revert Unauthorized();
        _;
    }

    function registerWorker(
        address _authorizer,
        bytes calldata _entropy
    ) external returns (WorkerId id) {
        id = _generateWorkerId(_entropy);
        require(registrants[id] == address(0), "unlucky");
        registrants[id] = msg.sender;
        authorizers[id] = _checkIsAuthorizer(_authorizer);
        emit WorkerRegistered(id);
    }

    function deregisterWorker(WorkerId _id) external onlyRegistrant(_id) {
        delete registrants[_id];
        delete proposedRegistrants[_id];
        delete authorizers[_id];
        emit WorkerDeregistered(_id);
    }

    function setAuthorier(WorkerId _id, address _authorizer) external onlyRegistrant(_id) {
        authorizers[_id] = _checkIsAuthorizer(_authorizer);
    }

    function proposeRegistrationTransfer(WorkerId _id, address _to) external onlyRegistrant(_id) {
        proposedRegistrants[_id] = _to;
    }

    function acceptRegistrationTransfer(WorkerId _id) external {
        address proposed = proposedRegistrants[_id];
        if (msg.sender != proposed) revert Unauthorized();
        registrants[_id] = proposed;
        delete proposedRegistrants[_id];
    }

    function getAuthorizer(WorkerId _id) external view returns (IIdentityAuthorizerV1) {
        IIdentityAuthorizerV1 authorizer = authorizers[_id];
        if (address(authorizer) == address(0)) revert NoSuchWorker();
        return authorizer;
    }

    function _generateWorkerId(bytes calldata _pers) internal view returns (WorkerId) {
        return
            WorkerId.wrap(
                block.chainid == 0x5aff || block.chainid == 0x5afe
                    ? bytes32(Sapphire.randomBytes(16, _pers))
                    : keccak256(bytes.concat(bytes32(block.prevrandao), _pers))
            );
    }

    function _checkIsAuthorizer(address _authorizer) internal view returns (IIdentityAuthorizerV1) {
        if (!ERC165Checker.supportsInterface(_authorizer, type(IIdentityAuthorizerV1).interfaceId))
            revert InterfaceUnsupported();
        return IIdentityAuthorizerV1(_authorizer);
    }
}
