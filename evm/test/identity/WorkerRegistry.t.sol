// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {Test} from "forge-std/Test.sol";

import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

import {WorkerId, WorkerRegistryV1, IIdentityAuthorizerV1} from "../../contracts/identity/WorkerRegistry.sol";

contract WorkerRegistryV1Test is Test {
    address private constant AUTHORIZER = address(4242);

    WorkerRegistryV1 private reg;

    function setUp() public {
        vm.prank(address(0));
        reg = new WorkerRegistryV1();

        vm.mockCall(
            AUTHORIZER,
            abi.encodeWithSelector(
                IERC165.supportsInterface.selector,
                type(IIdentityAuthorizerV1).interfaceId
            ),
            abi.encode(true)
        );
        vm.mockCall(
            AUTHORIZER,
            abi.encodeWithSelector(IERC165.supportsInterface.selector, type(IERC165).interfaceId),
            abi.encode(true)
        );
    }

    function testRegisterWorker() public {
        // vm.expectEmit();
        // emit WorkerRegistryV1.WorkerRegistered(WorkerId.wrap(0));
        WorkerId workerId1 = reg.registerWorker(AUTHORIZER, "abc123");
        require(address(reg.getAuthorizer(workerId1)) == AUTHORIZER, "create1 failed");

        vm.expectRevert("unlucky");
        reg.registerWorker(AUTHORIZER, "abc123");

        WorkerId workerId2 = reg.registerWorker(AUTHORIZER, "xyz789");
        require(address(reg.getAuthorizer(workerId2)) == AUTHORIZER, "create2 failed");

        require(WorkerId.unwrap(workerId1) != WorkerId.unwrap(workerId2), "bad generation");
    }

    function testRegisterWorkerNotIdentityAuthorizer() public {
        vm.expectRevert(WorkerRegistryV1.InterfaceUnsupported.selector);
        reg.registerWorker(address(0), "");
    }

    function testDeregisterWorker() public {
        WorkerId workerId = reg.registerWorker(AUTHORIZER, "");

        vm.prank(address(0));
        vm.expectRevert(WorkerRegistryV1.Unauthorized.selector);
        reg.deregisterWorker(workerId);

        // vm.expectEmit();
        // emit WorkerRegistryV1.WorkerRegistered(workerId);
        reg.deregisterWorker(workerId);

        vm.expectRevert(WorkerRegistryV1.NoSuchWorker.selector);
        reg.getAuthorizer(workerId);
    }

    function testSetAuthorizer() public {
        WorkerId workerId = reg.registerWorker(AUTHORIZER, "");

        vm.prank(address(0));
        vm.expectRevert(WorkerRegistryV1.Unauthorized.selector);
        reg.setAuthorier(workerId, address(AUTHORIZER));

        address newAuthorizer = address(9999);
        vm.mockCall(
            newAuthorizer,
            abi.encodeWithSelector(
                IERC165.supportsInterface.selector,
                type(IIdentityAuthorizerV1).interfaceId
            ),
            abi.encode(true)
        );
        vm.mockCall(
            newAuthorizer,
            abi.encodeWithSelector(IERC165.supportsInterface.selector, type(IERC165).interfaceId),
            abi.encode(true)
        );
        reg.setAuthorier(workerId, newAuthorizer);

        require(address(reg.getAuthorizer(workerId)) == newAuthorizer, "failed to set authorizer");
    }

    function testTransferRegistration() public {
        WorkerId workerId = reg.registerWorker(AUTHORIZER, "");

        vm.prank(address(0));
        vm.expectRevert(WorkerRegistryV1.Unauthorized.selector);
        reg.proposeRegistrationTransfer(workerId, address(this));

        reg.proposeRegistrationTransfer(workerId, address(41));
        reg.proposeRegistrationTransfer(workerId, address(0));
        reg.proposeRegistrationTransfer(workerId, address(42));

        vm.prank(address(41));
        vm.expectRevert(WorkerRegistryV1.Unauthorized.selector);
        reg.acceptRegistrationTransfer(workerId);

        vm.prank(address(42));
        reg.acceptRegistrationTransfer(workerId);

        vm.expectRevert(WorkerRegistryV1.Unauthorized.selector);
        reg.proposeRegistrationTransfer(workerId, address(this));
    }
}
