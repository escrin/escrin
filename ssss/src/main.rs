#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

mod api;
mod cli;
mod verify;

use std::collections::HashMap;

use anyhow::Result;
use ssss::{backend, eth, types};
use tracing::{debug, trace};

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_env_filter(match args.verbosity {
            0 => "ssss=warn,tower_http=warn",
            1 => "ssss=info,tower_http=info",
            2 => "ssss=debug,tower_http=debug",
            _ => "ssss=trace,tower_http=trace",
        })
        .with_target(true);
    if cfg!(not(debug_assertions)) {
        subscriber.json().with_ansi(false).init();
    } else {
        subscriber.without_time().init();
    }

    debug!(args = ?args, "loaded config");

    trace!("loading providers");
    let providers = eth::providers(args.gateway.iter()).await?;
    let permitters: HashMap<_, _> = args.permitter.into_iter().collect();
    let missing_providers: Vec<_> = permitters
        .keys()
        .filter(|&chain| (!providers.contains_key(chain)))
        .map(|chain| chain.to_string())
        .collect();
    if !missing_providers.is_empty() {
        anyhow::bail!(
            "missing providers for chains {}",
            missing_providers.join(", ")
        );
    }

    trace!("creating store");
    let store = backend::create(args.store, args.env, &args.host).await?;

    trace!("starting API task");
    api::serve(store, providers, args.host).await;

    Ok(())
}
