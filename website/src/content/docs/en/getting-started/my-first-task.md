---
title: "My First Task"
description: "A crash course on how to use Escrin to run secure off-chain tasks."
---

## Objective

This guide will walk you through assembling an `AddingAtHome` dapp that pays people for contributing valid additions to the NumbersDAO.
In this illustrative example, adding two numbers is done as an Escrin task due to the intense and high-security calculation required.
The result is posted back to the blockchain as a part of the NumbersDAO mission to create all of the numbers.

The key concepts covered are:
* installing the Escrin toolkit
* creating tasks
* accepting task results
* submitting task results

## On-Chain Steps

Escrin allows smart contracts to run secure off-chain computation, which are called _tasks_.
Tasks are created implicitly by a contract when it provides an incentive for off-chain task runners to submit work.
To allow your contract to run tasks, you will need to use the Escrin Solidity library to create and accept tasks.
The steps below work through creating tasks that will be completed by task runners created in the next set of steps.

### 1. Create a new Remix project

<!-- TODO: make a tip callout box thing for this meta-exposition -->
This guide uses [Remix] because it is convenient for prototyping.
The steps should be easy to follow if you are using a different development environment like Hardhat or Foundry.

Start by navigating to [Remix].
Once there, create a new workspace from the "OpenZeppelin ERC721" template.
You should see a file browser containing `contracts/MyToken.sol`.
Open it for editing.

Rename the contract, and optionally also the source file.

```diff
 // SPDX-License-Identifier: MIT
 pragma solidity ^0.8.9;

 import "@openzeppelin/contracts/token/ERC721/ERC721.sol";

-contract MyToken is ERC721 {
+contract AddingAtHome is ERC721 {
     constructor() ERC721("Number", "NUM") {}
 }
```

[Remix]: https://remix.ethereum.org

### 2. Get the Escrin Solidity library

Escrin is easy to use within any dapp.
All you have to do is implement [ITaskAcceptor], which is all of one function.

The Escrin Solidity library contains a number of pre-made task acceptors and widgets that you can use to build your own.

The task acceptance criterion for `AddingAtHome` is whether the result is the sum of the two inputs.
This is simple to verify, so the `AddingAtHome` contract will inherit from the [TaskAcceptorV1] base class.
`TaskAcceptorV1` implements basic validation and exposes lifecycle hooks, so it is a good place to start.

Here is how you can change your code to make `AddingAtHome` a task acceptor:

```diff
+import {TaskAcceptorV1} from "@escrin/evm/contracts/tasks/acceptor/TaskAcceptor.sol";
+import {TaskHubNotifier} from "@escrin/evm/contracts/tasks/widgets/TaskHubNotifier.sol";
 import "@openzeppelin/contracts/token/ERC721/ERC721.sol";

-contract AddingAtHome {
+contract AddingAtHome is TaskAcceptorV1, TaskHubNotifier {
     constructor() ERC721("AddingAtHome", "SUM") {}
 }
```

This changeset also adds the completely optional `TaskHub`, which is just a notification center that lets a worker pool know that tasks are available for completion.
`AddingAtHome` is a public service, so it announces itself to the world.

This code does not yet compile because `AddingAtHome` does not yet implement the `_acceptTaskResults` function stub left by `TaskAcceptorV1`.
We will add that in the next step.

[ITaskAcceptor]: https://github.com/escrin/escrin/blob/main/evm/contracts/tasks/acceptor/ITaskAcceptor.sol
[TaskAcceptorV1]: https://github.com/escrin/escrin/blob/main/evm/contracts/tasks/acceptor/TaskAcceptor.sol

### 3. Accept tasks

Task acceptance is an important decision because the contract virtually creates whatever tasks it accepts.
Task acceptance is also a highly personal decision, so you have all the power of a function to express it.

For `AddingAtHome`, a task is to discover a target number by providing two already-discovered numbers that sum to that number.
The submitter will be rewarded with an NFT representing discovery of the new number.
A natural choice is to make the NFT token ID be the number that was discovered.

In that case, first add some code that allows the discovery process to begin.
Also let task runners know that this contract has tasks available.

```diff
 contract AddingAtHome is ERC721 {
-    constructor() ERC721("AddingAtHome", "SUM") {}
+    constructor() ERC721("AddingAtHome", "SUM") {}
+        _mint(msg.sender, 1);
+        taskHub().notify();
+    }
 }
```

