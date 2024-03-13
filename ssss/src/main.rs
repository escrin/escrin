#![forbid(unsafe_code)]
#![feature(anonymous_lifetime_in_impl_trait, lazy_cell, stmt_expr_attributes)]

mod api;
mod cli;
mod identity;
mod sync;
mod verify;

use std::collections::HashMap;

use anyhow::Result;
use ethers::middleware::MiddlewareBuilder as _;
use ssss::{
    eth,
    store::{self, Store as _},
    types, utils,
};
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
            Some(eth::SsssHub::new(
                chain,
                *permitter,
                provider.with_signer(signer.clone()),
            ))
        })
        .collect();

    trace!("creating store");
    let store = store::create(args.store, args.env).await;

    let identity_key_id = types::KeyId {
        name: "ssss-identity".into(),
        identity: types::IdentityLocator {
            chain: 0,
            registry: Default::default(),
            id: types::IdentityId(Default::default()),
        },
        version: 1,
    };
    let identity_key = match store.get_key(identity_key_id.clone()).await? {
        Some(k) => p384::SecretKey::from_slice(&k.into_vec())?,
        None => {
            let identity_key = p384::SecretKey::random(&mut rand::thread_rng());
            store
                .put_key(identity_key_id, identity_key.to_bytes().to_vec().into())
                .await?;
            identity_key
        }
    };
    let identity = identity::Identity::persistent(identity_key);
    let identity_pub_jwk = identity.public_key().to_jwk();

    trace!("running sync tasks");
    sync::run(store.clone(), sssss.iter().cloned(), identity).await?;

    trace!("starting API task");
    let api_task = api::serve(store, sssss.into_iter(), args.host, identity_pub_jwk);

    tokio::join!(api_task);

    Ok(())
}
