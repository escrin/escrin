<h1><img width="38" height="38" src="../website/public/logo.svg"/>Escrin - Solidity Library</h1>

<a href="https://escrin.org"><img src="https://img.shields.io/badge/Get_Started-eeaa00?style=for-the-badge"/></a>&nbsp;
<a href="https://enshrine.ai/discord"><img src="https://img.shields.io/badge/Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white"/></a>&nbsp;
<a href="https://opencollective.com/escrin"><img src="https://img.shields.io/badge/OpenCollective-1F87FF?style=for-the-badge&logo=OpenCollective&logoColor=white"/></a>&nbsp;
<a href="https://twitter.com/EnshrineCC"><img src="https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white"/></a>&nbsp;
<a href="https://www.npmjs.com/package/@escrin/evm"><img src="https://img.shields.io/badge/npm-CB3837?style=for-the-badge&logo=npm&logoColor=white"/></a>

This is the source code for the [@escrin/evm](https://www.npmjs.com/package/@escrin/evm) Solidity library.

You can get started by installing `forge` (Foundry) and running `forge install`.

Once the dependencies have been installed, this is just like any other [Foundry](https://book.getfoundry.sh/projects/working-on-an-existing-project) project, meaning that you can do things like

* `forge compile [--watch]` - builds the contracts
* `forge test` - tests the contracts

The `Makefile` also offers conveniences such as
* `make lint` - checks contracts for formatting issues
* `make format` - applies consistent formatting to contracts
* `make test` - runs tests

### Publishing

To publish this library, bump the version in `package.json`, push that change to `main`, and then run `pnpm publish`.
