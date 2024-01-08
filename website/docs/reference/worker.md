---
outline: deep
---

# Smart Worker Reference

A Smart Worker runs within a JavaScript [`WorkerGlobalScope`](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope) and therefore has most of the same global properties found in a browser execution context (e.g., `fetch`, `setTimeout`, `crypto.subtle`).

## Smart Worker Lifecycle

::: warning
The worker creation interface is almost certainly going to change in the future, as this one is very simplistic and only intended to provide a publicly usable API.
:::

The Escrin Runner will listen on a local port (8080 by default) for requests to run Smart Workers.
The request may be sent as JSON or form data having the following properties:

::: code-group

```ts [Interface]
interface CreateWorkerRequest {
  /// The complete script to run, including any imported modules.
  /// In most cases, this will be the output of a bundler like `esbuild --bundle`.
  script: string;

  /// The type of script being sent. If ESM, send `module`, otherwise `classic` for service worker.
  type: 'module' | 'classic';

  /// Optional arbitrary JSON that will be exposed to the worker via `rnr.getConfig()`.
  /// These variables are private and known only to the Escrin Runtime and newly-created Worker.
  config?: Record<string, unknown>

  // Triggers:

  /// A cron string describing when to trigger the worker's `tasks()` handler.
  schedule?: string;
}
```

```sh [curl]
curl -isS http://localhost:8080 -F 'script=@<path-to-script>' -F 'type=module' -F 'schedule="*/5 * * * *"' -F config="<JSON object>"
```

:::

## Configuration

### Triggers

Smart Workers are dormant unless an external event wakes them up.
There are several events to which a worker will respond.

#### Cron Schedule

Smart Workers can be requested to run periodically according to a [cron](http://crontab.guru/) schedule.
When the scheduled time has arrived, the `tasks()` callback will be called.

:::info
Currently, the only supported cron schedule is every five minutes (`*/5 * * * *`).
:::

#### HTTP Request

This feature is not currently implemented, but workers will be able to register themselves at a dedicated URL so that they can directly process requests made by clients.
This will look almost exactly like it does in [Cloudflare Workers](https://developers.cloudflare.com/workers/runtime-apis/fetch-event), as the Escrin Runner is based on the same code.

The Escrin Runner will securely terminate TLS connections within its trusted execution environment using a TLS secret known only to the runner enclave.

#### On-Chain Events

This feature is not currently implemented, but workers will be able to register themselves to run whenever a [TaskHub](https://github.com/escrin/escrin/blob/main/evm/contracts/tasks/hub/ITaskHub.sol) contract notifies the world of tasks being available.

## Runner Bindings

The Escrin Runner provides a [variety](https://github.com/escrin/escrin/blob/main/runner/src/worker-interface.ts) of methods to Smart Workers that enhance convenience, security, or both.

A Smart Worker is able to communicate with the Escrin Runner via the [`EscrinRunner`](https://github.com/escrin/escrin/blob/main/runner/src/worker-interface.ts) provided to each worker callback.

The return values runner bindings are cached for an appropriate length of time, so it is not necessary for worker code to additionally cache them.
Just request what you need when you need it.

### Configuration

The Smart Worker can access its configuration at any time by calling the `getConfig` binding.
It will return whatever (valid) data was securely provided during construction.

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

::: details Design note
The key management interface is purposely kept narrow (i.e. a single key) so that more key management networks can be supported in the future.

Although generating and storing many keys is convenient, it is not necessarily cheap to do on all networks like it is on Sapphire.
For example, distributed key generation, used by secret sharing networks, is a rather expensive operation.

This API may be expanded in the future to take advantage of network-specific features.
Please submit a feature request if this is something you would like to see!
:::
