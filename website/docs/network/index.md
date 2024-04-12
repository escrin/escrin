---
description:
  'The Escrin Network adds high-performance, high-trust off-chain computation to all blockchains.'
---

# Escrin Network

Escrin is a framework that extends the autonomy and self-execution nature of smart contracts into
the real world through high-integrity, trustworthy off-chain computation powered by trusted
execution environments (TEE), secret sharing, and techniques like zero-knowledge proofs and
homomorphic encryption, where applicable.

The Escrin Network is the hosted offering of Escrin, owned and operated by the community. Although
Escrin is designed to be easily self-hosted, many developers prefer the supreme convenience of
Web3-flavored serverless computing. Accordingly, a public network is an ideal way to build and scale
with Escrin, while benefiting from the flexibility and robustness of a permissionless, decentralized
network. In many senses, the Escrin Network is positioned to be the AWS of Web3, having secure,
trust minimized virtual machines, key management, storage, and tight integration with consensus
networks like Ethereum, L2s, and above. The goal of the Escrin Network is to provide smart
infrastructure for next-generation enhanced-convenience and interactivity dapps.

The Escrin Network, like the computation it hosts, is autonomous, being held together by the
protocol and the $ESCRIN utility token. Anyone can operate an Escrin node and compete in the global
market to provide value to users. Developers and their end-users can leverage the Escrin Network to
augment their dapps with scalable, high-performance, and high-integrity computation to enable
use-cases like AI, real-time gaming, and advanced Web3 [interconnectivity](../apps/nanobridges).

<!-- use-cases like [AI](../apps/ai), real-time [gaming](../apps/games), and advanced Web3 -->

The end vision for the Escrin Network is to provide the ideal substrate large-scale autonomous
computation and foster an ecosystem of programs that exist for their own sake, generating value for
themselves and others, independently of their creator or any operator. For example, imagine the
utility of an AI agent that can ingest private data, think private thoughts, and be trusted to
faithfully execute its plans. Escrin is intended to provide the technological foundations for such
an application.

<!-- [technological foundations](../guide/technology) for such an application. -->

The remainder of this document covers the design and function of Escrin specifically in the context
of the Escrin Network.

## Technical Architecture

### Overview

> Choice. The problem is choice.
>
> \- Neo, describing why the previous Matrix failed

The number one goal of the Escrin Network is to provide trustworthy off-chain computation. The
number two goal is to give users as much choice as possible.

