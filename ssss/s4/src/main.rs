mod cli;

use ethers::{
    core::k256::{self, elliptic_curve::group::GroupEncoding as _},
    middleware::MiddlewareBuilder,
    providers::{Http, Middleware, Provider},
    signers::Signer as _,
    types::Bytes,
};
use eyre::{Result, WrapErr as _};
use futures_util::future::try_join_all;
use s4::SsssClient;
use ssss::{
    eth::SsssPermitter,
    types::{api::*, *},
};
use tracing::{debug, warn};
use vsss_rs::PedersenResult as _;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_env_filter(match args.verbosity {
            0 => "s4=warn",
            1 => "s4=info",
            2 => "s4=debug",
            _ => "s4=trace",
        })
        .with_target(true)
        .without_time()
        .init();

    match args.command {
        cli::Command::GetSsssSigner {
            ssss: cli::Ssss { ssss },
        } => {
            let signer = SsssClient::new(ssss).signer().await?;
            println!("0x{:x}", signer);
        }
        cli::Command::SetPolicy {
            wp:
                cli::WritePermitterArgs {
                    gateway,
                    permitter,
                    identity,
                    wallet,
                },
            il,
            verifier,
            policy_path,
            sssss: cli::Sssss { sssss },
        } => {
            let input: Box<dyn std::io::Read> = match policy_path {
                Some(p) => Box::new(std::fs::File::open(p)?),
                None => Box::new(std::io::stdin()),
            };
            let policy: serde_json::Value = serde_json::from_reader(input)?;

            let policy_doc = &PolicyDocument {
                verifier: verifier.to_string(),
                policy,
            };
            let policy_doc_hash = ethers::utils::keccak256(serde_json::to_vec(policy_doc)?);

            let (chain, provider) = get_provider(&gateway).await?;
            let provider = provider.with_signer(wallet.private_key.with_chain_id(chain));
            let ssss = SsssPermitter::new(*permitter, provider.into());

            let identity_id = (*identity).into();
            let existing_policy_hash = ssss.policy_hash(identity_id).await?;
            if policy_doc_hash != existing_policy_hash.0 {
                let tx_hash = ssss.set_policy_hash(identity_id, policy_doc_hash).await?;
                eprintln!("{tx_hash}");
            }

            try_join_all(sssss.into_iter().map(|ssss_url| async move {
                SsssClient::new(ssss_url)
                    .set_policy(il.into(), policy_doc)
                    .await
            }))
            .await?;
        }
        cli::Command::Deal {
            sssss: cli::Sssss { sssss },
            il,
            name,
            version,
            threshold,
            secret,
            wallet,
        } => {
            let threshold = if *threshold > 1.0 {
                *threshold as usize
            } else {
                (*threshold * (sssss.len() as f64)).ceil() as usize
            };
            eyre::ensure!(sssss.len() >= threshold, "not enough SSSSs supplied");
            eyre::ensure!(threshold >= 2, "threshold must be at least 2");

            let mut rng = rand::thread_rng();

            const SECRET_SIZE: usize = 32;
            let secret = match secret {
                Some(secret) => {
                    eyre::ensure!(
                        secret.len() == SECRET_SIZE,
                        "invalid secret size. expected exactly {SECRET_SIZE} bytes"
                    );
                    k256::NonZeroScalar::try_from(secret.as_ref())?
                }
                None => {
                    let s = k256::NonZeroScalar::random(&mut rng);
                    debug!("the secret is {:x}", s);
                    s
                }
            };

            let shares: Vec<api::SecretShare> = {
                let res =
                    vsss_rs::pedersen::split_secret::<k256::ProjectivePoint, u64, (u64, Vec<u8>)>(
                        threshold,
                        sssss.len(),
                        *secret,
                        None,
                        Some(k256::ProjectivePoint::GENERATOR),
                        Some(*PEDERSEN_VSS_BLINDER_GENERATOR),
                        &mut rand::thread_rng(),
                    )
                    .map_err(|e| eyre::eyre!("vss error: {e}"))?;
                res.secret_shares()
                    .iter()
                    .cloned()
                    .zip(res.blinder_shares().iter().cloned())
                    .map(|((index, share), (_, blinder))| api::SecretShare {
                        meta: SecretShareMeta {
                            index,
                            commitments: res
                                .pedersen_verifier_set()
                                .iter()
                                .map(|p| p.to_bytes().to_vec())
                                .collect(),
                        },
                        share: share.into(),
                        blinder: blinder.into(),
                    })
                    .collect()
            };

            let share_id = ShareId {
                identity: il.into(),
                secret_name: name.clone(),
                version: *version,
            };

            let ssss_clients = sssss.into_iter().map(SsssClient::new).collect::<Vec<_>>();
            try_join_all(
                ssss_clients
                    .iter()
                    .zip(shares.into_iter())
                    .map(|(ssss, share)| ssss.deal_share(&share_id, share, &wallet)),
            )
            .await?;

            try_join_all(
                ssss_clients
                    .iter()
                    .map(|ssss| ssss.commit_share(&share_id, &wallet)),
            )
            .await?;
        }
        cli::Command::Reconstruct {
            il,
            name,
            version,
            sssss: cli::Sssss { sssss },
            wallet,
        } => {
            let wallet = &*wallet;
            let share_id = &ShareId {
                identity: il.into(),
                secret_name: name,
                version: *version,
            };
            let shares = try_join_all(sssss.into_iter().map(|ssss_url| async move {
                SsssClient::new(ssss_url).get_share(share_id, wallet).await
            }))
            .await?;
            // TODO: verify shares
            let secret = vsss_rs::combine_shares::<k256::Scalar, u64, (u64, Vec<u8>)>(
                &shares
                    .into_iter()
                    .map(|s| (s.meta.index, s.share.0.into()))
                    .collect::<Vec<_>>(),
            )
            .map_err(|_| eyre::eyre!("failed to reconstruct shares"))?;

            println!("{:x}", Bytes::from(secret.to_bytes().to_vec()))
        }
        cli::Command::AcquireIdentity {
            sssss: cli::Sssss { sssss },
            il,
            wp:
                cli::WritePermitterArgs {
                    gateway: _,
                    permitter,
                    wallet,
                    ..
                },
            duration,
            authorization,
            context,
            recipient,
        } => {
            let req = &AcqRelIdentityRequest {
                permitter: *permitter,
                recipient,
                base_block: Default::default(),
                duration: Some(duration),
                authorization,
                context,
            };
            let signatures = try_join_all(sssss.into_iter().map(|ssss_url| {
                let wallet = &wallet;
                async move {
                    SsssClient::new(ssss_url)
                        .request_acquire_identity_permit(il.into(), req, Some(wallet))
                        .await
                }
            }))
            .await?;

            warn!(
                "s4 does not yet have support for submitting identity permits. Please submit \
                 these manually."
            );

            for (address, sig) in signatures {
                println!("0x{address:x} 0x{sig}");
            }
        }
    }

    Ok(())
}

async fn get_provider(gateway: &str) -> Result<(ChainId, Provider<Http>)> {
    let provider = Provider::<Http>::try_from(gateway).wrap_err("failed to connect to gateway")?;
    let chain = provider
        .get_chainid()
        .await
        .wrap_err("failed to get chainid from gateway")?
        .as_u64();
    Ok((chain, provider))
}
