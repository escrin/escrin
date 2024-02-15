use clap::{ArgAction::Count, Parser, Subcommand, ValueHint};
use ethers::types::{Address, H256};

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
        args: PolicyArgs,

        /// The file from which to read the JSON policy document or stdin if not specified.
        #[arg(long = "policy", value_hint = ValueHint::FilePath)]
        policy_path: Option<String>,

        #[arg(long, required = true)]
        private_key: ethers::signers::LocalWallet,
    },
}

#[derive(Clone, Debug, clap::Args)]
pub struct PolicyArgs {
    /// The Web3 gateway URL.
    #[arg(short, long, default_value = "http://127.0.0.1:8545", value_hint = ValueHint::Url)]
    pub gateway: String,

    /// The address of the SsssPermitter.
    #[arg(long, default_value = "0x0000000000000000000000000000000000000000")]
    pub permitter: Address,

    #[arg(long, required = true)]
    pub identity: H256,
}
