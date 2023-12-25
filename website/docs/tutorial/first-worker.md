---
description: "Using Escrin Smart workers to securely complete tasks using off-chain computation"
outline: deep
---

# My First Smart Worker

The [previous tutorial](././first-task.md) described how to create a simple on-chain `TaskAcceptor` that rewards off-chain workers for "discovering" new numbers.
In this `AddingAtHome` contract, each number is represented by an NFT having the same `tokenId` as the number.
A new number may be discovered by sending two already-discovered numbers to the contract.

Completing these tasks manually using the Remix UI is not very fun, so this tutorial covers automating number discovery using an Escrin Smart Worker.

After finishing this tutorial, you should feel comfortable creating Escrin Smart Workers using the `@escrin/worker` JavaScript library and the `escrin-runner`.

## Setup

Start by creating a new workspace for your project (e.g., using `mkdir`).
Once that exists, run one of the following sets of commands to set up the workspace containing the required dependencies.

::: code-group

```sh [pnpm]
pnpm init
pnpm install @escrin/worker ethers
pnpm install --dev esbuild
```

```sh [npm]
npm init --yes
npm install @escrin/worker ethers
npm install --save-dev esbuild
```

```sh [yarn]
yarn init --yes
yarn install @escrin/worker ethers
yarn install --save-dev esbuild
```

:::

| Dependency     |  Purpose |
| -------------  | -------- |
| @escrin/worker | Types and utilities that make developing Smart Workers more convenient |
| ethers         | Ethereum library that enables interaction with the `AddingAtHome` contract. This dependency may be removed in a future release. |
| esbuild        | Bundles code and dependencies into a single file, as required by the `escrin-runner`. |

Once the package manager has finished installing the dependencies, create a new file called `worker.js`, which will be the focus of the rest of the tutorial.

## Building the Smart Worker

### Scaffolding

Start by opening `worker.js` and adding following lines of code.

```javascript
// worker.js
import escrinWorker from '@escrin/worker';
import { ethers } from 'ethers';

export escrinWorker(new class {
    async tasks(rnr) {
    }
});
```

