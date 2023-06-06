---
title: "What is Escrin?"
description: "Escrin is a secure decentralized computing network."
---

Escrin is a secure decentralized computing network.

* _secure_ - Algorithms and data are hidden from unauthorized observers.
* _decentralized_ - Algorithms and data can be run on trustworthy hardware anywhere.
* _computing_ - Algorithms are general-purpose and run on any private data.
* _network_ - Algorithms, data, and hardware can be contributed by anyone.

If you have ever wished for a trusted third party that could run some private computation for you, Escrin is the answer.

Escrin focuses on Web3 applications and integrates closely with Ethereum and similar networks to provide a seamless secure off-chain computing experience.

## General Workflow

1. Package the algorithm into an OCI (Docker) image in the usual way.
2. Use Escrin to enclave the application image.
3. Push the enclaved application image.
4. Submit a job that runs the image on secure hardware.

* Inputs and outputs are end-to-end encrypted.
* Almost any container image is supported
* Transparency and integrity are guaranteed by decentralized ledgers.
* Confidentiality and encryption keys are guarded by decentralized secret sharing networks.
* Convenient integration with Ethereum tools like Hardhat and MetaMask.

## Key Concepts

### Decentralized Computing

Decentralized computing refers to the distribution of computing resources across a network of computers, rather than relying on a single centralized entity.
This provides a number of benefits, including improved fault tolerance, greater scalability, and increased security.
In Escrin, decentralized computing allows algorithms and data to be run on trustworthy hardware anywhere, and allows anyone to contribute to the network.

### Trusted Execution Environments

A trusted execution environment (TEE) is a secure hardware environment that provides a high level of protection for executing code and managing data.
In Escrin, TEEs are used to protect algorithms and data from unauthorized observers, ensuring that inputs and outputs are end-to-end encrypted.
These hardware enclaves provide strong privacy guarantees and enable fully autonomous computation.

### Decentralized Key Management

Decentralized Key Management refers to the use of smart contracts to trigger key release from decentralized key management networks that hold secrets using techniques like secret sharing and threshold encryption.
Escrin uses decentralized key management to ensure that keys are always available, but only to the authorized party or hardware enclave.
This approach provides a high level of security and privacy, even in a decentralized computing environment.

## How It Works


