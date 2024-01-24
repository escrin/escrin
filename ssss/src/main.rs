#![forbid(unsafe_code)]
#![feature(anonymous_lifetime_in_impl_trait, lazy_cell, stmt_expr_attributes)]

mod api;
mod cli;
mod store;
mod sync;
mod types;
mod utils;
mod verify;

use anyhow::Error;
use tracing::{debug, trace};

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

    trace!("creating store");
    let store = store::create(args.store, args.env).await;

    trace!("running sync tasks");
    sync::run(store, args.gateway.iter(), args.permitter).await?;

    trace!("starting API task");
    let api_task = api::serve(args.port);

    tokio::join!(api_task);

    Ok(())
}
