# CREATE2 salt generator

## Usage

### 0. Compile

```
cargo build --release
```

### 1. Generate a salt for the IdentityRegistry

```sh
target/release/salter ../abi/IdentityRegistry.json
```

The program will print out salts with low weights (high number of zeros) as it discovers them.
Take the salt that yields the lowest weight and then generate the SsssPermitter address.

### 2. Generate a salt for the SsssPermitter

```sh
target/release/salter ../abi/SsssPermitter.json $(cast abi-encode 'constructor(address)' <the address from step 1>)
```

Now you have the salt for the SsssPermitter.

### 3. Update the salts in `Deploy.sol`

Paste the best salts into the `{salt: ...}new <contract>(...);` and then redeploy.

Currently the best salts have weight:
* IdentityRegistry: 14
* SsssPermitter: 14

### 4. Update the contract addresses in the s4 CLI defaults

In `:/ssss/s4/src/cli.rs` there is a definition that looks like

```rust
#[derive(Clone, Debug, clap::Args)]
pub struct Permitter {
    /// The address of the SsssPermitter.
    #[arg(
        short,
        long,
        default_value = "0xold address"
    )]
    pub permitter: Address,
}
```

Paste the new SsssPermitter address in the location containing the previous address.
And then also update the default identity id based on the results of running the `SetupTestEnv` forge script.

If you don't want to do either of these, just paste your seeds in chat.
