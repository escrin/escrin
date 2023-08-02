<h1><img width="38" height="38" src="../website/public/logo.svg"/>Escrin - Solidity Library</h1>

<a href="https://escrin.org"><img src="https://img.shields.io/badge/Get_Started-eeaa00?style=for-the-badge"/></a>&nbsp;
<a href="https://enshrine.ai/discord"><img src="https://img.shields.io/badge/Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white"/></a>&nbsp;
<a href="https://opencollective.com/escrin"><img src="https://img.shields.io/badge/OpenCollective-1F87FF?style=for-the-badge&logo=OpenCollective&logoColor=white"/></a>&nbsp;
<a href="https://twitter.com/EnshrineCC"><img src="https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white"/></a>&nbsp;
<a href="https://www.npmjs.com/package/@escrin/evm"><img src="https://img.shields.io/badge/npm-CB3837?style=for-the-badge&logo=npm&logoColor=white"/></a>

This is the source code for the [@escrin/evm](https://www.npmjs.com/package/@escrin/evm) Solidity library.

You can get started by installing `pnpm` and running `pnpm install`.

Once the dependencies have been installed, this is just like any other [Hardhat](https://hardhat.org/hardhat-runner/docs/getting-started) project, meaning that you can do things like

* `pnpm hardhat compile` (or just `pnpm build`) - builds the contracts
* `pnpm hardhat test` (or just `pnpm test`) - tests the contracts

There are also several convenience scripts including
* `pnpm lint` - reports formatting errors and Solidity lints
* `pnpm format` - attempts to fix lints in-place
* `pnpm watch`, `pnpm watch:build` - watches files and runs the appropriate commands when they change

### Publishing

To publish this library, bump the version in `package.json`, push that change to `main`, and then run `pnpm publish`.
