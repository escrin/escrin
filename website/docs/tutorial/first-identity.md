---
description: "Creating and managing identities and secrets using the Escrin identity framework."
---

> "I am who IAM"  - Escrin Smart Worker

# Identity, the Genesis of Agency

Like many autonomous agents, an Escrin Smart Worker is born with no possessions of its own.
All it has is the purpose written into its code.
To accomplish its goals, the worker must bootstrap itself using the tools available in its environment.
The Escrin identity framework simplifies this bootstrapping by providing a single [permit](https://en.wikipedia.org/wiki/Capability-based_security) that can be presented to other agents & services to gain access to their valuable assets and information.
Essentially, the worker gets a passport: a trustworthy record of identity that entitles the holder to benefits like the ability to complete tasks and access wallets and APIs.
Accordingly, if DeFi is _be your own bank_, autonomous computing is _be your own government_!

## Overview

These next two tutorials will cover the Escrin identity framework.
Whereas the task framework that was explored in the previous two tutorials is concerned with *how* a task gets done, the identity framework is about *who* can do it and under *what* conditions.
Accordingly, the identity framework is the first line of defense towards data misuse: a worker cannot complete tasks whose data it cannot see!

By the end of this tutorial, you should have a better understanding of:
* why and how to create an on-chain identity
* how to use smart contracts to control access to an identity
* identity-based secret management

These concepts will be at the core of the next tutorial which will cover getting and using secrets within a Smart Worker.

## Setup

Before diving in, we need to set up a development environment.
As in the first tutorial, we will use the [Remix IDE](https://remix.ethereum.org/) for its convenient user experience, but tools like Foundry and Hardhat work just as well.

Once Remix has loaded, start by creating a new blank workspace and create two new blank files: `TrustedOwnerPermitter.sol` and `TestIdentityRegistry.sol`.
To silence compiler warnings, add the following to the top of each:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;
```

The remainder of this tutorial will be all about these two soon-to-be contracts and how to interact with them.

## The Identity Registry

An Escrin Identity Registry is the issuer of _permits_, which are roughly non-transferable NFTs that on-chain contracts and off-chain services can check to establish that the holder is trusted in some way.

In most cases, all apps on one chain will share the same Identity Registry, as the default one is secure, efficient, and convenient.
However, anyone can deploy and use their own custom Identity Registry within their own Escrin ecosystem.

Actually, since this tutorial runs on a local testnet, it is necessary to create a local Identity Registry that will be used to issue permits.
Accomplishing this is as simple as pasting the following lines into `TestIdentityRegistry.sol`.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "@escrin/evm/contracts/identity/v1/IdentityRegistry.sol"; // [!code ++:2]
contract TestIdentityRegistry is IdentityRegistry {}
```

Next, navigate to the Remix "Deployments" tab and click the big orange "Deploy" button.
You should see a new entry under "Deployed Contracts" that says something like "TESTIDENTITYREGISTRY AT 0X.."
That is your local copy of the default Escrin Identity Registry!
Indeed the default one is quite convenient.

Expanding the deployed contract's methods by clicking on the show/hide arrow icon, you should see a number of write methods like `createIdentity` and `destroyIdentity` in orange; and a few read methods like `getPermitter` and `readPermit` in blue.
The complete list can be found in the definition of [IIdentityRegistry](https://github.com/escrin/escrin/blob/main/evm/contracts/identity/v1/IIdentityRegistry.sol), but this tutorial only requires knowledge of two:

```solidity
interface IIdentityRegistry is IERC165 {
  // ...

  // Creates a new identity controlled by the specified Permitter contract.
  // @param permitter The address of the contract that grants the identity.
  // @param pers [optional] Extra entropy used to generate the identity.
  // @return The newly created identity's id (store this somewhere).
  function createIdentity(address permitter, bytes calldata pers)
    external
    returns (IdentityId id);

  /// Grants an identity's permit to an account. Must be called by the Permitter.
  /// @param id The id of the identity to grant.
  /// @param to The address of the permit's recipient.
  /// @param expiry The Unix timestamp at which the permit expires.
  function grantIdentity(IdentityId id, address to, uint64 expiry) external;

  // ...
}
```

`createIdentity` is called by you, and a `grantIdentity` call is initiated by whichever service wants to act as the identity–usually the Escrin Worker.

It is not possible yet to create an identity because we do not yet have a Permitter contract that decides whether to grant or revoke it.
Instead, what we can do is set the goal for the rest of this tutorial to be acquiring an identity for yourself.
Therefore, it should come as no surprise that the next task will be to create a Permitter!

## Gate Access using a Permitter

The [Permitter interface](https://github.com/escrin/escrin/blob/main/evm/contracts/identity/v1/IPermitter.sol) is just two methods, which are included below for your skimming pleasure:


```solidity
interface IPermitter is IERC165 {
  /// Requests that the permitter trigger the upstream identity registry to grant an identity.
  /// @param identity The identity that the requester wishes to acquire.
  /// @param requester The account to which the identity permit will be issued.
  /// @param duration The requested lifetime of the permit, which may be different from lifetime actually granted.
  /// @param context Non-authentication data provided to the permitter to make its decision.
  /// @param authorization Authentication data provided to the permitter to make its decision.
  /// @return expiry The timestamp at which the permit expires, which may be different from the request timestamp plus the requested duration.
  function acquireIdentity(
    IdentityId identity,
    address requester,
    uint64 duration,
    bytes calldata context,
    bytes calldata authorization
  ) external returns (uint64 expiry);

  /// Requests that the permitter trigger the upstream identity registry to revoke an identity.
  /// @param identity The identity that the requester wishes to acquire.
  /// @param possessor The account that will no longer have the permit.
  /// @param context Non-authentication data provided to the permitter to make its decision.
  /// @param authorization Authentication data provided to the permitter to make its decision.
  function releaseIdentity(
    IdentityId identity,
    address possessor,
    bytes calldata context,
    bytes calldata authorization
  ) external;
}
```

A Permitter really is just a gate: access is provided to any caller that can pass through either method.
If the Permitter is the one registered with the Identity Registry for the requested identity, a successful call to `acquireIdentity` or `releaseIdentity` ends up granting or revoking the identity permit to/from the requester.
There might be several layers of gates, though, and calls into the Identity Registry for a particular identity must be from the identity's registered permitter.
For examples, assuming `Permitter 2` is the one associated with the requested identity:

```
✅ EOA (you) → Permitter 1 → Permitter 2 → Identity Registry
✅ EOA (you) → Permitter 2 → Identity Registry
❌ EOA (you) → Permitter 1 → Identity Registry
```

This chained setup makes it easy to extend Permitters through composition.
`Permitter 2` in the diagram above can be said to delegate to `Permitter 1`, but without the hassle of `DELEGATECALL` or proxies.
This is useful when creating complex custom Permitters.

The main takeaway is that any call chain ending in `IdentityRegistry.grantIdentity(id)` must have the permitter for that identity as its second-to-last hop.

### A Simple First Permitter

The mechanics of setting up the call chain are streamlined by the pre-made [Permitter base contract](https://github.com/escrin/escrin/blob/main/evm/contracts/identity/v1/permitters/Permitter.sol.)

For this tutorial, we will create a simple Permitter that allows the account that owns the Permitter to grant and revoke the identity.
We will go step-by-step to fully understand the important process of permitting, though in most cases a [pre-made Permitter](https://github.com/escrin/escrin/blob/main/evm/contracts/identity/v1/permitters/) should suffice.

Start by heading back to the Remix files tab and opening (the aptly named) `TrustedOwnerPermitter.sol`.

Drop in the following lines to set up the contract and its base contract.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "@escrin/evm/contracts/identity/v1/permitters/Permitter.sol"; // [!code ++:2]
import "@openzeppelin/contracts/access/Ownable.sol";

contract TrustedOwnerPermitter is Ownable, Permitter { // [!code ++:2]
}
```

The Solidity compiler should have started complaining about unimplemented methods from the [abstract base Permitter contract](https://github.com/escrin/escrin/blob/main/evm/contracts/identity/v1/permitters/Permitter.sol), so quickly add these to banish those stressful red underlines so that we can leisurely go over what each component does.

```solidity
contract TrustedOwnerPermitter is Ownable, Permitter {
  constructor(IIdentityRegistry registry) Ownable(msg.sender) Permitter(registry) {} // [!code ++]

  /// Authorizes the identity acquisition request, returning the expiry if approved or reverting if denied. // [!code ++:9]
  function _acquireIdentity(
    IdentityId /* identity */,
    address /* requester */,
    uint64 duration,
    bytes calldata /* context */,
    bytes calldata /* authorization */
  ) internal override returns (uint64 expiry) {
  }

  /// Authorizes the identity release request, reverting if denied. // [!code ++:7]
  function _releaseIdentity(
    IdentityId /* identity */,
    address /* requester */,
    bytes calldata /* context */,
    bytes calldata /* authorization */
  ) internal override {}
}
```

Along with setting the initial (trusted) owner to deployer, this `constructor` sets up the Permitter base contract with the address of the upstream Identity Registry since every permitter (or its upstream) needs to know either the address of the Identity Registry where the identity is registered.

The Permitter base contract automatically calls the Identity Registry when the `_acquireIdentity` or `_releaseIentity` lifecycle hook returns without reverting.
Each receives contextual information as arguments that can be forwarded to the next Permitter in the chain or used to directly authorize the request.
All a Permitter needs to do is revert if called in the wrong context!
Here is how we can make that happen for a trusted owner.

```solidity
contract TrustedOwnerPermitter is Ownable, Permitter {
  constructor(IIdentityRegistry registry) Ownable(msg.sender) Permitter(registry) {}

  /// Authorizes the identity acquisition request, returning the expiry if approved or reverting if denied. // [!code focus:10]
  function _acquireIdentity(
    IdentityId /* identity */,
    address /* requester */,
    uint64 duration,
    bytes calldata /* context */,
    bytes calldata /* authorization */
  ) internal override returns (uint64 expiry) { // [!code --]
  ) internal view override onlyOwner returns (uint64 expiry) { // [!code ++]
  }

  /// Authorizes the identity release request, reverting if denied. // [!code focus:8]
  function _releaseIdentity(
    IdentityId /* identity */,
    address /* requester */,
    bytes calldata /* context */,
    bytes calldata /* authorization */
  ) internal override {} // [!code --]
  ) internal view override onlyOwner {} // [!code ++]
}
```

Now it will be impossible for any caller other than the owner of the Permitter to get past the gate.

To wrap things up, set the expiry that the Permitter will send to the Identity Registry.
Note that the `_acquireIdentity` method takes a `duration` parameter.
This duration is what the requester _wants_, but the permitter can grant a longer or shorter duration (or revert) as it prefers.
Since you trust yourself, it is simple enough to have whatever duration you want:

```solidity
  function _acquireIdentity( // [!code focus]
    IdentityId /* identity */,
    address /* requester */,
    uint64 duration, // [!code focus]
    uint64 duration,
    bytes calldata /* context */,
    bytes calldata /* authorization */
  ) internal override onlyOwner returns (uint64 expiry) { // [!code focus:2]
    return uint64(block.timestamp) + duration; // [!code ++]
  }
```

Your finished `TrustedOwnerPermitter.sol` should now contain the following code.
If that is true, compile it and enjoy an absence of errors!

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "@escrin/evm/contracts/identity/v1/permitters/Permitter.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract TrustedOwnerPermitter is Ownable, Permitter {
  constructor(IIdentityRegistry registry) Ownable(msg.sender) Permitter(registry) {}

  function _acquireIdentity(
    IdentityId /* identity */,
    address /* requester */,
    uint64 duration,
    bytes calldata /* context */,
    bytes calldata /* authorization */
  ) internal view override onlyOwner returns (uint64 expiry) {
    return uint64(block.timestamp) + duration;
  }

  /// Authorizes the identity release request, reverting if denied.
  function _releaseIdentity(
    IdentityId /* identity */,
    address /* requester */,
    bytes calldata /* context */,
    bytes calldata /* authorization */
  ) internal view override onlyOwner {}
}
```

All done! It should be all downhill from here ⛷️.

### Deploying the Permitter

The `TrustedOwnerPermitter` takes the address of an Identity Registry.
Fortunately, you already created one of those!

Go back to the Deployments tab and, where it says "TESTIDENTITYREGISTRY AT", click on the copy button to get its address.

In the same tab but in the top panel, ensure that `TrustedOwnerPermitter` is selected in the dropdown list, the paste the address into the box next to the big orange Deploy button that says `address registry`.
The button should light up, which means that you should click it!
Once that succeeds, there will be a new contract at the bottom of the lower Deployed Contracts panel–your permitter.
Copy its address to enter the home stretch.

## Create an Identity

Returning again to the `createIdentity` button in the "TESTIDENTITYREGISTRY" deployed contract's interaction panel, paste the Permitter's address into the text box and then append `,0x` to represent an empty byte string.
The arguments should look similar to `0xd8b934580fcE35a11B58C6D73aDeE468a2833fa8,0x`.
Smash the orange button when it lights up; even do it a few times because identities are free and can share a permitter.

If the transaction(s) succeeded, you should see a green checkmark in the console at the bottom of the page alongside a summary of the transaction.
Click on it to expand the details, which will include fields like `status`, `transaction hash`, `decoded input`, and `decoded output`.
The decoded output is what we want and should look something like this:

```
{
	"0": "uint256: id 53122387064105297136834696072460720124149155026917914001996103119252653300344"
}
```

That long hexadecimal string is the id of the identity you just created!
Copy that into your clipboard because it's time to (drum roll, please...) acquire the identity!

## Manually Acquire the Identity

Now, you, the autonomous agent, will acquire your on-chain identity by requesting it from the Permitter you just deployed.
This would normally be done your Escrin Smart Worker, of course.

In the "TRUSTEDOWNERPERMITTER" interaction panel, paste in the contents of your clipboard, which should be your identity id, and then follow it with some text representing the remaining arguments.

```
,0x0000000000000000000000000000000000000000,86400,0x,0x
```

That should have earned you another green check mark, which means that you are now authenticated as your identity. Congratulations! 🎉

If you are feeling adventurous, you can go back to the identity registry and read the permit you have, or even release your identity.
Such power!

Anyway, that covers the mechanics of identity creation and acquisition, so the final substantive section of the tutorial will be about conceptually how identities are commonly used within Escrin and beyond.
This is to say that you can put away your development environment for now–you've earned a break.

## Doing Things With Identities

To recap, an Escrin identity is nothing more than an authenticated record of having passed some layers of checks.
Nevertheless, this is a powerful feature because if there is a secret management network that can verify the issuance of a permit and return a root secret (_OmniKey_), the holder of the permit can use the secret to derive wallets, encryption keys, TLS certificates, random numbers, and basically anything security-related that one could possibly want.

Take, for example, the [NFTrout Worker](https://github.com/escrin/nftrout/blob/main/worker/src/main.ts#L15-L22).
When it starts up, the worker acquires the global NFTrout identity, and uses it to obtain an _OmniKey_ from the Oasis Sapphire secret management network.
The newly-minted NFTrout Worker then uses its identity's associated OmniKey to encrypt/decrypt virtual fish genes such that it can interoperate with any other worker in the world having the same identity.
The ability for multiple workers to acquire an identity is the basis for decentralization in Escrin.

Other foods for thought are acquiring an identity and presenting it to a Web2 API that verifies it against the blockchain and returns some secret data.
Alternatively, the OmniKey can be used to derive an autonomous wallet that can submit transactions on behalf of the autonomous service and fund its own existence.
The possibilities are truly endless, and now you should have a solid foundation for designing your own autonomous worker identities and access control schemes.

## Next Steps

Paralleling the task framework pair of tutorials, the next tutorial will be on how to use identities and their secrets programmatically using an Escrin Smart Worker.

Congratulations again learning how Escrin identities work!
After completing [one more tutorial](./secret-worker) you will have been equipped with all of the skills needed to create your own autonomous computing workflows! 🎉
