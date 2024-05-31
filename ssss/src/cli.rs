use clap::{
    builder::TypedValueParser,
    ArgAction::{Append, Count},
    Parser, ValueHint,
};
use ethers::types::Address;

use crate::types::ChainId;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, action = Count, default_value_t = 0)]
    pub verbosity: u8,

    /// The domain at which this SSSS's API is expected to be served.
    #[arg(long, default_value = "127.0.0.1:1075")]
    pub host: axum::http::uri::Authority,

    /// Web3 gateway(s) to watch. Multiple gateways per chain provides quorum.
    #[arg(short, long, action = Append, default_values = ["http://127.0.0.1:8545"], value_hint = ValueHint::Url)]
    pub gateway: Vec<String>,

    #[arg(short, long, value_enum, default_value = "memory")]
    pub store: crate::backend::StoreKind,

    /// The SsssPermitter address per chain.
    #[arg(short, long = "permitter", value_parser = permitters_parser(), action = Append, default_values = [
        "31337=0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"
    ])]
    pub permitter: Vec<(ChainId, Address)>,

    #[arg(short, long, value_enum, default_value = "dev")]
    pub env: crate::backend::Environment,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}

fn permitters_parser() -> impl TypedValueParser {
    clap::builder::StringValueParser::default().try_map(|v| {
        let err = "permitter argument must have format <chain_id>=<permitter_address>";
        match v.split_once('=') {
            Some((chain_str, addr_str)) => {
                let chain: u64 = chain_str.parse().map_err(|_| err)?;
                let addr: Address = addr_str.parse().map_err(|_| err)?;
                Ok((chain, addr))
            }
            _ => Err(err),
        }
    })
}
