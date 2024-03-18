<h1><img width="38" height="38" src="https://escrin.org/logo.svg"/>Escrin - SSSS</h1>

<a href="https://escrin.org"><img src="https://img.shields.io/badge/Get_Started-eeaa00?style=for-the-badge"/></a>&nbsp;<a
href="https://escrin.org/discord"><img src="https://img.shields.io/badge/Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white"/></a>&nbsp;<a
href="https://escrin.org/telegram"><img src="https://img.shields.io/badge/Telegram-26A5E4?style=for-the-badge&logo=telegram"/></a>&nbsp;<a
href="https://escrin.org/twitter"><img src="https://img.shields.io/badge/x-000000?style=for-the-badge&logo=x"/></a>&nbsp;<a
href="https://opencollective.com/escrin"><img src="https://img.shields.io/badge/OpenCollective-1F87FF?style=for-the-badge&logo=OpenCollective&logoColor=white"/></a>&nbsp;<a
href="https://www.npmjs.com/package/@escrin/worker"><img src="https://img.shields.io/badge/npm-CB3837?style=for-the-badge&logo=npm&logoColor=white"/></a>

This is the code for the Simple Secret Sharing Server (SSSS), the `s4` SSSS CLI, and deployment
scripts for the SSSS.

### Deploying

To deploy the SSSS, check out the [./deploy](./deploy) directory for different options. The
[SSSS container](https://github.com/escrin/escrin/pkgs/container/ssss) is the canonical deployment
strategy.

To obtain a dev build of the SSSS or `s4`, download the artifacts from a successful run of
[the SSSS CI workflow](https://github.com/escrin/escrin/actions/workflows/ssss.yaml?query=branch%3Amain).

### Developing

The SSSS is a Rust project and can be developed using the
[standard tools](https://www.rust-lang.org/learn/get-started). If you are a Nix user, you can also
run `nix develop` to drop into a fully-configured development shell.
