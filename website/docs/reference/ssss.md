---
description:
  'The Simple Secret Sharing Server (SSSS) is a highly secure, decentralized secret management
  solution having tight integration with Escrin Smart Wokrers for trustworthy off-chain computation.'
---

# SSSS

The _Simple Secret Sharing Server_ (SSSS) is the next-generation first-party secret management
system used by all kinds of workers in the Escrin ecosystem. The name describes the function:

- _Simple_: the SSSS eschews as much complexity as possible to increase efficiency and
  decentralizability
- [_Secret Sharing_](https://en.wikipedia.org/wiki/Shamir%27s_secret_sharing): secrets are split and
  stored among distributed, non-colluding shareholders, which ensure that only authorized parties
  can reveal the secret
- _Server_: the SSSS stores encrypted secrets and serves them to authorized requesters

The SSSS, via the [Escrin identity framework](../tutorial/first-identity), securely holds secrets in
a decentralized manner. When an Escrin Worker begins its life, it will authenticate to the SSSSs
holding its OmniKey secret shares, fetch the secret shares, reconstruct the secret, and unwrap its
OmniKey.

Compared to the other key management mechanisms supported by Escrin, the SSSS is designed to be more
flexible, secure, and reliable than third party secret management networks.

## Design

<figure class="my-10">
<svg xmlns="http://www.w3.org/2000/svg" width="100%" viewBox="0 0 700.02 285.8"><g class="graph" transform="translate(4 281.8)"><path fill="var(--vp-c-bg)" stroke="transparent" d="M-4 4v-285.8h700.022V4z"/><g class="cluster"><path fill="none" stroke="var(--vp-c-text-1)" d="M200.712-49.8v-220h483.31v220z"/><text x="409.891" y="-253.2" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">SSSS Node</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M523.742-93.8h-74.42v36h74.42z"/><text x="457.177" y="-71.6" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">Sync Task</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M672.285-176.527c0-1.806-17.156-3.273-38.276-3.273s-38.276 1.467-38.276 3.273v29.454c0 1.806 17.156 3.273 38.276 3.273s38.276-1.467 38.276-3.273v-29.454"/><path fill="none" stroke="var(--vp-c-text-1)" d="M672.285-176.527c0 1.806-17.156 3.272-38.276 3.272s-38.276-1.466-38.276-3.272"/><text x="603.871" y="-157.6" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">Blob Store</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="m517.448-93.828 77.74-45.334"/><path fill="var(--vp-c-text-1)" stroke="var(--vp-c-text-1)" d="m593.74-142.369 10.401-2.014-6.875 8.061z"/><text x="545.5" y="-135" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">policy</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" stroke-width="2" d="M676.035-104.527c0-1.806-18.837-3.273-42.026-3.273s-42.026 1.467-42.026 3.273v29.454c0 1.806 18.837 3.273 42.026 3.273s42.026-1.467 42.026-3.273v-29.454"/><path fill="none" stroke="var(--vp-c-text-1)" stroke-width="2" d="M676.035-104.527c0 1.806-18.837 3.273-42.026 3.273s-42.026-1.467-42.026-3.273"/><text x="599.996" y="-85.6" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">Secret Store</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M523.82-69.861c16.127 1.669 35.353 2.472 59.84-1.415"/><path fill="var(--vp-c-text-1)" stroke="var(--vp-c-text-1)" d="m583.232-74.755 10.462 1.674-9.223 5.216z"/><text x="558.707" y="-56.102" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">secret share</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M217.212-166.8v-30h121v30z"/><text x="251.082" y="-177.2" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">API Task</text><path fill="none" stroke="var(--vp-c-text-1)" d="M217.212-138.8v-28h121v28z"/><text x="223.721" y="-149" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="12">POST ../permits</text><path fill="none" stroke="var(--vp-c-text-1)" d="M217.212-110.8v-28h121v28z"/><text x="230.92" y="-121" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="12">GET ../shares</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M533.968-236.8h-94.872v36h94.872z"/><text x="447.064" y="-214.6" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">Policy Engine</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M338.212-152.8c9.587 0 7.418-9.844 15.5-15 23.164-14.776 51.274-26.34 75.66-34.694"/><path fill="var(--vp-c-text-1)" stroke="var(--vp-c-text-1)" d="m428.271-205.816 10.594.165-8.385 6.477z"/><text x="353.712" y="-207" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">verify request</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M338.212-152.8c87.242 0 188.89-3.96 247.448-6.626"/><path fill="var(--vp-c-text-1)" stroke="var(--vp-c-text-1)" d="m585.579-162.926 10.15 3.035-9.827 3.957z"/><text x="468.258" y="-161" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">permit</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="m595.55-176.664-52.74-20.385"/><path fill="var(--vp-c-text-1)" stroke="var(--vp-c-text-1)" d="m541.26-193.895-8.065-6.87 10.589.34z"/><text x="542" y="-201" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">  policy</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M599.316-106.956c-5.042-1.932-10.245-3.636-15.32-4.844-103.093-24.533-134.446-13.734-235.756-13.035"/><path fill="var(--vp-c-text-1)" stroke="var(--vp-c-text-1)" d="m348.225-121.335-10.013-3.465 9.988-3.535z"/><text x="403.299" y="-115.138" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">secret share</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" stroke-width="2" d="M96.853-142.402H1.473v41.204h95.38z"/><text x="31.282" y="-126" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">Escrin</text><text x="9.319" y="-109.2" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">Smart Worker</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M96.904-135.397c30.662-7.646 71.786-15.92 110.11-17.225"/><path fill="var(--vp-c-text-1)" stroke="var(--vp-c-text-1)" d="m207.153-156.125 10.06 3.325-9.938 3.674z"/><text x="41.484" y="-151.213" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="12">acquire_identity</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M96.876-118.75c3.197.14 6.37.258 9.45.35 38.378 1.14 48.306 4.892 86.386 0 6.803-.874 10.233-3.06 14.422-4.633"/><path fill="var(--vp-c-text-1)" stroke="var(--vp-c-text-1)" d="m206.758-126.52 10.454 1.72-9.245 5.174z"/><text x="106.327" y="-121.4" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="12">get_omni_key</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M341.577-41.402h-124.73l-4 4V-.198h124.73l4-4zM337.577-37.402h-124.73M337.577-37.402V-.198M337.577-37.402l4-4"/><text x="221.03" y="-25" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">Consensus Network</text><text x="231.733" y="-8.2" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">(e.g., Ethereum)</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M86.429-101.153c30.24 16.274 74.27 38.89 114.283 55.353q1.272.523 2.564 1.042"/><path fill="var(--vp-c-text-1)" stroke="var(--vp-c-text-1)" d="m204.616-47.992 8.065 6.87-10.589-.341z"/><text x="57.431" y="-56.611" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">distribute shares</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="m341.438-37.676 97.945-25.735"/><path fill="var(--vp-c-text-1)" stroke="var(--vp-c-text-1)" d="m438.674-66.844 10.561.844-8.782 5.926z"/><text x="395.376" y="-39.248" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">policy &amp; secret share</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="m53.728-2.85 3.022-.098 2.991-.147 2.947-.196 2.892-.244 2.826-.292 2.746-.338 2.658-.383 2.56-.428 2.45-.471 2.333-.514 2.208-.554 2.076-.593 1.936-.63 1.79-.667 1.641-.7 1.487-.733 1.33-.763 1.17-.79 1.009-.817.847-.84.686-.861.525-.88.367-.897.21-.91.056-.923-.093-.93-.239-.938-.38-.942-.514-.942-.644-.942-.768-.937-.886-.931-.996-.922-1.102-.911-1.199-.897-1.29-.88-1.374-.86-1.452-.84-1.524-.817-1.589-.79-1.647-.764-1.701-.732-1.749-.7-1.79-.667-1.829-.631-1.861-.593-1.89-.554-1.915-.514-1.936-.471-1.954-.428-1.97-.383-1.981-.338-1.993-.292-2-.244-2.008-.196-2.012-.147-2.015-.099-2.017-.049h-2.019l-2.017.05-2.015.098-2.013.147-2.007.196-2 .244-1.993.292-1.982.338-1.97.383-1.953.428-1.936.471-1.915.514-1.89.554-1.861.593-1.829.63-1.79.667-1.749.7-1.7.733-1.648.763-1.59.79-1.523.817-1.452.84-1.374.861-1.29.88-1.2.897-1.1.91-.997.923-.886.93-.768.938-.644.942-.515.942-.38.942-.238.937-.093.931.057.922.21.911.366.897.525.88.686.86.847.84 1.009.817 1.17.79 1.33.764 1.487.732 1.64.7 1.791.667 1.936.631 2.075.593 2.208.554 2.334.514 2.45.471 2.56.428 2.657.383 2.747.338 2.825.292 2.892.244 2.948.196 2.99.147 3.023.099 3.04.049h3.048z"/><text x="23.7" y="-16.6" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">App Dev</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M97.527-20.8h105.31"/><path fill="var(--vp-c-text-1)" stroke="var(--vp-c-text-1)" d="m202.9-24.3 10 3.5-10 3.5z"/><text x="122.497" y="-25" fill="var(--vp-c-text-1)" font-family="Times,serif" font-size="14">set policy</text></g></g></svg>
<figcaption class="text-center mt-4">Figure 1. SSSS architecture overview</figcaption>
</figure>

### Goals

- implement the Escrin Identity interface (`acquire_identity`, `get_omni_key`)
- run efficiently on practically any hardware
- connect to any chain where users are found
- be straightforward to self-host or deploy in a permissioned setting
- scale to meet the needs of a large public network
- NOT implement yet another consensus layer
- NOT support programmable identity access policies (the Escrin Runner can be used for that)

The SSSS has four main components: the API task, the sync task, the policy engine, and the storage
backend. The storage backend can be implemented against any cloud or local host, which gives the
SSSS portability.

The sync task watches the `SSSSHub` contract on any EVM-compatible consensus network for policy
configuration changes and new secret shares. When it receives either, it unpacks the transaction
data and stores it in the backend. Policies are posted as Brotli-compressed CBOR documents, and each
share is encrypted under the SSSS's public key and bundled with a cryptographic commitment. All are
sent gas-efficiently as calldata or blobs (depending on availability) to enable deployment on as
many networks as possible.

The policy engine executes the policies received by the sync task against worker requests for
identity permits. Currently the only policy available is the one that verifies AWS Nitro Enclave
attestations, which is prohibitively expensive to do on most chains. Support for arbitrary
JavaScript and Python policies can easily be added in the future by executing them in the Escrin
Runner, which keeps the SSSS simple and easy to deploy.

The API task is a simple HTTP server that listens on a public network interface and is reachable
over TLS. Compared to a P2P network interface, this simple mechanism is highly robust and benefits
from decades of work on internet security, performance, and interoperability including DDoS
prevention, certificate transparency, and accessibility from web browsers.

Knitting these components together is state-of-the-art cryptography, memory-safe code, and
meticulous attention to detail. All combined, the SSSS is a secure, fault-tolerant system for
storing secrets in a cloud and L1-agnostic manner, leaving the power of choice in the hands of
developers and their users.

## Roadmap

The SSSS is currently in the alpha stages and, although fully functional, is missing several highly
desirable features and security enhancements.

The following roadmap items are in order, as each one is required by the next. All are towards the
smooth function of the public SSSS network. Private deployments of the SSSS are pretty much good to
go already.

**Phase I: Public Network Payments**

1. Enable [verifiable secret sharing](https://en.wikipedia.org/wiki/Verifiable_secret_sharing) to
   pinpoint which SSSS/Worker sent faulty shares
2. Add automatic trust-minimized batch payment settlement
3. Track per-node reliability metrics to allow participants to stop/continue payments.  
   As a stretch goal, create the SSSSScan web app to track SSSSs and their metrics.

**Phase II: Greater Decentralization**

The goal of this phase is to allow secrets to be shared on a rotating basis with all nodes that an
application trusts. This will allow for conveniently increasing the size of the set of shareholders,
as well as give smaller SSSS operators a greater opportunity to participate in the public network.

4. Provide auto-configuration of SSSS nodes using P2P gossip network
5. Enable [proactive secret sharing](https://en.wikipedia.org/wiki/Proactive_secret_sharing) for
   greater resilience to adaptive attacks

**Phase III: Trustless Payments**

6. Enable
   [publicly verifiable secret sharing](https://en.wikipedia.org/wiki/Publicly_Verifiable_Secret_Sharing)
   for on-chain verification of shares
7. Add automatic trustless payment settlement via smart contract using HE, and ZKP

## Trust

The following models assume the use of the SSSS in a public network configuration. If operated as
part of a permissioned Escrin deployment, users must trust the app dev to have properly deployed
their SSSSs to non-colluding entities.

### Security

The security of the SSSS is derived from the use of
[Shamir's secret sharing](https://en.wikipedia.org/wiki/Shamir%27s_secret_sharing), which itself is
information-theoretically secure. Practically, this means that as long as the cost to breach a
threshold of SSSSs is higher than the value obtained by doing so, an attack is unlikely occur.

Accordingly, the most important property for a set of SSSS shareholders is to be
non-colluding–essentially that they are operated by independent entities, each with different
motives, and ideally different deployment strategies.

A useful analogy is to the utility of genetic diversity of a population when assailed by a pathogen.
The pathogen's ability to proliferate and destroy the population greatly reduced by each organism
having its own genome-specific defenses that the pathogen must evolve to bypass. Similarly, a large
and motley group of SSSSs is more likely to be secure than a group operated in the same way or by
the same entity.

All this being said, assuming non-collusion and a sufficiently diverse attack surface such that only
fewer than the configured threshold of shares are missing or faulty, there are no risks to
security\*.

\* Currently secret shares are not verifiable, so a faulty share would likely result in the wrong
secret being reconstructed. However, Escrin OmniKeys are wrapped using
[authenticated encryption](https://en.wikipedia.org/wiki/Authenticated_encryption), so an incorrect
shared secret would cause an irrecoverable error rather than silently proceed. This problem will go
away once Phase I of the roadmap is complete.

### Liveness

The trust model for liveness can be summarized by the following table.

| ⬇️ trusts ➡️? | User |      App Dev      |      Worker       |     SSSS Node     |
| ------------- | :--: | :---------------: | :---------------: | :---------------: |
| **User**      |  -   |        ❌         |        ✅         |        ✅         |
| **App Dev**   |  ❌  |         -         |        ✅         | 🤝/✅<sup>1</sup> |
| **Worker**    |  ❌  |        ❔         |         -         | 🤝/✅<sup>2</sup> |
| **SSSS Node** |  ❌  | 🤝/❔<sup>1</sup> | 🤝/✅<sup>2</sup> |         -         |

❔ represents trust that is not established by the protocol, and 🤝 represents trust through
economic collaboration: an SSSS node will hold on to a worker's secrets as long as the worker pays
more than the cost to store them, and the worker will pay the SSSS node as long as the Worker's
application is worth the cost to maintain.

After Phase I of the roadmap is implemented, this model can be enforced automatically: SSSS nodes
and workers will maintain metrics of which workers/SSSS are unreliable on payments and drop them as
customers/service providers. And if an SSSS node goes down or becomes too expensive, the Worker can
re-share its secret with its new preferred set of secret shareholders. the SSSS.

<sup>1</sup> After Phase III of the roadmap, payments will be made by the worker directly without
the app dev in the loop.  
<sup>2</sup> After Phase III of the roadmap, payments for valid shares are guaranteed, and faulty
participants punished.

## Operating requirements

The SSSS is a lightweight program that can run on even the most resource-constrained hardware,
although it also benefits from the features provided by more powerful hardware. Accordingly, the
only hard requirements of the SSSS are:

- to be operated under security best-practices to minimize the risk of compromise
- a machine with 1 CPU and 0.5GB of memory
- a stable internet connection
- a domain name at which to host the SSSS API
- to rarely lose, delete, or otherwise corrupt the SSSS infrastructure

However, for greater security, an operator may choose to do any of the following:

- run replicas of the SSSS and load balance amongst them
- use multiple high-reliability Web3 gateway services for each blockchain
- run your own full/light nodes and Web3 gateways
- run the SSSS and blockchain nodes inside of TEE
- and generally do all of the things a Web2 API service provider would do

::: tip

The security of secret sharing improves with the number of non-colluding shareholders, so more SSSSs
with decent deployment security (and non-overlapping vulnerabilities) is better than a smaller
number of locked down SSSSs, so these extras are truly only nice-to-have.

:::

The SSSS can be deployed on AWS, Azure, or locally at very low cost, or even completely free under
free tiers. The most basic configuration on the clouds is about $6/month before free tier
incentives, so everyone who wants to run an SSSS should be able to. But do expect competition on the
public network from operators who invested in higher reliability and defenses!

## Deployment

For deployment scripts and detailed instructions, please refer to the
[deployments README](https://github.com/escrin/escrin/tree/main/ssss/deploy).

If you want to run the latest versioned release of the SSSS for testing, you can execute the
following command:

```sh
docker run --init --rm --name ssss -p 1075:1075 ghcr.io/escrin/ssss --help
```

To obtain the latest "nightly" release, you can grab it (and `s4`) from the latest `main` branch
build of the [SSSS CI workflow](https://github.com/escrin/escrin/actions/workflows/ssss.yaml).

To create a lighter build of the SSSS that only targets on the particular host that you plan to use,
either ask in the [community chat](https://escrin.org/discord) or be a Rust developer and build the
SSSS from [source](https://github.com/escrin/escrin/tree/main/ssss) with the appropriate feature
flag and target:

```sh
export target={x86_64,aarch64}-unknown-linux-musl
rustup target add $target
cargo build -p ssss --locked --release --target $target \
    --no-default-features --feature {aws,azure,local}
```

## Utilities

The `s4` CLI tool has several commands that enable convenient debugging and testing of your SSSS.
When running against a local SSSS and fresh `anvil` testnet, commands can be run with few or no
arguments.

```
Usage: s4 [OPTIONS] <COMMAND>

Commands:
  set-policy        Write a new policy to the blockchain for retrieval by the listening SSSSs
  acquire-identity  Acquire an Escrin identity from an SSSS
  deal              Split a secret into shares and deal it to the requested SSSSs
  reconstruct       Reconstructs a secret from shares requested by the requested SSSSs
  help              Print this message or the help of the given subcommand(s)

Options:
  -v, --verbosity...
  -h, --help          Print help
  -V, --version       Print version
```
