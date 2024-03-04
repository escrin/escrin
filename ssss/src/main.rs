#![forbid(unsafe_code)]
#![feature(anonymous_lifetime_in_impl_trait, lazy_cell, stmt_expr_attributes)]

mod api;
mod cli;
mod sync;
mod verify;

use std::collections::HashMap;

use anyhow::Result;
use ethers::middleware::MiddlewareBuilder as _;
use ssss::{eth, store, types, utils};
use tracing::{debug, trace};

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_env_filter(match args.verbosity {
            0 => "ssss=warn",
            1 => "ssss=info",
            2 => "ssss=debug",
            _ => "ssss=trace",
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
    let permitters: HashMap<_, _> = args.permitters.into_iter().collect();
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
    let signer = ethers::signers::LocalWallet::new(&mut rand::thread_rng());
    let sssss: Vec<_> = providers
        .into_iter()
        .filter_map(|(chain, provider)| {
            let permitter = permitters.get(&chain)?;
            Some(eth::SsssPermitter::new(
                chain,
                *permitter,
                provider.with_signer(signer.clone()),
            ))
        })
        .collect();

    trace!("creating store");
    let store = store::create(args.store, args.env).await;

    trace!("running sync tasks");
    sync::run(store.clone(), sssss.iter().cloned()).await?;

    trace!("starting API task");
    let api_task = api::serve(store, sssss.into_iter(), args.host);

    tokio::join!(api_task);

    Ok(())
}