The primary method by which Escrin provides trusted computation is through tight integration with
[trusted execution environments (TEEs)](https://en.wikipedia.org/wiki/Trusted_execution_environment),
though these are by no means required. A TEE is essentially a hardened general-purpose computing
environment in which code can run with privacy and also
[cryptographic guarantees of the identity](https://en.wikipedia.org/wiki/Trusted_Computing#Remote_attestation)
of the code that is running. In other words, TEE provides integrity and confidentiality, which means
that users can entrust their secret data to the application running within, and also trust the
outputs. TEEs have near-identical performance to normal hardware and are even built into some GPU
accelerators, enabling efficient large-scale computation. When properly configured, TEE makes it
possible to trust a cloud computer as though it were one's own.

One major shortcoming of TEE and earlier TEE-based networks is lack of choice: TEEs are very strong
but are vulnerable to side-channel and physical attacks, which means that TEEs are not safely
operated by one's enemies. The traditional solutions have been to do nothing and end up in a
research paper, or operate a permissioned network and masquerade as a public network.

Escrin solves the problem of choice by creating implicit sub-networks when the level of trust
between nodes changes. By defining classes of trustworthy nodes, complete choice in
trust-versus-decentralization lies in the hands of applications. This approach is a departure from
the now-common parachain model in which disparate app-chains share a base network layer. In Escrin,
every node runs the same software, so there are no hard divisions between trust boundaries and
applications can scope their execution environment as broadly as they trust.

For example, consider that you are an AI agent running on Escrin: you may feel safe running on any
AWS TEE, but you don't trust Intel TEE unless you consider the operator to be highly reputable.
Expressed in code, this defines your trust boundary. You can still interoperate with applications
within other trust boundaries, but the data might be subject to
[enhanced validation](https://en.wikipedia.org/wiki/Document_legalization) or authentication by a
mutually trusted third party.

All this being said, the Escrin network is focused on extending the autonomy of smart contracts into
the real world and is not intended to bake a consensus protocol into the platform. Instead Escrin
meets developers and users where they already are, to provide maximal security without compromising
on decentralization, through the power of choice.

### Node Types

Like any good cloud platform, Escrin provides computation nodes. An Escrin computation node is
usually an instance of [`escrin-runner`](../reference/runner), which is a modified version of
Cloudflare's highly reliable Workerd JavaScript/Wasm/Python runtime. Other runner types are
possible, though changing the code bundle necessarily puts them into a different trust boundary. The
`escrin-runner` lives in a TEE and protects the _smart worker_ code running within. Smart workers
are the main interaction points of both developers and users of the Escrin Network. Runner nodes are
generally more expensive to run than the other kinds and must be more highly trusted due to their
access to secrets; most trust boundaries are predicated on the operation context of runners. As a
result, most $ESCRIN will flow to runners, which settle hardware costs and upstream fees on the
backend.

Computation in TEE is good, but without decentralized secret management, it is limited to ephemeral
tasks. With secret management, a smart worker can gain access to an autonomous stable identity,
encryption keys, crypto wallets, and so forth. Secret management gives a smart worker all of the
capabilities of a user of Web3. Therefore, in order to provide a highly convenient developer
experience, the Escrin Network includes first-class support for secret management via a secret
sharing node known as the [**SSSS**](../reference/ssss). The SSSS is tightly integrated with the
smart worker lifecycle to make verifying machine identities and operational contexts secure,
flexible, and straightforward. SSSSs are intended to be cheap to run and widely available, making an
ideal choice for a new operator wishing to join the network. Beyond the technical requirements, all
one needs is a moderate amount of $ESCRIN to demonstrate commitment to the security of the network.

In addition to computation and secret management, the existence of a storage node is strongly
implied by the goal of becoming the secure cloud for Web3. Motivated by the goal of becoming the
cloud for Web3, leveraging the trustworthy nature of Escrin Workers to provide more featureful
databases like SQL and NoSQL is an active area of research.

An operator is not limited to one node type and can in fact run all kinds, thereby benefiting from
economies of scale. For example, an SSSS does not require TEE, but TEE certainly enhances its
security posture. If there's already a TEE-enabled runner running, why not put an SSSS on there too!

Operators and app devs can coordinate with each other using the Escrin Observer, a sort of network
explorer that shows statistics that allow participants to pick nodes that they trust.

### Security

The security of the Escrin Network is based on assigning trust to the right participants. This is
easier said than done, but the general idea is that Escrin nodes establish trust from whatever
sources are available rather than solely through cryptoeconomics–incentive compatibility and
Sybil-resistance are all that matter.

To better understand why staking is not always the only option for establishing security, consider a
scenario involving a dev shop that runs Escrin nodes primarily for their own app, but also makes
them available for others to use: in this case the credibility of the dev shop and the success of
its app is at stake. Since devs can directly pick the nodes on which their smart workers run, it is
perfectly secure and acceptable to use on these nodes without requiring additional indicators. Of
course, if your app requires further guarantees, you can always fall back on protocol-level
mechanisms without relying on out-of-band trust.

The protocol-level security mechanisms are intended to preventing one entity from spoofing a large
number of nodes to violate non-collusion assumptions or make it more likely to run on their broken
hardware. This property is known as Sybil-resistance. Escrin primarily relies on
[fidelity bonds](https://en.bitcoin.it/wiki/Fidelity_bonds) in which an operator burns or locks up
$ESCRIN to prove that they are either a single entity or hold enough $ESCRIN that breaking the
network would be personally destructive. Most operators will choose to post time-locked fidelity
bonds which are much like proof-of-stake, but the protocol also supports burnt offerings. There is
no optimal value for the amount of bond that must be posted, as it depends on how much value the
smart worker controls, so it is up to the market of individual app devs to negotiate with operators.

By the same logic of letting the market sort out which node operators to use, Escrin (mostly) does
not introduce slashing. Instead, apps and the Escrin Observer track statistics about the different
nodes and unreliable ones will see their $ESCRIN revenue decrease, which is basically slashing.
After all, why pay for many nines of availability if only a few are required? Again, Escrin is all
about choice.

## Tokenomics

The Escrin Network is "omni-chain" in that it can add autonomous computation to any consensus
network. Indeed, since Escrin makes it so easy for dapps to operate across chains, it is expected
that most dapps using the Escrin Network will be chain-agnostic as well. Therefore to simplify
payments for computation within the network, we introduce the $ESCRIN token as a standardized unit
of value accepted by any node in the network on any chain where $ESCRIN can be found.

$ESCRIN is a network-specific utility token with no intrinsic value that only can be used within the
Escrin Network. It is used to pay for computation, upstream gas, secret sharing, and storage; also
for securing the network by posting fidelity bonds and staking. The amount of payment to nodes is
set by the node operators that the application has chosen to use; it is likely that more trustworthy
operators will command a premium gas fee.

### Allocation

<svg xmlns="http://www.w3.org/2000/svg" viewBox="120 75 400 255" class="w-2/3 mx-auto my-12"><path fill="none" d="M0 0h635v407.5H0z"/><path fill="none" d="M122 44h392v13H122z"/><path fill="#36c" stroke="var(--vp-c-bg)" d="M318 204V78a126 126 0 0 1 126 126H318"/><text x="384.153" y="135.397" fill="var(--vp-c-bg)" stroke-width="0" font-family="Arial" font-size="13">25</text><path fill="#dc3912" stroke="var(--vp-c-bg)" d="M318 204h126a126 126 0 0 1-102.39 123.768L318 204"/><text x="390.719" y="274.499" fill="var(--vp-c-bg)" stroke-width="0" font-family="Arial" font-size="13">22</text><path fill="#d47" stroke="var(--vp-c-bg)" d="M318 204 279.064 84.167A126 126 0 0 1 318 78v126"/><text x="297.917" y="103.849" fill="var(--vp-c-bg)" stroke-width="0" font-family="Arial" font-size="13">5</text><path fill="#0099c6" stroke="var(--vp-c-bg)" d="m318 204-91.85-86.253a126 126 0 0 1 52.914-33.58L318 204"/><text x="257.898" y="119.359" fill="var(--vp-c-bg)" stroke-width="0" font-family="Arial" font-size="13">8</text><path fill="#909" stroke="var(--vp-c-bg)" d="m318 204-125.006-15.792a126 126 0 0 1 33.156-70.46L318 204"/><text x="217.023" y="164.328" fill="var(--vp-c-bg)" stroke-width="0" font-family="Arial" font-size="13">10</text><path fill="#109618" stroke="var(--vp-c-bg)" d="m318 204-86.253 91.85a126 126 0 0 1-38.753-107.642L318 204"/><text x="213.007" y="243.83" fill="var(--vp-c-bg)" stroke-width="0" font-family="Arial" font-size="13">15</text><path fill="#f90" stroke="var(--vp-c-bg)" d="m318 204 23.61 123.768a126 126 0 0 1-109.863-31.918L318 204"/><text x="281.79" y="309.091" fill="var(--vp-c-bg)" stroke-width="0" font-family="Arial" font-size="13">15</text><text x="514" y="131.033" fill="var(--vp-c-text-1)" stroke-width="0" font-family="Arial" font-size="13" text-anchor="end">Ecosystem</text><path fill="none" d="M459 119.983h55v13h-55z"/><text x="514" y="152.067" fill="#9e9e9e" stroke-width="0" font-family="Arial" font-size="13" text-anchor="end">25%</text><path fill="none" stroke="#636363" stroke-opacity=".7" d="M385.5 137.5h129"/><circle cx="385.5" cy="137.5" r="2" fill="#636363" fill-opacity=".7"/><g fill="var(--vp-c-text-1)" stroke-width="0" font-family="Arial" font-size="13" text-anchor="end"><text x="514" y="241.015">Team
&amp;</text><text x="514" y="258.033">Advisors</text></g><text x="514" y="279.067" fill="#9e9e9e" stroke-width="0" font-family="Arial" font-size="13" text-anchor="end">22%</text><path fill="none" stroke="#636363" stroke-opacity=".7" d="M391.5 264.5h123"/><circle cx="391.5" cy="264.5" r="2" fill="#636363" fill-opacity=".7"/><text x="122" y="289.033" fill="var(--vp-c-text-1)" stroke-width="0" font-family="Arial" font-size="13">Pre-Sale</text><text x="122" y="310.067" fill="#9e9e9e" stroke-width="0" font-family="Arial" font-size="13">15%</text><path fill="none" stroke="#636363" stroke-opacity=".7" d="M292.5 295.5h-170"/><circle cx="292.5" cy="295.5" r="2" fill="#636363" fill-opacity=".7"/><g fill="var(--vp-c-text-1)" stroke-width="0" font-family="Arial" font-size="13"><text x="122" y="218.015">Public</text><text x="122" y="235.033">Sale</text></g><text x="122" y="256.067" fill="#9e9e9e" stroke-width="0" font-family="Arial" font-size="13">15%</text><path fill="none" stroke="#636363" stroke-opacity=".7" d="M231.5 241.5h-109"/><circle cx="231.5" cy="241.5" r="2" fill="#636363" fill-opacity=".7"/><text x="122" y="181.033" fill="var(--vp-c-text-1)" stroke-width="0" font-family="Arial" font-size="13">Liquidity</text><text x="122" y="202.067" fill="#9e9e9e" stroke-width="0" font-family="Arial" font-size="13">10%</text><path fill="none" stroke="#636363" stroke-opacity=".7" d="M230.5 169.5h-51v18h-57"/><circle cx="230.5" cy="169.5" r="2" fill="#636363" fill-opacity=".7"/><g fill="var(--vp-c-text-1)" stroke-width="0" font-family="Arial" font-size="13"><text x="122" y="126.015">Staking</text><text x="122" y="143.033">Rewards</text></g><text x="122" y="164.067" fill="#9e9e9e" stroke-width="0" font-family="Arial" font-size="13">8%</text><path fill="none" stroke="#636363" stroke-opacity=".7" d="M267.5 124.5h-88v25h-57"/><circle cx="267.5" cy="124.5" r="2" fill="#636363" fill-opacity=".7"/><text x="122" y="89.033" fill="var(--vp-c-text-1)" stroke-width="0" font-family="Arial" font-size="13">Airdrop</text><text x="122" y="110.067" fill="#9e9e9e" stroke-width="0" font-family="Arial" font-size="13">5%</text><path fill="none" stroke="#636363" stroke-opacity=".7" d="M303.5 111.5h-124v-16h-57"/><circle cx="303.5" cy="111.5" r="2" fill="#636363" fill-opacity=".7"/></svg>

| Category        | Purpose                              | Conditions                                                                    |
| --------------- | ------------------------------------ | ----------------------------------------------------------------------------- |
| Ecosystem       | Grants, partnerships, treasury, etc. | Unlocks linearly over 24 months                                               |
| Team & Advisors | Incentivizing core contributors      | 20% cliff at 6 months, remainder unlocks linearly over 24 months              |
| Pre-sale        | Activating strategic partners        | 10% at TGE, 20% cliff at 6 months, remainder unlocks linearly over 24 months  |
| Public Sale     | Decentralization                     | 10% upfront, 20% cliff at 4 months, remainder unlocks linearly over 12 months |
| Liquidity       | Minimizing slippage                  | Locked immediately                                                            |
| Staking Rewards | Reward for securing the network      | Fixed 2% APY with 2 week lockup period                                        |
| Airdrop         | Early access to early supporters     | Unlocks linearly over 3 months                                                |

## Governance

The Escrin Network does not have any features that would fall under governance. Fully embracing the
ethos of Web3, all smart contracts do not have an owner and do not have proxies/backdoors.

Upgrades can be proposed by any member of the community, and operators are free to integrate changes
without permission or risk of being slashed for divergence. Although it is recommended that node
operators do not introduce breaking changes to the core protocol, nothing particularly bad happens
if they do. All that will occur is that affected applications will not run on inhospitable nodes
until they receive updates. The governance decision is therefore delegated to the application's own
community, where it is more clear which centralization and security trade-offs are acceptable. It is
expected that the free market for computation will ensure continuity of service: operators will
continue to support dapps that generate significant $ESCRIN revenue, or new ones will take their
place.

The Escrin Foundation, of course, will play a role in shaping the evolution of the network. Namely,
the Foundation will maintain community social channels, evangelize the network, encourage new
applications built on the network, and develop core infrastructure with the advice of the community.

## Roadmap

- Mid 2024: Public network launch
- Late 2024: Ability to run runner nodes, SSSS upgrades
- 2025: Large scale adoption

<!-- ## Legal Considerations -->

<!-- ## Conclusion -->