<svg width="87%" viewBox="0.00 0.00 526.18 527.80" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" class="mx-auto">
<g id="graph0" class="graph" transform="scale(1 1) rotate(0) translate(4 523.8)">
<title>%0</title>
<polygon fill="var(--theme-bg)" stroke="transparent" points="-4,4 -4,-523.8 522.1806,-523.8 522.1806,4 -4,4"/>
<g id="clust2" class="cluster">
<title>cluster_runner</title>
<polygon fill="var(--theme-bg)" stroke="var(--theme-text)" points="79.892,-79.8 79.892,-207.6 252.892,-207.6 252.892,-79.8 79.892,-79.8"/>
<text text-anchor="start" x="87.8233" y="-191" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">TEE-having Task Runner</text>
</g>
<g id="clust3" class="cluster">
<title>cluster_escrin</title>
<polygon fill="var(--theme-bg)" stroke="#eeaa00" stroke-width="2" points="97.892,-87.8 97.892,-174.8 233.892,-174.8 233.892,-87.8 97.892,-87.8"/>
<text text-anchor="start" x="146.4488" y="-159.2" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">Escrin</text>
<text text-anchor="start" x="138.892" y="-147" font-family="Helvetica,sans-Serif" font-size="12.00" fill="var(--theme-text)">(in a TEE)</text>
</g>
<g id="clust1" class="cluster">
<title>cluster_ledger</title>
<polygon fill="var(--theme-bg)" stroke="var(--theme-text)" stroke-width="2" points="42.892,-349 42.892,-511.8 358.892,-511.8 358.892,-349 42.892,-349"/>
<text text-anchor="start" x="134.3766" y="-495.2" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">Decentralized Ledger</text>
</g>
<!-- orchestrator -->
<g id="node1" class="node">
<title>orchestrator</title>
<polygon fill="none" stroke="var(--theme-text)" points="0,-250.9 0,-286.9 93.784,-286.9 93.784,-250.9 0,-250.9"/>
<text text-anchor="start" x="8" y="-264.7" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">Orchestrator</text>
</g>
<!-- agent -->
<g id="node2" class="node">
<title>agent</title>
<polygon fill="none" stroke="var(--theme-text)" points="59.892,-445 59.892,-475 342.892,-475 342.892,-445 59.892,-445"/>
<text text-anchor="start" x="154.7111" y="-455.4" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">Agent Contract</text>
<polygon fill="none" stroke="var(--theme-text)" points="59.892,-417 59.892,-445 342.892,-445 342.892,-417 59.892,-417"/>
<text text-anchor="start" x="66.4814" y="-428.2" font-family="Helvetica,sans-Serif" font-size="12.00" fill="#eeaa00">(escrin)  </text>
<text text-anchor="start" x="113.1398" y="-428.2" font-family="Courier,monospace" font-size="12.00" fill="var(--theme-text)">acceptTasks(tasks,report,proof)</text>
<polygon fill="none" stroke="var(--theme-text)" points="59.892,-389 59.892,-417 342.892,-417 342.892,-389 59.892,-389"/>
<text text-anchor="start" x="158.1992" y="-399.2" font-family="Courier,monospace" font-size="12.00" fill="var(--theme-text)">public state</text>
<polygon fill="none" stroke="var(--theme-text)" points="59.892,-361 59.892,-389 342.892,-389 342.892,-361 59.892,-361"/>
<text text-anchor="start" x="84.4784" y="-372.2" font-family="Helvetica,sans-Serif" font-size="12.00" fill="#eeaa00">(escrin)  </text>
<text text-anchor="start" x="131.1368" y="-372.2" font-family="Courier,monospace" font-size="12.00" fill="var(--theme-text)">approveSecret(credentials)</text>
</g>
<!-- orchestrator&#45;&gt;agent -->
<g id="edge1" class="edge">
<title>orchestrator-&gt;agent:state</title>
<path fill="none" stroke="var(--theme-text)" stroke-dasharray="5,2" d="M41.3027,-286.9853C31.803,-320.5862 16.5794,-390.2597 49.7898,-401.4644"/>
<polygon fill="var(--theme-text)" stroke="var(--theme-text)" points="49.4795,-404.9573 59.892,-403 50.5316,-398.0368 49.4795,-404.9573"/>
<text text-anchor="start" x="36.892" y="-326.4" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">1. detect tasks</text>
<text text-anchor="start" x="52.0729" y="-309.6" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">&amp; policies</text>
</g>
<!-- tasks -->
<g id="node4" class="node">
<title>tasks</title>
<polygon fill="none" stroke="var(--theme-text)" points="105.7209,-96.3 105.7209,-132.3 226.0631,-132.3 226.0631,-96.3 105.7209,-96.3"/>
<text text-anchor="start" x="113.7209" y="-110.1" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">Task</text>
<polyline fill="none" stroke="var(--theme-text)" points="152.0561,-96.3 152.0561,-132.3 "/>
<text text-anchor="start" x="160.0561" y="-110.1" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">Task</text>
<polyline fill="none" stroke="var(--theme-text)" points="198.3913,-96.3 198.3913,-132.3 "/>
<text text-anchor="start" x="206.3913" y="-110.1" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">...</text>
</g>
<!-- orchestrator&#45;&gt;tasks -->
<g id="edge2" class="edge">
<title>orchestrator-&gt;tasks</title>
<path fill="none" stroke="var(--theme-text)" d="M43.5457,-250.7916C42.4195,-240.0683 42.5318,-226.4939 47.6368,-215.6 53.8232,-202.3984 62.6513,-190.2633 72.7335,-179.3313"/>
<polygon fill="var(--theme-text)" stroke="var(--theme-text)" points="75.4397,-181.5685 79.8969,-171.9568 70.4186,-176.6912 75.4397,-181.5685"/>
<text text-anchor="start" x="47.892" y="-219.8" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">2. dispatch tasks</text>
</g>
<!-- agent&#45;&gt;tasks -->
<g id="edge3" class="edge">
<title>agent:approveSecret-&gt;tasks</title>
<path fill="none" stroke="#eeaa00" d="M201.2619,-350.9128C197.8088,-324.8508 181.1088,-317.9395 174.8584,-287.4 167.3051,-250.4947 164.9948,-208.2916 164.6061,-174.7965"/>
<polygon fill="#eeaa00" stroke="#eeaa00" points="197.7753,-351.2377 201.892,-361 204.7616,-350.8012 197.7753,-351.2377"/>
<text text-anchor="start" x="173.892" y="-264.7" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">4. request secret  </text>
</g>
<!-- km -->
<g id="node5" class="node">
<title>km</title>
<polygon fill="none" stroke="var(--theme-text)" stroke-width="2" points="296.2651,-250.9 296.2651,-286.9 493.5189,-286.9 493.5189,-250.9 296.2651,-250.9"/>
<text text-anchor="start" x="304.2651" y="-264.7" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">Secret Management Network</text>
</g>
<!-- agent&#45;&gt;km -->
<g id="edge8" class="edge">
<title>agent:state-&gt;km</title>
<path fill="none" stroke="var(--theme-text)" stroke-dasharray="5,2" d="M352.868,-401.802C392.6759,-391.7462 396.3131,-320.9437 395.7176,-286.9853"/>
<polygon fill="var(--theme-text)" stroke="var(--theme-text)" points="352.4033,-398.3326 342.892,-403 353.238,-405.2826 352.4033,-398.3326"/>
<text text-anchor="start" x="237.7202" y="-307.2251" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">6. check secret approval</text>
</g>
<!-- storage -->
<g id="node3" class="node">
<title>storage</title>
<polygon fill="none" stroke="var(--theme-text)" points="105.7728,-.5 105.7728,-36.5 226.0112,-36.5 226.0112,-.5 105.7728,-.5"/>
<text text-anchor="start" x="113.7728" y="-14.3" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">Storage Network</text>
</g>
<!-- tasks&#45;&gt;agent -->
<g id="edge6" class="edge">
<title>tasks-&gt;agent:acceptTasks</title>
<path fill="none" stroke="#eeaa00" d="M233.8884,-131.3836C323.8209,-155.6024 474.3212,-202.3432 502.892,-250.4 555.9802,-339.6955 456.6921,-425.4698 352.9414,-430.7435"/>
<polygon fill="#eeaa00" stroke="#eeaa00" points="352.7994,-427.2459 342.892,-431 352.9781,-434.2436 352.7994,-427.2459"/>
<text text-anchor="start" x="233.8884" y="-135.5836" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">                     8. submit results</text>
</g>
<!-- tasks&#45;&gt;storage -->
<g id="edge4" class="edge">
<title>tasks-&gt;storage</title>
<path fill="none" stroke="#eeaa00" stroke-dasharray="5,2" d="M97.8922,-94.9687C76.9582,-85.3615 62.118,-72.0961 73.735,-55 79.4656,-46.5666 87.4801,-40.0818 96.4144,-35.0953"/>
<polygon fill="#eeaa00" stroke="#eeaa00" points="98.1652,-38.135 105.6047,-30.5916 95.0848,-31.8492 98.1652,-38.135"/>
<text text-anchor="start" x="73.892" y="-59.2" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">3. fetch program &amp; inputs  </text>
</g>
<!-- tasks&#45;&gt;storage -->
<g id="edge5" class="edge">
<title>tasks-&gt;storage</title>
<path fill="none" stroke="#eeaa00" d="M226.6781,-87.8016C238.7361,-78.68 245.6047,-67.5571 237.892,-55 235.0761,-50.4155 231.4856,-46.3851 227.4161,-42.8458"/>
<polygon fill="#eeaa00" stroke="#eeaa00" points="229.1865,-39.8 219.0775,-36.6282 225.0021,-45.4118 229.1865,-39.8"/>
<text text-anchor="start" x="240.892" y="-59.2" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)">  7. store outputs</text>
</g>
<!-- km&#45;&gt;tasks -->
<g id="edge7" class="edge">
<title>km-&gt;tasks</title>
<path fill="none" stroke="#eeaa00" stroke-dasharray="5,2" d="M359.7246,-245.1582C326.3326,-222.6149 275.2364,-188.1194 233.8878,-160.2046"/>
<polygon fill="#eeaa00" stroke="#eeaa00" points="357.9281,-248.1683 368.1746,-250.8628 361.8449,-242.3666 357.9281,-248.1683"/>
<text text-anchor="start" x="338.892" y="-219.8" font-family="Helvetica,sans-Serif" font-size="14.00" fill="var(--theme-text)"> 5.  fetch secret</text>
</g>
</g>
</svg>

## Use-cases

* DAOs
* DeFi
* DID
* Gaming
* Data Economy
* Healthcare & Genomics
* AI
* MEV
