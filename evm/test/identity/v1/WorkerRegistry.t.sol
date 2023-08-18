// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

// import {Test} from "forge-std/Test.sol";

// import {Sapphire} from "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";
// import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

// import {IIdentityAuthorizer, IWorkerRegistry, WorkerId, WorkerRegistry} from "../../../contracts/identity/v1/WorkerRegistry.sol";
// import {InterfaceUnsupported, Unauthorized} from "../../../contracts/identity/v1/Types.sol";

// contract RandomBytes {
//     fallback (bytes calldata input) external returns (bytes memory) {
//         (uint256 count, bytes memory pers) = abi.decode(input, (uint256, bytes));
//         uint256 words = (count + 31) >> 5;
//         bytes memory out = new bytes(words << 5);
//         bytes32 seed = keccak256(
//             abi.encodePacked(
//                 msg.sender,
//                 blockhash(block.number),
//                 block.timestamp,
//                 block.prevrandao,
//                 block.coinbase,
//                 count,
//                 pers
//         )
//         );
//         for (uint256 i = 0; i < words; i++) {
//             seed = keccak256(abi.encodePacked(seed, i, blockhash(block.number - i)));
//             assembly {
//                 mstore(add(out, add(32, mul(32, i))), seed)
//             }
//         }
//         assembly {
//             mstore(out, count)
//         }
//         return out;
//     }
// }

// contract MockWorkerRegistry is WorkerRegistry {
//     function supportsInterface(bytes4 interfaceId) external view override returns (bool) {
//         return interfaceId == type(IWorkerRegistry).interfaceId;
//     }
// }

// contract WorkerRegistryTest is Test {
//     address private constant AUTHORIZER = address(4242);

//     WorkerRegistry private reg;

//     function setUp() public {
//         vm.prank(address(0));
//         reg = new MockWorkerRegistry();

//         vm.etch(0x0100000000000000000000000000000000000001, address(new RandomBytes()).code);

//         vm.mockCall(
//             AUTHORIZER,
//             abi.encodeWithSelector(
//                 IERC165.supportsInterface.selector,
//                 type(IIdentityAuthorizer).interfaceId
//             ),
//             abi.encode(true)
//         );
//         vm.mockCall(
//             AUTHORIZER,
//             abi.encodeWithSelector(IERC165.supportsInterface.selector, type(IERC165).interfaceId),
//             abi.encode(true)
//         );
//     }

//     function testRegisterWorker() public {
//         // vm.expectEmit();
//         // emit WorkerRegistry.WorkerRegistered(WorkerId.wrap(0));
//         WorkerId workerId1 = reg.registerWorker(AUTHORIZER, "abc123");
//         require(address(reg.getAuthorizer(workerId1)) == AUTHORIZER, "create1 failed");

//         vm.expectRevert("unlucky");
//         reg.registerWorker(AUTHORIZER, "abc123");

//         WorkerId workerId2 = reg.registerWorker(AUTHORIZER, "xyz789");
//         require(address(reg.getAuthorizer(workerId2)) == AUTHORIZER, "create2 failed");

//         require(WorkerId.unwrap(workerId1) != WorkerId.unwrap(workerId2), "bad generation");
//     }

//     function testRegisterWorkerNotIdentityAuthorizer() public {
//         vm.expectRevert(InterfaceUnsupported.selector);
//         reg.registerWorker(address(0), "");
//     }

//     function testDeregisterWorker() public {
//         WorkerId workerId = reg.registerWorker(AUTHORIZER, "");

//         vm.prank(address(0));
//         vm.expectRevert(Unauthorized.selector);
//         reg.deregisterWorker(workerId);

//         // vm.expectEmit();
//         // emit WorkerRegistry.WorkerRegistered(workerId);
//         reg.deregisterWorker(workerId);

//         vm.expectRevert(IWorkerRegistry.NoSuchWorker.selector);
//         reg.getAuthorizer(workerId);
//     }

//     function testSetAuthorizer() public {
//         WorkerId workerId = reg.registerWorker(AUTHORIZER, "");

//         vm.prank(address(0));
//         vm.expectRevert(Unauthorized.selector);
//         reg.setAuthorier(workerId, address(AUTHORIZER));

//         address newAuthorizer = address(9999);
//         vm.mockCall(
//             newAuthorizer,
//             abi.encodeWithSelector(
//                 IERC165.supportsInterface.selector,
//                 type(IIdentityAuthorizer).interfaceId
//             ),
//             abi.encode(true)
//         );
//         vm.mockCall(
//             newAuthorizer,
//             abi.encodeWithSelector(IERC165.supportsInterface.selector, type(IERC165).interfaceId),
//             abi.encode(true)
//         );
//         reg.setAuthorier(workerId, newAuthorizer);

//         require(address(reg.getAuthorizer(workerId)) == newAuthorizer, "failed to set authorizer");
//     }

//     function testTransferRegistration() public {
//         WorkerId workerId = reg.registerWorker(AUTHORIZER, "");

//         vm.prank(address(0));
//         vm.expectRevert(Unauthorized.selector);
//         reg.proposeRegistrationTransfer(workerId, address(this));

//         reg.proposeRegistrationTransfer(workerId, address(41));
//         reg.proposeRegistrationTransfer(workerId, address(0));
//         reg.proposeRegistrationTransfer(workerId, address(42));

//         vm.prank(address(41));
//         vm.expectRevert(Unauthorized.selector);
//         reg.acceptRegistrationTransfer(workerId);

//         vm.prank(address(42));
//         reg.acceptRegistrationTransfer(workerId);

//         vm.expectRevert(Unauthorized.selector);
//         reg.proposeRegistrationTransfer(workerId, address(this));
//     }
// }
