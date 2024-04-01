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
<svg xmlns="http://www.w3.org/2000/svg" class="w-full mx-auto my-8" viewBox="0 0 668.55 341.8"><g class="graph" transform="translate(4 337.8)"><path fill="var(--vp-c-bg)" stroke="transparent" d="M-4 4v-341.8h668.547V4z"/><g class="cluster"><path fill="none" stroke="var(--vp-c-text-1)" d="M176.715-49.8v-276h475.832v276z"/><text x="377.285" y="-309.2" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">SSSS Node</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M515.091-93.8H433.4v36h81.691z"/><text x="441.573" y="-71.6" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Sync Task</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M643.32-227.282c0-2.88-12.101-5.218-27-5.218s-27 2.339-27 5.218v46.964c0 2.88 12.102 5.218 27 5.218s27-2.339 27-5.218v-46.964"/><path fill="none" stroke="var(--vp-c-text-1)" d="M643.32-227.282c0 2.88-12.101 5.218-27 5.218s-27-2.339-27-5.218"/><text x="602.316" y="-208" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Blob</text><text x="599.594" y="-191.2" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Store</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="m494.3-93.868 87.326-78.674"/><path stroke="var(--vp-c-text-1)" d="m579.3-175.157 9.772-4.093-5.087 9.293z"/><text x="534.589" y="-173" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">policy  </text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" stroke-width="2" d="M644.775-122.282c0-2.88-12.754-5.218-28.454-5.218s-28.455 2.339-28.455 5.218v46.964c0 2.88 12.754 5.218 28.455 5.218s28.454-2.339 28.454-5.218v-46.964"/><path fill="none" stroke="var(--vp-c-text-1)" stroke-width="2" d="M644.775-122.282c0 2.88-12.754 5.218-28.454 5.218s-28.455-2.339-28.455-5.218"/><text x="596.094" y="-103" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Secret</text><text x="599.594" y="-86.2" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Store</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M514.942-66.96c17.828 2.35 38.976 3.075 63.525-4.2"/><path stroke="var(--vp-c-text-1)" d="m577.448-74.51 10.594.156-8.38 6.484z"/><text x="551.765" y="-53.365" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">secret share</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M193.215-206.8v-30h121v30z"/><text x="225.32" y="-217.2" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">API Task</text><path fill="none" stroke="var(--vp-c-text-1)" d="M193.215-178.8v-28h121v28z"/><text x="199.724" y="-189" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="12">POST ../permits</text><path fill="none" stroke="var(--vp-c-text-1)" d="M193.215-150.8v-28h121v28z"/><text x="206.923" y="-161" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="12">GET ../shares</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M524.542-292.8H423.95v36h100.593z"/><text x="431.847" y="-270.6" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Policy Engine</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M314.215-192.8c12.183 0 7.838-13.737 17.62-21 24.618-18.28 55.55-32.375 82.293-42.388"/><path stroke="var(--vp-c-text-1)" d="m413.177-259.567 10.594-.126-8.203 6.705z"/><text x="331.835" y="-260" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">verify request</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M314.215-192.8c95.407 0 207.65-5.547 264.711-8.762"/><path stroke="var(--vp-c-text-1)" d="m579.068-205.076 10.184 2.923-9.784 4.065z"/><text x="454.804" y="-202" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">permit</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="m589.141-217.383-69.483-34.723"/><path stroke="var(--vp-c-text-1)" d="m517.845-249.099-7.38-7.601 10.51 1.34z"/><text x="532.644" y="-247" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">   policy</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M587.821-121.952c-2.527-1.455-5.118-2.766-7.727-3.848-107.093-44.406-144.266-39.35-255.593-39.016"/><path stroke="var(--vp-c-text-1)" d="m324.22-161.315-10.005-3.485 9.994-3.515z"/><text x="376.012" y="-149.584" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">secret share</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" stroke-width="2" d="M102.51-175.402H-.138v41.204H102.51z"/><text x="31.743" y="-159" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Escrin</text><text x="8.025" y="-142.2" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Smart Worker</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M102.584-175.468c23.693-7.938 52.675-15.484 80.46-17.04"/><path stroke="var(--vp-c-text-1)" d="m183.119-196.012 10.096 3.212-9.896 3.785z"/><text x="31.898" y="-191.229" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="12">acquire_identity</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M102.515-154.432c2.657.012 5.289.023 7.858.032 25.93.088 32.932 5.17 58.342 0 7.245-1.474 10.383-5.168 14.606-7.731"/><path stroke="var(--vp-c-text-1)" d="m182.648-165.575 10.567.775-8.744 5.984z"/><text x="114.348" y="-157.4" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="12">get_key</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M323.955-41.402h-137.48l-4 4V-.198h137.48l4-4zM319.955-37.402h-137.48M319.955-37.402V-.198M319.955-37.402l4-4"/><text x="190.595" y="-25" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Consensus Network</text><text x="202.651" y="-8.2" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">(e.g., Ethereum)</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M70.12-133.92c21.693 22.852 59.399 59.465 106.078 87.342"/><path stroke="var(--vp-c-text-1)" d="m178.11-49.515 6.906 8.035-10.409-1.975z"/><text x="21.091" y="-71.136" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">distribute shares</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="m324.062-38.43 99.28-24.703"/><path stroke="var(--vp-c-text-1)" d="m422.853-66.618 10.55.981-8.86 5.811z"/><text x="354.007" y="-81.8" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">policy,</text><text x="335.333" y="-65" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">secret share</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="m55.94-2.85 3.148-.098 3.115-.147 3.07-.196 3.011-.244 2.942-.292 2.861-.338 2.769-.383 2.665-.428 2.552-.471 2.43-.514 2.3-.554 2.161-.593 2.016-.63 1.865-.667 1.71-.7 1.548-.733 1.385-.763 1.218-.79 1.051-.817.882-.84.715-.861.547-.88.381-.897.219-.91.059-.923-.097-.93-.249-.938-.395-.942-.536-.942-.671-.942-.8-.937-.922-.931-1.038-.922-1.147-.911-1.249-.897-1.343-.88-1.432-.86-1.512-.84-1.587-.817-1.655-.79-1.716-.764-1.771-.732-1.821-.7-1.865-.667-1.904-.631-1.939-.593-1.968-.554-1.994-.514-2.017-.471-2.035-.428-2.05-.383-2.065-.338-2.075-.292-2.084-.244-2.09-.196-2.096-.147-2.099-.099-2.1-.049h-2.103l-2.1.05-2.1.098-2.095.147-2.09.196-2.084.244-2.075.292-2.065.338-2.05.383-2.036.428-2.016.471-1.994.514-1.969.554-1.938.593-1.904.63-1.865.667-1.821.7-1.772.733-1.716.763-1.654.79-1.587.817-1.513.84-1.431.861-1.344.88-1.248.897-1.147.91-1.038.923-.922.93-.8.938-.671.942-.536.942-.395.942-.249.937-.097.931.059.922.218.911.382.897.547.88.714.86.883.84 1.05.817 1.219.79 1.385.764 1.548.732 1.71.7 1.864.667 2.017.631 2.16.593 2.3.554 2.43.514 2.553.471 2.665.428 2.768.383 2.861.338 2.942.292 3.012.244 3.07.196 3.115.147 3.147.099L49.6-2.8h3.174z"/><text x="24.346" y="-16.6" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">App Dev</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M101.646-20.8h70.692"/><path stroke="var(--vp-c-text-1)" d="m172.459-24.3 10 3.5-10 3.5z"/><text x="110.373" y="-25" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">set policy</text></g></g></svg>
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
