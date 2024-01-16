#![forbid(unsafe_code)]
#![feature(anonymous_lifetime_in_impl_trait, stmt_expr_attributes)]
#![allow(unused)]

mod api;
mod cli;
mod eth;
mod sstore;

use anyhow::Error;
use tracing::{debug, Level};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = cli::Args::parse();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env());
    let subscriber = match dbg!(args.verbosity) {
        0 => subscriber.with_env_filter("ssss=warn"),
        1 => subscriber.with_env_filter("ssss=info"),
        2 => subscriber.with_env_filter("ssss=debug"),
        _ => subscriber.with_env_filter("ssss=trace"),
    }
    .with_target(true);
    if cfg!(not(debug_assertions)) {
        subscriber.json().with_ansi(false).init();
    } else {
        subscriber.without_time().init();
    }

    debug!(args = ?args, "loaded config");

    let sstore_client = sstore::create(args.backend, args.env).await;
    let providers = eth::providers(args.gateway.iter()).await?;

    let api_task = api::serve(args.port);

    tokio::join!(api_task);

    Ok(())
}
