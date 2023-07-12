# Smart Worker Reference

A Smart Worker runs within a JavaScript [`WorkerGlobalScope`](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope) and therefore has most of the same global properties found in a browser execution context (e.g., `fetch`, `setTimeout`, `crypto.subtle`).

## Smart Worker Lifecycle

## Configuration


## Runner Bindings

### Configuration


```ts
interface EscrinRunner {
  /// Returns the parsed configuration that was sent when the worker was created.
  getConfig(): Promise<Record<string, unknown>>;
}
```

### Key Management

Each Smart Worker is endowed with an _Omni Key_, a secret known only to instances of the Smart Worker and nobody else.
The Omni Key is securely stored on a key management blockchain network (KM network) and is released only to workers that satisfy the worker's _key policy_.
The key policy is a smart contract that lives on your application's preferred network, which may be the same or different from the key management network.

The Omni Key is requested and managed by the Escrin Runner on behalf of the Smart Worker.
All the smart worker must do is call the `getKey` runner binding with desired KM network.
A worker may have an Omni Key on each key management network.

```ts
/// The networks supported by the Escrin Runner for secret key management.
type KmNetwork = 'sapphire-mainnet' | 'sapphire-testnet';

interface EscrinRunner {
  /// Requests the Omni Key from the key store, negotiating first with any configured key policies.
  /// The Omni Key will be randomly generated the first time it is requested.
  /// @param store The name of the key management from which the key will be fetched.
  getOmniKey(store: KmNetwork): Promise<CryptoKey>;
}
```

The returned `CryptoKey` may be used for the [`deriveBits`](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/deriveBits) and [`deriveKey`](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/deriveKey) methods on `crypto.subtle`.

::: tip
The Omni Key can only be used to derive sub-keys for encryption/decryption, key agreement, or deterministic random number generation.
It cannot be used for these things directly.

You can refer to the [NFTrout `crypto.ts` example](https://github.com/escrin/nftrout/blob/main/worker/src/crypto.ts) to see the Omni Key in action.
:::

::: details
The key management interface is purposely kept narrow (i.e. a single key) so that more key management networks can be supported in the future.

Although generating and storing many keys is convenient, it is not necessarily cheap to do on all networks like it is on Sapphire.
For example, distributed key generation, used by secret sharing networks, is a rather expensive operation.

This API may be expanded in the future to take advantage of network-specific features.
Please submit a feature request if this is something you would like to see!
:::
