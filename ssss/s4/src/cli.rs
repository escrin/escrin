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
    /// Obtains the signer address of the SSSS.
    GetSsssSigner {
        #[command(flatten)]
        ssss: Ssss,
    },
    /// Write a new policy to the blockchain and send it to the SSSSs.
    SetPolicy {
        #[command(flatten)]
        wp: WritePermitterArgs,

        #[command(flatten)]
        il: IdentityLocatorArgs,

        #[arg(short, long, value_enum, required = true)]
        verifier: PolicyVerifier,

        /// The file from which to read the JSON policy document or stdin if not specified.
        #[arg(value_hint = ValueHint::FilePath)]
        policy_path: Option<String>,

        #[command(flatten)]
        sssss: Sssss,
    },
    SetApprovers {
        #[command(flatten)]
        wp: WritePermitterArgs,

        #[command(flatten)]
        identity: IdentityId,

        #[command(flatten)]
        sssss: Sssss,

        #[command(flatten)]
        threshold: Threshold,
    },
    /// Acquire an Escrin identity from a quorum of SSSSs and post the approvals to the permitter.
    AcquireIdentity {
        #[command(flatten)]
        sssss: Sssss,

        #[command(flatten)]
        il: IdentityLocatorArgs,

        #[command(flatten)]
        wp: WritePermitterArgs,

        #[arg(short, long, default_value_t = 24 * 60 * 60)]
        duration: u64,

        #[arg(short, long, default_value = "0x")]
        authorization: Bytes,

        #[arg(short, long, default_value = "0x")]
        context: Bytes,

        #[arg(short, long, required = true)]
        recipient: Address,

        #[command(flatten)]
        threshold: Threshold,
    },
    /// Split a secret into shares and deal it to the requested SSSSs.
    Deal {
        #[command(flatten)]
        sssss: Sssss,

        #[command(flatten)]
        il: IdentityLocatorArgs,

        /// The secret name.
        #[arg(short, long, default_value = "omni")]
        name: String,

        /// The secret to deal. A random one is generated if not provided.
        secret: Option<Bytes>,

        #[command(flatten)]
        version: ShareVersion,

        #[command(flatten)]
        threshold: Threshold,

        #[command(flatten)]
        wallet: Wallet,
    },
    /// Reconstructs a secret from shares requested by the requested SSSSs.
    ///
    /// Requires that the requester wallet possesses the appropriate identity.
    Reconstruct {
        #[command(flatten)]
        il: IdentityLocatorArgs,

        /// The secret name.
        #[arg(short, long, default_value = "omni")]
        name: String,

        #[command(flatten)]
        version: ShareVersion,

        #[command(flatten)]
        sssss: Sssss,

        #[command(flatten)]
        wallet: Wallet,
    },
}

#[derive(Clone, Debug, clap::Args)]
pub struct Ssss {
    /// The SSSS URL.
    #[arg(short, long, default_value = "http://127.0.0.1:1075")]
    pub ssss: url::Url,
}

#[derive(Clone, Copy, Debug, clap::Args)]
pub struct ShareVersion {
    /// The version of the secret to deal. SSSSs only accept only new versions.
    #[arg(short, long)]
    pub version: u64,
}

#[derive(Clone, Debug, clap::Args)]
pub struct Threshold {
    /// The threshold of SSSSs that must return correct shares for the secret to be reconstructed.
    /// If between 0 and 1, represents a percentage. Greater than 1 represents an absolute number.
    #[arg(short, long, default_value_t = 0.666666)]
    pub threshold: f64,
}

#[derive(Clone, Debug, clap::Args)]
pub struct Wallet {
    #[arg(
        short = 'k',
        long,
        default_value = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    )]
    pub private_key: ethers::signers::LocalWallet,
}

#[derive(Clone, Copy, Debug, clap::Args)]
pub struct IdentityId {
    #[arg(
        short,
        long,
        default_value = "0xb725694d2cfafceaf7dbbf2b29ce7f8879ba0c23451f19aee5db8722812e3409"
    )]
    pub identity: H256,
}

#[derive(Clone, Debug, clap::Args)]
pub struct Permitter {
    /// The address of the SsssPermitter.
    #[arg(
        short,
        long,
        default_value = "0xce00d95c90C17D1d00D400580000F000ABac00B0"
    )]
    pub permitter: Address,
}

#[derive(Clone, Copy, Debug, clap::Args)]
pub struct IdentityLocatorArgs {
    #[arg(long, default_value_t = 31337)]
    pub chain: u64,

    #[arg(long, default_value = "0x5FbDB2315678afecb367f032d93F642f64180aa3")]
    pub registry: Address,

    #[command(flatten)]
    pub identity: IdentityId,
}

impl From<IdentityLocatorArgs> for ssss::types::IdentityLocator {
    fn from(il: IdentityLocatorArgs) -> Self {
        Self {
            chain: il.chain,
            registry: il.registry,
            id: (*il.identity).into(),
        }
    }
}

#[derive(Clone, Debug, clap::Args)]
pub struct Sssss {
    #[arg(short, long = "ssss", action = Append, default_values = ["http://127.0.0.1:1075"])]
    pub sssss: Vec<url::Url>,
}

#[derive(Clone, Debug, clap::Args)]
pub struct WritePermitterArgs {
    /// The Web3 gateway URL.
    #[arg(short, long, default_value = "http://127.0.0.1:8545", value_hint = ValueHint::Url)]
    pub gateway: String,

    #[command(flatten)]
    pub permitter: Permitter,

    #[command(flatten)]
    pub wallet: Wallet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum PolicyVerifier {
    Mock,
    Nitro,
}

impl std::fmt::Display for PolicyVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Mock => "mock",
                Self::Nitro => "nitro",
            }
        )
    }
}

macro_rules! impl_deref_for_args {
    ($( $ty:ty { $prop:ident: $target:ty }),+ $(,)? ) => {
        $(
            impl std::ops::Deref for $ty {
                type Target=$target;

                fn deref(&self) -> &Self::Target {
                    &self.$prop
                }
            }
        )+
    };
}

impl_deref_for_args! {
    Permitter { permitter: Address },
    Ssss { ssss: url::Url },
    Sssss { sssss: Vec<url::Url> },
    Wallet { private_key: ethers::signers::LocalWallet },
    IdentityId { identity: H256 },
    Threshold { threshold: f64 },
    ShareVersion { version: u64 },
}
