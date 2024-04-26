---
description: "Using Escrin Smart Workers to handle secrets with trust and integrity"
---

# Trusting Secrets to Workers

Trust is the most convenient force in the universe.
Trust allows us to ignore all of the details of *how* an agent does something and focus just on *what* it does.
Trust erases the implementation details, leaving only the interface: if you make a request, you will get the expected response.
So easy!

Ideally, trust would be universal and everything could be smooth and simple, but the difficulty is always in establishing trust in the first place.
Fortunately, through the [identity framework](./first-identity) and the use of [trusted execution environments (TEEs)](https://en.wikipedia.org/wiki/Trusted_execution_environment), Escrin makes establishing trust in Smart Workers easy.
Escrin is largely a framework for establishing programmatic trust.

## Overview

This tutorial builds on the [previous one](./first-identity) on the identity framework by demonstrating how to use a Smart Worker to acquire an identity and its root secret, its _OmniKey_, in the context of an app that can encrypt NFT data and post it on IPFS using a private API key.
Like a pithy [NFTrout](https://nftrout.com).

You can follow along with this tutorial using a [locally deployed `escrin-runner`](../reference/runner) and EVM-compatible testnet like [anvil](https://book.getfoundry.sh/reference/anvil/) or ideally the [sapphire-dev](https://docs.oasis.io/dapp/sapphire/guide#running-a-private-oasis-network-locally) image.

By the end of this tutorial, you should be familiar with the following concepts and accordingly understand all of the core concept of Escrin!

* acquiring an identity in an Escrin Smart Worker
* the OmniKey and how to fetch it in a Smart Workers
* deriving secrets from the OmniKey

## Setup

### Package & Dependencies

Start by creating a new workspace for your project.
Once that exists, run one of the following sets of commands (just like in the [first Worker tutorial](./first-worker)):

::: code-group

```sh [pnpm]
pnpm init
pnpm install @escrin/worker nft.storage
pnpm install --dev esbuild
```

```sh [npm]
npm init --yes
npm install @escrin/worker nft.storage
npm install --save-dev esbuild
```

```sh [yarn]
yarn init --yes
yarn install @escrin/worker nft.storage
yarn install --save-dev esbuild
```

:::

| Dependency     |  Purpose |
| -------------  | -------- |
| @escrin/worker | The Escrin Smart Workers SDK |
| nft.storage    | NFT pinning service client library |
| esbuild        | ECMAScript (JS) code bundler |

Next, create `worker.js` to hold the code for this tutorial.

Additionally, create `config.json` to hold the worker's configuration and populate it with placeholders:

```json
{
   "nftStorageToken": "",
   "identity": {
   },
   "network": {
   }
}
```

### NFT.storage API key

To upload and pin NFTs using the NFT.storage API, you will need an API key, which can be obtained for free by following these steps:

1. Log into the their [website](https://nft.storage/)
2. Click on the "API Keys" link at the top of the page
3. Click "+ New Key"
4. Name the key "Escrin Demo" (or really anything; it's for your reference alone)
5. Click the "Create" button
6. Copy the key (it looks like a [JWT](https://jwt.io) because it is a JWT)
7. Paste the key into `config.js` like so
   ```json
   // config.json
   {
       "nftStorageToken": "", // [!code --]
       "nftStorageToken": "<paste>", // [!code ++]
       "identity": {
       },
       "network": {
       }
   }
   ```

### Identity Registry & Identity

Use your preferred Ethereum development toolkit like Remix to deploy a local [`OmniKeyStore`](https://github.com/escrin/escrin/blob/main/evm/contracts/identity/v1/OmniKeyStore.sol) (a special kind of `IdentityRegistry`) and create an identity by following the [previous tutorial](./first-identity), but using this `Permitter` to minimize extraneous complexity:

::: warning IMPORTANT

Ensure that you deploy the `OmniKeyStore` and not the base `IdentityRegistry`.
The `OmniKeyStore`, when run on a private EVM, securely holds both identity permits and keys in one secure, convenient location.
Deploy it just like a base `IdentityRegistry`.

:::

```solidity
// EveryonePermitter.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "@escrin/evm/contracts/identity/v1/permitters/Permitter.sol";

contract EveryonePermitter is Permitter {
  constructor(IIdentityRegistry registry) Permitter(registry) {}

  function _acquireIdentity(
    IdentityId, address, uint64, bytes calldata, bytes calldata
  ) internal view override returns (uint64) {
    return type(uint64).max;
  }

  function _releaseIdentity(
    IdentityId, address, bytes calldata, bytes calldata
  ) internal view override {}
}
```

Once you have your identity id, you can finish filling in `config.json`:

```json [Local]
// config.json
{
   "nftStorageToken": "eyJhbGciOiJ...",
   "identity": {
     "registry": "<paste registry address>", // [!code ++:2]
     "id": "<paste identity id>"
   },
   "network": {
      "chainId": 31337, // [!code ++:2]
      "rpcUrl": "http://127.0.0.1:8545"
   }
}
```

### Worker Boilerplate

The context of this next code blob should not come as a surprise (if it does, read [this tutorial](./first-worker) and come back), so just paste into `worker.js` and continue on your merry way.

```javascript
import escrinWorker from '@escrin/worker';

export escrinWorker(new class {
    async tasks(rnr) {
        const { nftStorageToken, identity, network } = await rnr.getConfig();
    }
});
```

## Identity Acquisition

Acquiring an identity takes just one call to the `acquireIdentity` function provided to the worker by the environment.
Once called, the `escrin-runner` will use the parameters to acquire an identity on behalf of the worker so that future calls to other environmental functions will use this identity when needed.

In this case, because our `Permitter` is very simple, all that the runner needs to acquire the identity its fully qualified identifier: network → registry → id.

```javascript
import escrinWorker from '@escrin/worker';

export escrinWorker(new class {
    async tasks(rnr) {
        const { nftStorageToken, identity, network } = await rnr.getConfig();
        // [!code ++:2]
        await rnr.acquireIdentity({ identity, network });
    }
});
```

There are a few other options for configuring the identity acquisition request, the full list of which can be found in [the reference](../reference/worker.html#identity), but the most important ones are just these four:

```typescript
export type AcquireIdentityParams = {
  /** The network where the identity is registered. */
  network: Network;
  /** A pointer to the identity to acquire. */
  identity: Identity;

  /** Bytes sent directly to the Permitter as the `context` parameter. */
  context?: Hex;
  /** Bytes sent directly to the Permitter's `authorization` parameter. */
  authorization?: Hex;
};
```

You have already seen the first two, `network` and `identity`.
The second two, `context` and `authorization`, are used by the worker to communicate directly with the [`Permitter`](https://github.com/escrin/escrin/blob/main/evm/contracts/identity/v1/IPermitter.sol).
These can be whatever the `Permitter` and worker agree upon.

In many cases `context` and `authorization` will represent a Trusted Platform Module (TPM) [attestation](https://en.wikipedia.org/wiki/Trusted_Computing#Remote_attestation) that proves that the worker is running in a genuine TEE.
In that case, all one must do is make an extra call to `rnr.getAttestation` like so:

::: info

The following snippet will only work if the `escrin-runner` is running in an [AWS Nitro Enclave](https://aws.amazon.com/ec2/nitro/nitro-enclaves/) and the `Permitter` is a [`NitroEnclavePermitter`](https://github.com/escrin/escrin/blob/main/evm/contracts/identity/v1/permitters/NitroEnclavePermitter.sol).

:::

```typescript
const config = { network, identity };
const { context, authorization, } = await rnr.getAttestation(config);
await rnr.acquireIdentity({ ...config, context, authorization }); // so easy!
```

## Getting the OmniKey

Having an identity permit is generally useful, but its _most_ useful function is enabling access to the identity's OmniKey.


The OmniKey is the root secret from which all other secrets like encryption and signing can be derived.
One of the more interesting examples is using the OmniKey to derive a wallet that the worker uses to fund its own existence and achieve true autonomy.

For applications both complex and simple, getting an OmniKey once a permit is held is very simple, and made simpler when the identity permit and OmniKey are on the same network, as is true when using the Oasis Sapphire confidential EVM (or the local version that you just set up).
Experience the convenience by transcribing the following into your `worker.js`:

```javascript
import escrinWorker from '@escrin/worker';

export escrinWorker(new class {
    async tasks(rnr) {
        const { nftStorageToken, identity, network } = await rnr.getConfig();

        await rnr.acquireIdentity({ identity, network });
        const omniKey = await rnr.getOmniKey({ identity, network }); // [!code ++]
    }
});
```

Really, it is that simple.
The `rnr.getOmniKey` environment function instructs the runner to communicate with the key management network you have configured–in this case the `OmniKeyStore`-flavored `IdentityRegistry`–and fetch the secret.

## OmniKey Key Derivation

After all of this hype about the amazing things that can be done with an OmniKey, the time has come to see it in action!

Since our goal is to encrypt and decrypt IPFS data, we will need an encryption key.
The OmniKey is not an encryption key, but it can be used to make one!
To accomplish this, we will use the standard [WebCrypto API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Crypto_API), specifically [`crypto.subtle.deriveKey`](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/deriveKey) and [`crypto.subtle.encrypt`](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/encrypt).

Modify your `worker.js` according to the following diff.
Do not fret if this code does not make sense; all you need to go by is the interface.
In fact, `Cipher` implements sane defaults, so it is safe to copy into your own projects.

```javascript
import escrinWorker from '@escrin/worker';

export escrinWorker(new class {
    async tasks(rnr) {
        const { nftStorageToken, identity, network } = await rnr.getConfig();

        await rnr.acquireIdentity({ identity, network });
        const omniKey = await rnr.getOmniKey({ identity, network });
        // [!code ++:2]
        const cipher = await Cipher.create(omniKey, 'escrin-demo/encryption');
    }
});
// [!code ++:35]
class Cipher {
    static async create(omniKey, keyId) {
        const info = new TextEncoder().encode(keyId);
        const salt = new Uint8Array();
        return new Cipher(await crypto.subtle.deriveKey(
            { name: 'HKDF', hash: 'SHA-512', salt, info },
            omniKey,
            { name: 'AES-GCM', length: 256 },
            false,
            ['encrypt', 'decrypt']
        ));
    }

    constructor(gcmKey) {
        this.gcmKey = gcmKey;
    }

    async encrypt(plaintext, iv) {
        const ciphertext = await crypto.subtle.encrypt(
            { name: 'AES-GCM', iv },
            this.gcmKey,
            plaintext
        );
        return ciphertext;
    }

    async decrypt(iv, ciphertext) {
        return crypto.subtle.decrypt(
            { name: 'AES-GCM', iv },
            this.gcmKey,
            ciphertext
        );
    }
}
```

There are many other ways to use the OmniKey, including using it to [derive raw bits](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/deriveBits), which is the lowest-level, most general function.
But for now, data privacy ready to go and it is time to get some data!

## IPFS Roundtrip

The stage is set for the final act.
We have our worker configured, the identity acquired, the OmniKey fetched, and the encryption key derived.
All that is left to do is store encrypted data to IPFS using NFT.storage!
For completeness, we will also decrypt it just to make sure everything worked okay.

Admittedly, this might not be the flash-bang demo for which you were looking, but it does keep the focus on the identity framework and how it works within a Smart Worker.

In the following diff, we will

1. define a simple message as content to store
2. encrypt it using the cipher created in the previous step
3. store it to IPFS using the NFT.storage API using your private API key
4. fetch the stored content from an IPFS gateway
5. decrypt the content using the cipher again
6. ensure that the data made it through safely

And without further ado, here it is:

```javascript
import escrinWorker from '@escrin/worker';
import { NFTStorage } from 'nft.storage'; // [!code ++]

export escrinWorker(new class {
    async tasks(rnr) {
        const { nftStorageToken, identity, network } = await rnr.getConfig();

        await rnr.acquireIdentity({ identity, network });
        const omniKey = await rnr.getOmniKey({ identity, network });

        const cipher = await Cipher.create(omniKey, 'escrin-demo/encryption');
        // [!code ++:19]
        const nftStorage = new NFTStorage({ token: nftStorageToken });

        // First encrypt & store some data to IPFS.
        const msg = 'Hello, world!';
        const iv = new Uint8Array(12);
        const ciphertext = await cipher.encrypt(new TextEncoder().encode(msg), iv);
        const content = new Blob([ciphertext]);
        const cid = await nftStorage.storeBlob(content);

        // Now the reverse.
        const res = await fetch(`https://${cid}.ipfs.nftstorage.link`);
        if (!res.ok) throw new Error('IPFS fetch failed');
        const content2 = await res.blob();
        const ciphertext2 = new Uint8Array(await content2.arrayBuffer());
        const plaintext = await cipher.decrypt(iv, ciphertext2);;
        const msg2 = new TextDecoder().decode(plaintext);

        if (msg !== msg2) throw new Error('huh...');
        console.log(msg);
    }
});

class Cipher {
    // snip ...
}
```

## Deployment

Just like in the [first Worker tutorial](./first-worker), we will need to bundle the `worker.js` with its dependencies and then submit the bundle along with `config.js` to the `escrin-runner`.

To bundle, use our favorite bundling command:

```sh
npx --no-install esbuild \
    --bundle ./worker.js --outfile=./bundled-worker.js \
    --target=es2022 --format=esm --minify
```

Deployment is quite similar to before, too.
First ensure that you are [running](../reference/runner) a local `escrin-runner` and then `curl` your way to victory.

```sh
curl -isS http://127.0.0.1:1057 \
    -F 'script=@bundled-worker.js' \
    -F 'type=module' \
    -F 'config=@config.json'
```

If everything works correctly, you will see no error messages.
You can also go to your [NFT.storage dashboard](https://nft.storage/files/) and see the IPFS CID that the worker just created.

Congratulations 🎉!
You have completed the full Escrin Smart Worker tutorial series.

## Recap & Next Steps

In this tutorial, you were exposed to the following concepts, which cap off your understanding of the Escrin identity framework:

* acquiring an identity using the `escrin-runner`
* that you can acquire an identity using a TPM attestation
* using an identity permit to retrieve an OmniKey
* deriving secrets from the OmniKey
* encrypting and decrypting data using the WebCrypto API

If you noticed that the objective of uploading and downloading data from IPFS was a bit contrived, you are not incorrect.
That a symptom of intentionally eschewing the task framework!
As underscored in the [previous tutorial](./first-identity) on the identity framework, workers are stateless and must bootstrap themselves when they spawn.
Fortunately, the blockchain is a highly available, durable store of state, which means that smart contracts can store all of the data smart worker needs for when it wakes up.
The exact way in which you set up your contracts-worker system is entirely up to you, but an excellent next step would be to check out the [code for the NFTrout demo app](https://github.com/escrin/nftrout) that builds on the concepts in this tutorial to provide a fun NFT game based on private genomes.

As you formulate your strategy for building on Escrin, please feel encouraged to drop by our [Telegram](https://escrin.org/telegram) and [Discord](https://escrin.org/discord) communities for friendly help and support!
Otherwise, you should be well on your way to creating autonomous computing systems using Escrin.
