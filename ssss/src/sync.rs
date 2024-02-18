use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use ethers::middleware::Middleware;
use futures::stream::StreamExt as _;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, trace, warn};

use crate::{
    eth,
    store::Store,
    types::{ChainId, ChainState, ChainStateUpdate, PermitterLocator},
    utils::retry,
};

#[tracing::instrument(skip_all)]
pub async fn run<M: Middleware + 'static>(
    store: impl Store + 'static,
    sssss: impl Iterator<Item = eth::SsssPermitter<M>>,
) -> Result<(), eth::Error<eth::Provider>> {
    trace!("collating providers");

    for ssss in sssss {
        let store = store.clone();
        let chain = ssss.chain;
        trace!("launching task for chain {chain}");
        tokio::spawn(async move {
            let ssss = &ssss;
            loop {
                match sync_chain(chain, ssss, &store).await {
                    Ok(_) => warn!("sync task for chain {chain} unexpectedly exited"),
                    Err(e) => error!("sync task for chain {chain} exited with error: {e}"),
                }
                sleep(Duration::from_millis(1000)).await;
            }
        });
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn sync_chain<M: Middleware + 'static, S: Store + 'static>(
    chain_id: ChainId,
    permitter: &eth::SsssPermitter<M>,
    store: &S,
) -> Result<(), Error<M>> {
    let start_block = match store.get_chain_state(chain_id).await? {
        Some(ChainState { block }) => block,
        None => permitter.creation_block().await?,
    };

    let processed_block = Arc::new(AtomicU64::new(start_block));
    let state_updater_task = tokio::spawn({
        let store = store.clone();
        let processed_block = processed_block.clone();
        async move {
            loop {
                sleep(Duration::from_secs(5 * 60)).await;
                debug!("updating sync state for chain {chain_id}");
                if let Err(e) = store
                    .update_chain_state(
                        chain_id,
                        ChainStateUpdate {
                            block: Some(processed_block.load(Ordering::Acquire)),
                        },
                    )
                    .await
                {
                    warn!("failed to update sync state for chain {chain_id}: {e}");
                }
            }
        }
    });

    let processed_block = &processed_block;
    permitter
        .events(start_block, None)
        .buffered(100)
        .map(futures::stream::iter)
        .flatten()
        .for_each(|event| async move {
            match event.kind {
                eth::EventKind::Configuration(eth::ConfigurationEvent { identity, config }) => {
                    retry(|| {
                        store.update_verifier(
                            PermitterLocator::new(chain_id, permitter.address),
                            identity,
                            config.clone(),
                            event.index,
                        )
                    })
                    .await;
                }
                eth::EventKind::ProcessedBlock => {
                    processed_block.store(event.index.block, Ordering::Release);
                }
            }
        })
        .await;

    state_updater_task.abort();
    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum Error<M: Middleware> {
    #[error(transparent)]
    Store(#[from] crate::store::Error),
    #[error(transparent)]
    Eth(#[from] eth::Error<M>),
}