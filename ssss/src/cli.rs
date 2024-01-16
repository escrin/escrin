use clap::{
    ArgAction::{Append, Count},
    Parser, ValueEnum,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, action = Count, default_value_t = 0)]
    pub verbosity: u8,

    #[arg(long, default_value_t = 1056)]
    pub port: u16,

    /// Specify multiple gateways to watch multiple chains or multiple providers per chain.
    #[arg(short, long, action = Append, default_values_t = ["http://127.0.0.1:8545".to_string()])]
    pub gateway: Vec<String>,

    #[arg(short, long, value_enum, default_value = "memory")]
    pub store: Store,

    /// The address of the SsssPermitter (same for all chains).
    #[arg(long, default_value = "0x0000000000000000000000000000000000000000")]
    pub permitter: ethers::types::Address,

    #[arg(long, value_enum, default_value = "dev")]
    pub env: Environment,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum Store {
    Memory,
    Local,
    #[cfg(feature = "aws")]
    Aws,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum Environment {
    Dev,
    Prod,
}
