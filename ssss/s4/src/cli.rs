use clap::{
    ArgAction::{Append, Count},
    Parser, Subcommand, ValueHint,
};
use ethers::types::{Address, Bytes, H256};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, action = Count, default_value_t = 0)]
    pub verbosity: u8,

    #[command(subcommand)]
    pub command: Command,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    SetPolicy {
        #[command(flatten)]
        args: WritePermitterArgs,

        /// The file from which to read the JSON policy document or stdin if not specified.
        #[arg(value_hint = ValueHint::FilePath)]
        policy_path: Option<String>,
    },
    SignOmniKeyRequest {
        #[arg(long, default_value_t = 31337)]
        chain: u64,

        #[arg(long, default_value = "127.0.0.1:1075")]
        ssss: String,

        #[arg(long, default_value = "0x5FbDB2315678afecb367f032d93F642f64180aa3")]
        registry: Address,

        #[arg(
            long,
            default_value = "0xb725694d2cfafceaf7dbbf2b29ce7f8879ba0c23451f19aee5db8722812e3409"
        )]
        identity: H256,

        #[arg(long, default_value_t = 1)]
        share_version: u64,

        #[arg(
            long,
            default_value = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        )]
        private_key: ethers::signers::LocalWallet,
    },
    Deal {
        #[command(flatten)]
        args: WritePermitterArgs,

        /// The secret to deal. A random one is generated if not provided.
        secret: Option<Bytes>,

        /// The version of the secret to deal. SSSSs only accept only new versions.
        #[arg(long)]
        version: u64,

        #[arg(long = "ssss", action = Append, default_values = ["http://127.0.0.1:1075"])]
        sssss: Vec<String>,

        /// The threshold of SSSSs that must return correct shares for the secret to be reconstructed.
        /// If between 0 and 1, represents a percentage. Greater than 1 represents an absolute number.
        #[arg(short, long, default_value_t = 2.0/3.0)]
        threshold: f64,
    },
}

#[derive(Clone, Debug, clap::Args)]
pub struct WritePermitterArgs {
    /// The Web3 gateway URL.
    #[arg(short, long, default_value = "http://127.0.0.1:8545", value_hint = ValueHint::Url)]
    pub gateway: String,

    /// The address of the SsssPermitter.
    #[arg(long, default_value = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512")]
    pub permitter: Address,

    #[arg(
        long,
        default_value = "0xb725694d2cfafceaf7dbbf2b29ce7f8879ba0c23451f19aee5db8722812e3409"
    )]
    pub identity: H256,

    #[arg(
        long,
        default_value = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    )]
    pub private_key: ethers::signers::LocalWallet,
}
