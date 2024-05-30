use clap::{
    ArgAction::{Append, Count},
    Parser, ValueHint,
};

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

    #[arg(short, long, value_enum, default_value = "dev")]
    pub env: crate::backend::Environment,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}
