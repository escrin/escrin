---
description: |
  Nanobridges are the fastest and most convenient way to securely connect your dapp to new chains and Web2 services.
---

# Nanobridges

## Introduction

As the Web3 ecosystem continues to expand, the demand for secure, efficient, and versatile
cross-chain communication solutions has never been higher. Enter nanobridges – a revolutionary
application of the Escrin autonomous computing framework that leverages Trusted Execution
Environments (TEEs) to provide developers with unparalleled customization and performance.

## How Nanobridges Work

At the core of nanobridges lies the Escrin framework, which uses
[Trusted Execution Environments (TEEs)](https://en.wikipedia.org/wiki/Trusted_execution_environment),
[decentralized secret management](../reference/ssss), and on-chain roots of trust to establish
highly secure enclaves of computation, known as Escrin Workers. Escrin Workers can execute
JavaScript, Wasm, and Python code with efficiency, privacy, and integrity.

Nanobridges are special-purpose Escrin Workers that facilitate cross-chain communication. A
nanobridge can execute custom message-passing workflows with speed efficiency on par with a native
virtual machine instances due to the integrity properties of TEE and cryptographic attestations of
machine identity. Additionally, the privacy properties of TEE enable fetching data using credentials
like API keys, and selective disclosure of secret fetched data.

If you are familiar with the Avalanche-Ethereum bridge, nanobridges are built on very similar
security technology, but are user-programmable for precise control over cost, decentralization,
robustness, and performance.

## Nanobridges vs. Traditional Bridges

To better understand the advantages of nanobridges, let's compare them to a traditional
message-passing bridge like Celer IM, Chainlink CCIP, and Axelar.

While the traditional bridge provides a secure and reliable way to communicate between the networks,
it has some limitations:

1. **Speed**: A traditional bridge can often take several minutes post-confirmation to pass a
   message between chains due to factors outside of a developer's control like batching, proofs, and
   consensus.
2. **Cost**: Users must pay gas on both networks to initiate a message passing event, which is
   wasteful if the data is available through read-only calls.
3. **Flexibility**: The traditional bridge only moves bytes from one chain to another and cannot be
   used for complex, multi-step, and multi-chain workflows.

In contrast, nanobridges offer:

1. **Speed**: Developers can customize the speed of message bridging, enabling sub-second
   cross-chain communication when needed.
2. **Cost**: With no additional fees beyond the base hardware costs, nanobridges provide a more
   cost-effective solution for high-volume requests.
3. **Flexibility**: Nanobridges can execute multi-chain and off-chain pipelines, support complex
   conditional logic, and even retrieve data from Web2 sources.

This comparison assumes that a network even has traditional bridge support, which is often not the
case for newly launched and up-and-coming chains that must wait up to several months after paying a
very large sum of money for integration. In cases like this, nanobridges are ideal for bootstrapping
connectivity.

## Real-World Example

To better illustrate the potential of nanobridges, let's explore a concrete example in the NFT
space:

The [Rose Portal](https://github.com/enshrinecc/oasis-nft-bridge), now archived after fulfilling its
purpose, was one of the first nanobridge-enabled apps built using Escrin. The Rose Portal allowed
users to migrate NFTs from old sidechain of the Oasis Network to a new one. The network does not
support bridging between sidechains, and traditional bridges had not yet moved in, so nanobridges
were the ideal–and only–solution.

The operation of the Rose Portal nanobridge was simple and straightforward:

1. a NFT holder locks a token on the old chain into the bridge contract
2. the nanobridge worker sends a message to the bridge bridge contract on the new chain to transfer
   a clone of the NFT to the user
3. the nanobridge worker locks the NFT on the old chain into the bridge contract

<figure>
<svg xmlns="http://www.w3.org/2000/svg" class="w-3/4 my-10 mx-auto" viewBox="0 0 431.77 250.03"><g class="graph" transform="translate(4 246.031)"><path fill="var(--vp-c-bg)" stroke="transparent" d="M-4 4v-250.031h431.767V4z"/><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="m139.714-206.08 2.047-.1 2.026-.147 1.997-.196 1.96-.244 1.913-.29 1.861-.339 1.801-.383 1.734-.428 1.66-.472 1.581-.513 1.496-.554 1.406-.593 1.312-.631 1.213-.667 1.112-.7 1.007-.733.9-.763.794-.79.683-.817.574-.84.465-.86.356-.88.248-.897.142-.91.038-.923-.063-.931-.161-.937-.257-.942-.349-.943-.436-.941-.52-.938-.6-.93-.676-.923-.746-.91-.812-.897-.874-.88-.932-.861-.984-.84-1.032-.817-1.076-.79-1.117-.763-1.152-.732-1.184-.7-1.214-.667-1.238-.631-1.261-.593-1.28-.555-1.298-.513-1.312-.471-1.324-.428-1.334-.384-1.343-.338-1.35-.291-1.355-.244-1.36-.196-1.363-.147-1.366-.099-1.366-.05h-1.368l-1.367.05-1.365.099-1.363.147-1.36.196-1.355.244-1.35.291-1.343.338-1.334.384-1.324.428-1.312.47-1.297.514-1.28.555-1.262.593-1.238.63-1.214.667-1.184.7-1.153.733-1.116.763-1.076.79-1.032.817-.984.84-.932.861-.874.88-.812.897-.746.91-.675.922-.6.931-.52.938-.437.941-.349.943-.257.941-.161.938-.064.931.039.922.142.91.248.897.356.88.465.862.574.84.683.816.793.79.9.763 1.008.733 1.112.7 1.213.667 1.312.63 1.406.594 1.496.554 1.58.513 1.66.472 1.735.428 1.8.383 1.861.338 1.914.291 1.96.244 1.997.196 2.026.148 2.047.098 2.06.05h2.065z"/><text x="121.846" y="-219.831" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">User</text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M191.753-153.033H81.489v41.204h110.264z"/><text x="89.555" y="-136.631" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Bridge contract</text><text x="91.116" y="-119.831" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">(old sidechain)</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M136.621-205.934v42.577"/><path stroke="var(--vp-c-text-1)" d="m140.121-163.049-3.5 10-3.5-10z"/><text x="136.621" y="-176.431" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">  1. </text><text x="159.966" y="-176.431" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="11">transferFrom(user,_,id)</text></g><g class="node"><ellipse cx="213.621" cy="-29.416" fill="none" stroke="#ea0" stroke-width="3" rx="62.449" ry="29.331"/><text x="177.445" y="-33.616" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Nanobridge</text><text x="191.071" y="-16.816" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Worker</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M137.546-111.783c1.327 11.105 4.346 24.603 11.075 34.952 4.39 6.751 10.1 12.867 16.312 18.29"/><path stroke="var(--vp-c-text-1)" d="m167.41-61.04 5.6 8.995-9.986-3.54z"/><text y="-81.873" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">2. </text><text x="15.564" y="-81.873" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="11">Transfer(user,_,id)</text><text x="140.943" y="-81.873" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">  </text></g><g class="node"><path fill="none" stroke="var(--vp-c-text-1)" d="M415.63-153.033H301.612v41.204h114.02z"/><text x="310.002" y="-136.631" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">Bridge Contract</text><text x="309.616" y="-119.831" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">(new sidechain)</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="m192.806-57.263-34.526-46.191"/><path stroke="var(--vp-c-text-1)" d="m155.387-101.48-3.184-10.105 8.79 5.915z"/><text x="176.621" y="-82.031" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">4. </text><text x="192.185" y="-82.031" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="11">lock(id)</text></g><g class="edge"><path fill="none" stroke="var(--vp-c-text-1)" d="M248.28-54.04c21.914-15.568 50.238-35.69 72.701-51.65"/><path stroke="var(--vp-c-text-1)" d="m319.099-108.646 10.179-2.939-6.125 8.645z"/><text x="302.621" y="-82.031" fill="var(--vp-c-text-1)" font-family="Helvetica,sans-Serif" font-size="14">3. </text><text x="318.185" y="-82.031" fill="var(--vp-c-text-1)" font-family="Courier,monospace" font-size="11">mintTo(user, id)</text></g></g></svg>
</figure>

In total, the bridging component of the nanobridge worker totaled about
[65 lines](https://github.com/enshrinecc/oasis-nft-bridge/blob/main/worker/src/bridge.ts) of
straightforward and easy-to-audit JavaScript code. This level of convenience is enabled by the
integrity properties of decentralizable TEE that Escrin provides.

The Rose Portal, although a simple application, neatly demonstrates the utility of nanobridges,
especially as a quick and easy solution for an otherwise intractable problem.

## Further Applications

Having supreme flexibility without compromising on security and decentralization, nanobridges can be
the foundation for a variety of next-generation Web3 applications including

- Intents: A trustworthy Escrin worker can seamlessly execute user intents across chains with
  programmable logic and a complete and simultaneous view of all network states. If you wanted to
  make an AI robo-advisor marketplace, this is how you would do it.

- **Real World Assets (RWA)**: Nanobridges can extract data from Web2 sources, including users'
  accounts, thereby establishing a secure link with DeFi.

- **GameFi**: With the ability to run JavaScript and Python programs with integrity and privacy,
  they're ideal for intricate games that rely on private shared state, random elements, and
  immediate interactivity.

## Getting Started with Nanobridges

In summary nanobridges offer a powerful and flexible solution for developers seeking to push the
boundaries of cross-chain communication.

One can begin development with nanobridges by forking the Rose Portal code, running through the
[Escrin tutorial](../tutorial/first-task), or asking around in our
[Discord](https://escrin.org/discord) channel where our dev team will be more than happy to help you
learn more.

> Nanobridges: the full power of a VM with the security of a bridge
