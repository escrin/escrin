mod eth;

use ethers::types::Address;
use tracing::{error, trace, warn};
use futures::stream::StreamExt as _;

use crate::store::Store;

#[tracing::instrument(skip_all)]
pub async fn run(
    store: impl Store + 'static,
    gateways: impl Iterator<Item = impl AsRef<str>>,
    permitter_addr: Address,
) -> Result<(), eth::Error> {
    trace!("collating providers");
    let providers = eth::providers(gateways).await?;

    for (chain, provider) in providers.into_iter() {
        let store = store.clone();
        let permitter = eth::SsssPermitter::new(permitter_addr, provider);
        trace!("launching task for chain {chain}");
        tokio::spawn(async move {
            loop {
                match sync_chain(chain, &permitter, &store).await {
                    Ok(_) => warn!("sync task for chain {chain} unexpectedly exited"),
                    Err(e) => error!("sync task for chain {chain} exited with error: {e}"),
                }
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            }
        });
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn sync_chain(
    chain_id: eth::ChainId,
    permitter: &eth::SsssPermitter,
    store: &impl Store,
) -> Result<(), eth::Error> {
    permitter
        .events(0, None)
        .buffer_unordered(100)
        .ready_chunks(1)
        .for_each(|e| async move {
            eprintln!("{:?}", e);
        })
        .await;
    Ok(())
}