This code uses the [`escrinWorker`](https://github.com/escrin/escrin/blob/main/runner/src/index.ts) function to set up callbacks into the handlers object passed as its argument.
The handlers that will be invoked depend on the worker's configuration and which event triggers to which it has been set up to respond.

In this tutorial, the worker will run on a (cron) schedule. When the scheduled time has arrived, the `tasks` handler will be called to notify the worker that there may be new tasks available.

The `rnr` argument of the `tasks` callback is a reference to to the `EscrinRunner`, a binding to the `escrin-runner` platform that provides secure and convenient access to commonly used functions like configuration, communication, and key management.
The complete set of bindings can be found in the [Smart Worker Reference](/docs/reference/worker.md).

### Connecting to the Contract

```javascript
import escrinWorker from '@escrin/worker';
import { ethers } from 'ethers';

export escrinWorker(new class {
    /// An `ethers.Contract` backed by the `AddingAtHome` contract. // [!code ++:3]
    #contract;

    async tasks(rnr) {
        if (!this.#contract) { // [!code ++:3]
            this.#contract = this.#makeContract(await rnr.getConfig());
        }
    }
    // [!code ++:16]
    #makeContract(config) {
        const runner = new ethers.Wallet(
            config.walletKey,
            new ethers.JsonRpcProvider(config.web3GatewayUrl),
        );
        return ethers.Contract(
            config.contractAddress,
            [
                'function acceptTaskResults(uint256[], bytes, bytes, address)',
                'function totalSupply() view returns (uint256)',
                'function tokenByIndex(uint256) view returns (uint256)',
            ],
            runner,
        );
    }
});
```

### Detecting Tasks

A core concept in Escrin is that a `TaskAcceptor` contract does not need to know which tasks it has available.
It is the responsibility of the Smart Worker to figure this out.
This setup is gas optimal because it moves detection off-chain where computation and storage are inexpensive.

For the `AddingAtHome` example, although there are many different ways in which to discover a number, to keep this tutorial simple, the worker will simply add one to the largest discovered number, and only one number will be discovered per wakeup.

Accordingly, the worker only needs to keep track of the total supply and the maximum discovered number.
To get this information, the worker will use the `Contract` object in the usual way.

```javascript
export escrinWorker(new class {
    /// An `ethers.Contract` backed by the `AddingAtHome` contract.
    #contract;
    /// The latest known total supply of the token. // [!code ++:4]
    #latestSupply = 0n;
    /// The maximum discovered number so far.
    #maxDiscovered = 0n;

    async tasks(rnr) {
        if (!this.#contract) {
            this.#contract = this.#makeContract(await rnr.getConfig());
        }
        // [!code ++:2]
        await this.#syncTasks();
    }
    // [!code ++:11]
    async #syncTasks() {
        const latestSupply = await this.#contract.totalSupply();
        for (let i = this.#latestSupply; i <= latestSupply; i++) {
            const discovery = await this.#contract.tokenByIndex(i);
            if (discovery > this.#maxDiscovered) {
                this.#maxDiscovered = discovery;
            }
        }
        this.#latestSupply = latestSupply;
    }

    #makeContract(config) {
```

### Submitting Task Results

Now, with the tasks discovered, all that is left to be done is submit the tasks to the contract.
This is done in the same way as it was before, just now with the help of code.

Since only one number will be discovered, only one task id is submitted–the same one as the number.
The proof remains as the two addends, but EthABI encoded so that they can be passed as `bytes` to the `TaskAcceptor` contract.

Because someone might have discovered the same number at the same time, the `#discover` function is wrapped in a `try..catch` block.

```javascript
export escrinWorker(new class {
    /// An `ethers.Contract` backed by the `AddingAtHome` contract.
    #contract;
    /// The latest known total supply of the token.
    #latestSupply = 0n;
    /// The maximum discovered number so far.
    #maxDiscovered = 0n;

    async tasks(rnr) {
        if (!this.#contract) {
            this.#contract = this.#makeContract(await rnr.getConfig());
        }

        await this.#syncTasks();
        // [!code ++:7]
        try {
            const discovery = await this.#discover();
            console.log('discovered', discovery);
        } catch (e) {
            console.error('failed to discover number:', e);
        }
    }
    // [!code ++:8]
    async #discover() {
        const newDiscovery = this.#maxDiscovered + 1n;
        const coder = ethers.AbiCoder.defaultAbiCoder();
        const proof = coder.encode(['uint256[]'], [[1n, this.#maxDiscovered]]);
        await this.#contract.acceptTaskResults([newDiscovery], proof, new Uint8Array());
        return newDiscovery;
    }

    async #syncTasks() {
```

Putting it all together, the final `worker.js` file should look like the following.
If that is true, then it is time for deployment!

Overally, you should have found that writing an Escrin Smart Worker is not much different from any other dapp.

```javascript
export escrinWorker(new class {
    /// An `ethers.Contract` backed by the `AddingAtHome` contract.
    #contract;
    /// The latest known total supply of the token.
    #latestSupply = 0n;
    /// The maximum discovered number so far.
    #maxDiscovered = 0n;

    async tasks(rnr) {
        if (!this.#contract) {
            this.#contract = this.#makeContract(await rnr.getConfig());
        }

        await this.#syncTasks();

        try {
            const discovery = await this.#discover();
            console.log('discovered', discovery);
        } catch (e) {
            console.error('failed to discover number:', e);
        }
    }

    async #discover() {
        const newDiscovery = this.#maxDiscovered + 1n;
        const coder = ethers.AbiCoder.defaultAbiCoder();
        const proof = coder.encode(['uint256[]'], [[1n, this.#maxDiscovered]]);
        await this.#contract.acceptTaskResults([newDiscovery], proof, new Uint8Array());
        return newDiscovery;
    }

    async #syncTasks() {
        const latestSupply = await this.#contract.totalSupply();
        for (let i = this.#latestSupply; i <= latestSupply; i++) {
            const discovery = await this.#contract.tokenByIndex(i);
            if (discovery > this.#maxDiscovered) {
                this.#maxDiscovered = discovery;
            }
        }
        this.#latestSupply = latestSupply;
    }

    #makeContract(config) {
        const runner = new ethers.Wallet(
            config.walletKey,
            new ethers.JsonRpcProvider(config.web3GatewayUrl),
        );
        return ethers.Contract(
            config.contractAddress,
            [
                'function acceptTaskResults(uint256[], bytes, bytes, address)',
                'function totalSupply() view returns (uint256)',
                'function tokenByIndex(uint256) view returns (uint256)',
            ],
            runner,
        );
    }
});
```

## Bundling & Deploying

### Bundling

In order to be run by an `escrin-runner`, the worker code must be bundled into a single file.
Bundling does not affect the behavior of your code or its imports–it just makes all of the code available for the `escrin-runner` to run.
This is no different from creating a [regular JavaScript Worker](https://developer.mozilla.org/en-US/docs/Web/API/Worker).

For bundling, this tutorial uses `esbuild`.
Run the following code to create a `bundled-worker.js` that is ready for deployment.

```sh
npx --no-install esbuild \
    --bundle ./worker.js --outfile=./bundled-worker.js \
    --target=es2022 --format=esm --minify
```

In this case, we chose to create an ES module bundle using `--format=esm` since that is the modern format that offers greater flexibility and features.

The `--target=es2022` and `--minify` flags are used to decrease bundle size and increase performance since less code is required.

### Deployment

First, create a JSON file called `worker-config.json` containing the configuration variables `contractAddress` and `walletKey`.

::: code-group

```json [Sapphire Testnet]
{
    "web3GatewayUrl": "https://testnet.sapphire.oasis.dev",
    "contractAddress": "0x4046d9265f3a2E9b0Ba8EE61A1a8bC8093CEfd53",
    "walletKey": "0xf9834a328ff8f2599724e689e24b3585fb4e3b0a4ab84effe1d74ae9c7ce9fff"
}
```

```json [Local]
{
    "web3GatewayUrl": "http://localhost:8545",
    "contractAddress": "0x...",
    "walletKey": "0x..."
}
```

:::

Next, and finally, send the bundled worker to the `escrin-runner` instance using the HTTP API.

::: code-group

```sh [Hosted]
curl -isS https://demo.escrin.org \
    -F 'script=@bundled-worker.js' \
    -F 'type=module' \
    -F 'schedule="*/5 * * * *"' \
    -F 'config=@worker-config.json'
```

```sh [Local]
curl -isS http://127.0.0.1:1057 \
    -F 'script=@bundled-worker.js' \
    -F 'type=module' \
    -F 'schedule="*/5 * * * *"' \
    -F 'config=@worker-config.json'
```

:::

You should see something like the following response, which means that your worker has been successfully submitted!

```http
HTTP/1.1 201 Created
Content-Length: 45
Content-Type: application/json

{"id":"36ed31e6-58a8-4025-918e-afe66af78896"}
```

::: tip
There is currently no way to check the status of the worker, but this feature is planned for an upcoming release.
For now, the only way to observe progress is to see if tasks are being completed.

When running the `escrin-runner` locally, it is possible to debug the worker using standard tools like `console.log` (and, also soon-to-come, the Chrome DevTools).
:::

After a few moments, you should find that a few numbers have already been discovered.
If you are using the Sapphire Testnet config JSON, you can see the results live on the [Oasis Explorer](https://explorer.oasis.io/testnet/sapphire/address/0x4046d9265f3a2E9b0Ba8EE61A1a8bC8093CEfd53/token-transfers#transfers).

Now you have not only completed several Escrin tasks, you have completed the tutorial.
Congratulations! 🎉

## Recap & Next Steps

In this tutorial we created an Escrin Smart Worker that completes on-chain tasks created using the Escrin Solidity library.
We even were able to complete a few tasks without manual labor!

Even though `AddingAtHome` remains a simple problem, it illustrates the idea of a Smart Worker, and how easy it is to write JavaScript that runs in a secure environment to autonomously complete tasks according to the designs of a smart contr'act.

In a more realistic scenario, the Smart Worker would coordinate private access to private data after authenticating itself to a key management contract via a technique like remote attestation.
These features are provided by the Escrin Runner, so it is a mostly simple matter of configuration to take advantage of these powerful features.

Having completed the two foundational tutorials on creating and fulfilling tasks using Escrin, you should feel comfortable getting started with Escrin in your own application.

The next tutorial will cover the more advanced security & privacy features that most will want to take advantage of.

