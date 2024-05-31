mod cli;

use std::collections::hash_map::{Entry, HashMap};

use ethers::{
    core::{
        k256::{self, elliptic_curve::group::GroupEncoding as _},
        utils::keccak256,
    },
    middleware::MiddlewareBuilder,
    providers::{Http, Middleware, Provider},
    signers::Signer as _,
    types::{transaction::eip712::Eip712 as _, Address, Signature, H256},
};
use eyre::{ensure, Result, WrapErr as _};
use futures_util::future::{join_all, try_join_all};
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
            let policy_doc_hash = keccak256(serde_json::to_vec(policy_doc)?);
            let permitter = *permitter;

            let (chain, provider) = get_provider(&gateway).await?;
            let provider = provider.with_signer(wallet.private_key.with_chain_id(chain));
            let ssss_permitter = SsssPermitter::new(permitter, provider.into());

            let identity_id = (*il.identity).into();
            let existing_policy_hash = ssss_permitter.policy_hash(identity_id).await?;
            if policy_doc_hash != existing_policy_hash.0 {
                let tx_hash = ssss_permitter
                    .set_policy_hash(identity_id, policy_doc_hash)
                    .await?;
                eprintln!("0x{tx_hash:x}");
            }

            try_join_all(sssss.into_iter().map(|ssss_url| async move {
                SsssClient::new(ssss_url)
                    .set_policy(il.into(), permitter, policy_doc)
                    .await
            }))
            .await?;
        }
        cli::Command::SetApprovers {
            wp:
                cli::WritePermitterArgs {
                    gateway,
                    permitter,
                    wallet,
                },
            identity,
            sssss: cli::Sssss { sssss },
            threshold,
        } => {
            let threshold = s4::calculate_threshold(sssss.len() as u64, *threshold);

            let signers = try_join_all(
                sssss
                    .into_iter()
                    .map(|ssss_url| async move { SsssClient::new(ssss_url).signer().await }),
            )
            .await?;

            let signers_root = s4::generate_signer_proof(&signers, &[])?.0[0];

            let (chain, provider) = get_provider(&gateway).await?;
            let provider = provider.with_signer(wallet.private_key.with_chain_id(chain));
            let ssss_permitter = SsssPermitter::new(*permitter, provider.into());

            ssss_permitter
                .set_approvers_root((*identity).into(), signers_root, threshold)
                .await?;
        }
        cli::Command::AcquireIdentity {
            sssss: cli::Sssss { sssss },
            il,
            wp:
                cli::WritePermitterArgs {
                    gateway,
                    permitter,
                    wallet,
                },
            duration,
            authorization,
            context,
            recipient,
            threshold,
        } => {
            let threshold = s4::calculate_threshold(sssss.len() as u64, *threshold);

            let (chain, provider) = get_provider(&gateway).await?;
            let provider = provider.with_signer(wallet.private_key.clone().with_chain_id(chain));

            let req = &AcqRelIdentityRequest {
                permitter: *permitter,
                recipient,
                base_block: provider.get_block_number().await?.low_u64(),
                duration: Some(duration),
                authorization,
                context,
            };

            let ssss_clients: Vec<_> = sssss.into_iter().map(SsssClient::new).collect();

            let signers = try_join_all(ssss_clients.iter().map(|ssss| ssss.signer())).await?;

            let permit_responses = join_all(ssss_clients.iter().map(|ssss| {
                let wallet = &wallet;
                async move {
                    ssss.request_acquire_identity_permit(il.into(), req, Some(wallet))
                        .await
                }
            }))
            .await
            .into_iter()
            .filter_map(|res| res.ok())
            .collect::<Vec<_>>();

            // SSSSs are not guaranteed to return the same permit, so we take the most common one.
            let (permit, signatures) = {
                let mut signers_by_permit: HashMap<H256, (SsssPermit, Vec<(Address, Signature)>)> =
                    HashMap::new();
                for res in permit_responses.into_iter() {
                    let v = (res.signer, res.signature);
                    match signers_by_permit.entry(res.permit.struct_hash()?.into()) {
                        Entry::Occupied(mut oe) => oe.get_mut().1.push(v),
                        Entry::Vacant(ve) => {
                            ve.insert((res.permit, vec![v]));
                        }
                    }
                }
                let (_, (permit, signatures)) = signers_by_permit
                    .into_iter()
                    .max_by_key(|(_, (_, sigs))| sigs.len())
                    .ok_or_else(|| eyre::eyre!("no permits granted"))?;
                if signatures.len() != signers.len() {
                    warn!("some SSSSs did not return the same permit");
                }
                (permit, signatures)
            };
            ensure!(signatures.len() as u64 >= threshold, "quorum not reached");

            let (proof, proof_flags, leaves) = s4::generate_signer_proof(
                &signers,
                &signatures.iter().map(|(addr, _)| *addr).collect::<Vec<_>>(),
            )?;

            let ssss_permitter = SsssPermitter::new(*permitter, provider.into());

            let mut signatures_by_address = signatures.into_iter().collect::<HashMap<_, _>>();
            let ordered_signatures = leaves
                .into_iter()
                .map(|l| {
                    let sig = signatures_by_address.remove(&l).unwrap();
                    (l, sig)
                })
                .collect::<Vec<_>>();

            ssss_permitter
                .acquire_identity(permit, threshold, (proof, proof_flags), ordered_signatures)
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
            let threshold = s4::calculate_threshold(sssss.len() as u64, *threshold) as usize;
            ensure!(sssss.len() >= threshold, "not enough SSSSs supplied");
            ensure!(threshold >= 2, "threshold must be at least 2");

            let mut rng = rand::thread_rng();

            const SECRET_SIZE: usize = 32;
            let (secret, print_secret) = match secret {
                Some(secret) => {
                    ensure!(
                        secret.len() == SECRET_SIZE,
                        "invalid secret size. expected exactly {SECRET_SIZE} bytes"
                    );
                    (k256::NonZeroScalar::try_from(secret.as_ref())?, false)
                }
                None => (k256::NonZeroScalar::random(&mut rng), true),
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
                                .skip(2) // the library adds the generators as the first two elements
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

            if print_secret {
                debug!("the secret is {:x}", secret);
            }
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

            println!("{:x}", k256::NonZeroScalar::new(secret).unwrap());
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
