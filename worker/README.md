<h1><img width="38" height="38" src="../website/public/logo.svg"/>Escrin - Worker</h1>

<a href="https://escrin.org"><img src="https://img.shields.io/badge/Get_Started-eeaa00?style=for-the-badge"/></a>&nbsp;
<a href="https://enshrine.ai/discord"><img src="https://img.shields.io/badge/Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white"/></a>&nbsp;
<a href="https://opencollective.com/escrin"><img src="https://img.shields.io/badge/OpenCollective-1F87FF?style=for-the-badge&logo=OpenCollective&logoColor=white"/></a>&nbsp;
<a href="https://twitter.com/EnshrineCC"><img src="https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white"/></a>&nbsp;
<a href="https://www.npmjs.com/package/@escrin/worker"><img src="https://img.shields.io/badge/npm-CB3837?style=for-the-badge&logo=npm&logoColor=white"/></a>

This is the source code for the `escrin-runner` platform services and the [@escrin/worker](https://www.npmjs.com/package/@escrin/worker) TypeScript library.

### Developing

You can get started by installing `pnpm` and running `pnpm install`.

Once the dependencies have been installed, you can run the [package scripts](https://github.com/escrin/escrin/blob/main/worker/package.json#L18).
The important ones are:

* `pnpm lint` - reports formatting errors and Solidity lints
* `pnpm format` - attempts to fix lints in-place
* `pnpm build` - builds the platform services and the TypeScript library
* `pnpm watch:build` - watches files and rebuilds them when they change

To run the platform services locally, compile [escrin/workerd](https://github.com/escrin/workerd) and run
`workerd serve --verbose config/local.capnp`.

To create a self-contained `escrin-runner`, use `workerd compile config/local.capnp > escrin-runner`, which can then be run without additional arguments (though `--verbose` is often helpful).

### Publishing

To publish this library, bump the version in `package.json`, push that change to `main`, and then run `pnpm publish`.

## Points of Interest

- [src/index.ts](./src/index.ts) - the entrypoint of the [@escrin/worker](https://www.npmjs.com/package/@escrin/worker) TypeScript library
- [workerd_config.capnp](./workerd_config.capnp) - the configuration of the `escrin/workerd` that runs the platform services and sets up the Smart Worker sandbox
- [src/runner.ts](./src/runner.ts) - the entrypoint service of the `escrin-runner` that spawns Smart Workers when requested
- [src/env/iam](./src/env/iam/) - a service linked to Smart Workers that provides decentrized key and identity management.
- [src/env/tpm](./src/env/tpm/) - a service that provides access to the local Trusted Platform Module, which can be used to remotely attest to relying parties

As you work on the code, please feel encouraged to [file issues](https://github.com/escrin/escrin/issues) or participate in [the Discord community](https://enshrine.ai/discord)!
