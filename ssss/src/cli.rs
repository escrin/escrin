use clap::{
    ArgAction::{Append, Count},
    Parser, ValueEnum,
};
use url::Url;

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

    #[arg(short, long, value_enum, default_value = "local")]
    pub backend: Backend,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum Backend {
    Local,
    #[cfg(feature = "aws")]
    Aws,
}
