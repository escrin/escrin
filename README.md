<h1><img width="38" height="38" src="./website/public/logo.svg"/>Escrin</h1>

<a href="https://escrin.org"><img src="https://img.shields.io/badge/Get_Started-eeaa00?style=for-the-badge"/></a>&nbsp;
<a href="https://enshrine.ai/discord"><img src="https://img.shields.io/badge/Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white"/></a>&nbsp;
<a href="https://opencollective.com/escrin"><img src="https://img.shields.io/badge/OpenCollective-1F87FF?style=for-the-badge&logo=OpenCollective&logoColor=white"/></a>&nbsp;
<a href="https://twitter.com/EnshrineCC"><img src="https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white"/></a>&nbsp;
<a href="https://www.npmjs.com/package/@escrin/worker"><img src="https://img.shields.io/badge/npm-CB3837?style=for-the-badge&logo=npm&logoColor=white"/></a>

Escrin is a Smart Worker runtime that gives smart contracts the ability to privately interact with private off-chain data, and push the results back on chain.

## Developer Orientation

This is the Escrin monorepo where you can find everything related to Escrin.

If you are a developer who wants to get started building on Escrin, the [dev docs](https://escrin.org/docs) are the best place to start.  
If you are a developer who wants to get started building Escrin itself, this is the place to be!

The main points of interest in this repository are in the following directories:

- [evm](./evm) - The source of the `@escrin/contracts` Solidity library that facilitates key management and task acceptance.
- [worker](./worker) - The library and services powering the `escrin-runner` and its contained workers.
- [website](./website) - The source of https://escrin.org.

There is also the [escrin/workerd](https://github.com/escrin/workerd) repository which contains the JavaScript VM that hosts the `escrin-runner` service and Smart Workers.

Please feel encouraged to [file issues](https://github.com/escrin/escrin/issues) or participate in [the Discord community](https://enshrine.ai/discord)!