Now the deploying account can discover new numbers like 2 (please don't steal my work).

Next, the contract needs to define the verification function that ensures that the numbers have the right sum.
The following code is the empty sub left to implement by [TaskAcceptorV1].
Copy it into Remix for later.

```diff
 contract AddingAtHome is ERC721 {
     constructor() ERC721("AddingAtHome", "SUM") {
      _mint(msg.sender, 1);
     }
+
+    /// Accepts one or more elements of a task runner's task results submission, returning the seto tasks that were accepted.
+    /// @param _taskIds A sorted set of taskIds completed in this submission
+    /// @param _proof Some proof of having completed the identiied tasks that the acceptor can verify.
+    /// @param _report Some data provided by the submitter that the requester may or may not trust
+    /// @param _submitter The account that submitted the task results
+    function _acceptTaskResults(
+        uint256[] calldata _taskIds,
+        bytes calldata _proof,
+        bytes calldata _report,
+        address _submitter
+   ) internal override returns (TaskIdSelector memory sel) {
+   }
 }
```

Our objective is to express our task validation logic in terms of task ids, proofs, and reports.
All of `_taskIds`, `_proof`, and `_report` are entirely generated by the submitter and can have any format.
The format, of course, being what the acceptor will choose to accept.

In this case, `_tasksIds` can be used identify the numbers to be discovered.
The `_proof` is the right place to put the two numbers that add up, but they need to be decoded from bytes using `abi.decode`.
`_submitter` and `_report` are not used.

The following diff changes the argument names for clarity and adds a statement to unzip `_proof`.

```diff
     function _acceptTaskResults(
-        uint256[] calldata _taskIds,
-        bytes calldata _proof,
-        bytes calldata _report,
-        address _submitter
+        uint256[] calldata _numbers,
+        bytes calldata _zippedSummands,
+        bytes calldata,
+        address
    ) internal override returns (TaskIdSelector memory sel) {
+     (uint256[] memory lefts, uint256[] memory rights) =
+         abi.decode(_zippedSummands, (uint256[], uint256[]));
    }
```

The last component of the `_acceptTaskResults` function is the actual validation.
This means going through each of the tasks and ensuring that the target number is undiscovered and that the two numbers provided as proof add up correctly.

```diff
     function _acceptTaskResults(
         uint256[] calldata _numbers,
         bytes calldata _zippedSummands,
         bytes calldata,
         address
    ) internal override returns (TaskIdSelector memory sel) {
      (uint256[] memory lefts, uint256[] memory) = abi.decode(_proof, (uint256[], uint256[]));
+     for (uint256 i; i < _tasksIds.length; ++i) {
+       require(_exists(_lefts[i]) && _exists(_rights[i]), "unknown summand");
+       require(!_exists(_numbers[i]), "already discovered");
+       require(lefts[i] + rights[i] == _numbers[i], "invalid sum");
+     }
+     sel.quantifier = Quantifier.All;
    }
```

That last line that sets the selection's `Quantifier` is just reporting that the entire task was accepted.
Usually the acceptor will report `Quantifier.All` but revert on error, but if you need partial acceptance, that's also possible.

### 4. Provide incentives

Unless you have a private deployment of task runners, your task isn't going to get run unless the runners get some value out of it.

`AddingAtHome` is for the benefit of all humankind, so an acknowledgement of discovering a number should be reward enough.
The submitter will get their number as an NFT, which is just a call to `_mint` using OpenZeppelin's ERC-721.

This logic will go in `_afterTaskResultsAccepted`, which is another lifecycle hook exposed by [TaskAcceptorV1].
Rewarding the submitter could have equally gone in `_acceptTaskResults`, but this way the separation of concerns is clearer.

```diff
+    function _afterTaskResultsAccepted(
+        uint256[] calldata _numbers,
+        bytes calldata,
+        address _submitter,
+        TaskIdSelector memory
+    ) internal override {
+        for (uint256 i; i < _numbers.length; ++i) {
+            _mint(_submitter, _numbers[i]);
+        }
+    }
```

Good!
The on-chain steps are complete.
All that remains is to create a program that task runners will run to complete the task.

## Off-Chain Steps

### 1. Run the tasks

Right now, there's no public pool of Escrin task runners, so you have to complete the tasks yourself.
This can be done using a library like Ethers.js because the task acceptance criteria are so lax.
In reality, a submission would be accepted if it came from an attested trusted execution environment running a particular program plus other things.

If you find this page exciting and want to get started building, drop into the [#escrin channel of our Discord](https://discord.gg/QjNpXkd3un) to ask any questions that you may have!

<!-- add next steps section once there are any -->
