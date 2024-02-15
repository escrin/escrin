use clap::{
    ArgAction::{Append, Count},
    Parser, ValueHint,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, action = Count, default_value_t = 0)]
    pub verbosity: u8,

    #[arg(long, default_value_t = 1056)]
    pub port: u16,

    /// Specify multiple gateways to watch multiple chains or multiple providers per chain.
    #[arg(short, long, action = Append, default_values_t = ["http://127.0.0.1:8545".to_string()], value_hint = ValueHint::Url)]
    pub gateway: Vec<String>,

    #[arg(short, long, value_enum, default_value = "memory")]
    pub store: crate::store::StoreKind,

    /// The address of the SsssPermitter.
    #[arg(long, default_value = "0x0000000000000000000000000000000000000000")]
    pub permitter: ethers::types::Address,

    #[arg(long, value_enum, default_value = "dev")]
    pub env: crate::store::Environment,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}
