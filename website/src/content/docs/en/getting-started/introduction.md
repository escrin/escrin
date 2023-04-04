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


![Escrin System Diagram](/escrin.png)

## Use-cases

* DAOs
* DeFi
* DID
* Gaming
* Data Economy
* Healthcare & Genomics
* AI
* MEV
