---
outline: deep
---

# Escrin Runner Reference

The Escrin Runner (stylized `escrin-runner`) is a JavaScript runtime based on the [Cloudflare Workerd](https://github.com/cloudflare/workerd) serverless computing platform, extended with trusted execution and blockchain.
The purpose of the `escrin-runner` is to make it very easy to run trusted programs backed by the security and integrity of smart contracts, hence the name _Smart Workers_.


## Running the Runner

The `escrin-runner` is meant to be fully operable when run on its own in a public, permissioned, or really any kind deployment.
The environment in which it runs determines whether other services will trust it, however.

### Local

The simplest environment in which to run the `escrin-runner` is on your local machine and is ideal for developing your Smart Worker.
You will generally not be able to get TPM attestations on this platform, but that is okay because it is straightforward to begin using a TPM-enabled platform when the time comes, as is described in the following subsection.

#### Docker

The simplest environment within the simplest environment is [Docker](https://www.docker.com/) (or your favorite other [OCI runtime](https://github.com/opencontainers/runtime-spec/blob/main/implementations.md)).

Running the containerized `escrin-runner` is as simple as the following command:

```sh
docker run --rm -it --init -p 1057:1057 ghcr.io/escrin/escrin-runner:latest-local
```

The `--init` is required for the container to respond to signals/keyboard commands like `^C`, which is required if you do not want to manually kill the process by id.

Similarly, during deployment, replace `--rm -it` with `--name escrin-runner -d`.

#### Native

A native local deployment is a good option when you do not enjoy using docker to develop.
This setup is slightly more involved but is still straightforward.

1. Clone `https://github.com/escrin/escrin`
2. Get the latest release of `workerd` from [https://github.com/escrin/workerd](https://github.com/escrin/workerd) for your platform (darwin, linux, or windows).
3. With your copy of `workerd` inside of the root of the Escrin monorepo, run
   ```sh
   workerd compile worker/config/local.capnp > escrin-runner
   ```

Now you can run `escrin-runner --verbose`, which does the same thing as the containerized version.

### Amazon Web Services (AWS)

Running the `escrin-runner` on a properly configured AWS instance provides it with [Nitro Enclave](https://aws.amazon.com/ec2/nitro/nitro-enclaves/) TPM support.

This brief setup guide will tell you what to do, but if you want more of the why or how, please refer to [the AWS docs](https://docs.aws.amazon.com/enclaves/latest/user/getting-started.html).

Start by running an `x86_64` instance with at least 4 vCPUs (2 physical CPUs, one for the host, one for the enclave).
The remaining instructions assume the use of the [Amazon Linux 2023](https://docs.aws.amazon.com/linux/) OS, but other OSes are possible, though not officially supported.

Next, install dependencies:
* `nitro-cli` - the AWS CLI for managing enclaves from the host. Follow the [AWS instructions](https://docs.aws.amazon.com/enclaves/latest/user/nitro-enclave-cli-install.html) for installing it on Amazon Linux 202.
* `socat` - a pipe between different socket types. `dnf install -y socat` should work.
* `microsocks` - lightweight proxy server used by the enclave to connect to the internet. This one needs to be [built from source](https://github.com/rofl0r/microsocks) (or substituted with a different server like [Dante](https://www.inet.no/dante/sslfiles/binaries.html)).

Once you have the dependencies installed, `curl` the latest `escrin-runner.eif` that was built as an artifact by the [CI pipeline](https://github.com/escrin/escrin/actions).

The stage is set.
Now all that must be done is to run the binaries.

```sh
#!/bin/sh

microsocks &
socat VSOCK-LISTEN:1057,fork TCP:127.0.0.1:1080 &
socat TCP4-LISTEN:1057,reuseaddr,fork VSOCK-CONNECT:16:1057 &

nitro-cli run-enclave [--attach-console] \
    --enclave-name escrin-runner
    --eif-path ./escrin-runner.eif \
    --cpu-count 1 \
    --memory 2048 \
    --enclave-cid 16
```

Include or omit the `--attach-console` for debug or production deployments, respectively.

If this script worked without error, you will be able to run `nitro-cli describe-enclaves` and see that your `escrin-runner` enclave is running.

Ensure that the measurements shown by `nitro-cli describe-eif` match what CI says and also what your `NitroEnclavePermitter` is configured to accept.

The enclave can be shut down by running `nitro-cli terminate-enclave --enclave-name escrin-runner`
