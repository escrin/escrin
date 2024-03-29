mod cli;

use aes_gcm_siv::AeadInPlace as _;
use ethers::{
    middleware::MiddlewareBuilder,
    providers::{Http, Middleware, Provider},
    signers::Signer as _,
    types::Bytes,
};
use eyre::{Result, WrapErr as _};
use rand::RngCore as _;
use s4::SsssClient;
use ssss::{
    eth::SsssHub,
    identity,
    types::{api::*, *},
};
use tracing::{debug, info, warn};

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
        cli::Command::SetPolicy {
            policy_path,
            verifier,
            args:
                cli::WritePermitterArgs {
                    wallet,
                    gateway,
                    permitter,
                    identity,
                    ..
                },
        } => {
            let input: Box<dyn std::io::Read> = match policy_path {
                Some(p) => Box::new(std::fs::File::open(p)?),
                None => Box::new(std::io::stdin()),
            };
            let policy: serde_json::Value = serde_json::from_reader(input)?;
            let mut policy_bytes = Vec::new();
            ciborium::into_writer(&policy, &mut policy_bytes)?;

            let mut preamble_bytes = Vec::with_capacity(policy_bytes.len() + 100);
            ciborium::into_writer(
                &PolicyPreamble {
                    verifier: verifier.to_string(),
                    policy: policy_bytes,
                },
                &mut preamble_bytes,
            )?;

            let mut cpolicy = Vec::with_capacity(preamble_bytes.len());
            brotli::BrotliCompress(
                &mut preamble_bytes.as_slice(),
                &mut cpolicy,
                &brotli::enc::backward_references::BrotliEncoderParams {
                    quality: 11,
                    size_hint: preamble_bytes.len(),
                    magic_number: true,
                    ..Default::default()
                },
            )?;

            let (chain, provider) = get_provider(&gateway).await?;
            let provider = provider.with_signer(wallet.private_key.with_chain_id(chain));
            let ssss = SsssHub::new(chain, *permitter, provider);

            ssss.set_policy((*identity).into(), cpolicy).await?;
        }
        cli::Command::Deal {
            secret,
            version,
            sssss,
            threshold,
            args:
                cli::WritePermitterArgs {
                    gateway,
                    permitter,
                    identity,
                    wallet,
                },
        } => {
            let ssss_identities =
                futures_util::future::try_join_all(sssss.iter().map(|maybe_ssss_url| async {
                    Ok::<_, eyre::Error>(
                        SsssClient::new(maybe_ssss_url.parse()?)
                            .get_ssss_identity()
                            .await?
                            .persistent
                            .to_public_key()?,
                    )
                }))
                .await?;

            let (chain, provider) = get_provider(&gateway).await?;
            let provider = provider.with_signer(wallet.private_key.with_chain_id(chain));
            let ssss = SsssHub::new(chain, *permitter, provider);

            let mut rng = rand::thread_rng();

            let mut nonce = [0u8; 32];
            rng.fill_bytes(&mut nonce);
            let shares_nonce = {
                let mut n = [0u8; 12];
                n.copy_from_slice(&nonce[0..12]);
                n.into()
            };

            let limit = sssss.len();

            let mut shares = if limit == 1 {
                warn!("with only one SSSS shareholder, ensure that you trust it completely!");
                let secret = match secret {
                    Some(s) => s.to_vec(),
                    None => {
                        let mut s = vec![0u8; 32];
                        rng.fill_bytes(&mut s);
                        debug!("the secret is {:x}", Bytes::from(s.to_vec()));
                        s
                    }
                };
                vec![secret]
            } else {
                let threshold = if *threshold > 1.0 {
                    *threshold as usize
                } else {
                    (*threshold * (limit as f64)).ceil() as usize
                };
                let secret = match secret {
                    Some(s) => p384::Scalar::from_slice(&s)?,
                    None => {
                        let mut scalar_bytes = [0u8; 48];
                        rng.fill_bytes(&mut scalar_bytes);
                        debug!("the secret is {:x}", Bytes::from(scalar_bytes));
                        p384::Scalar::from_bytes(&scalar_bytes.into()).unwrap()
                    }
                };
                vsss_rs::shamir::split_secret::<p384::Scalar, u8, Vec<u8>>(
                    threshold,
                    limit,
                    secret,
                    &mut rand::thread_rng(),
                )
                .map_err(|e| eyre::eyre!("vss error: {e}"))?
            };

            let my_identity = ssss::identity::Identity::ephemeral();
            my_identity.public_key();

            for (i, ssss_identity) in ssss_identities.into_iter().enumerate() {
                let cipher = my_identity
                    .derive_shared_cipher(ssss_identity, identity::DEAL_SHARES_DOMAIN_SEP);
                cipher
                    .encrypt_in_place(&shares_nonce, &[], &mut shares[i])
                    .unwrap();
            }

            ssss.deal_shares_sss(
                (*identity).into(),
                *version,
                my_identity.public_key().to_sec1_bytes().into_vec(),
                nonce,
                shares.into_iter().map(Bytes::from).collect(),
            )
            .await?;
        }
        cli::Command::Reconstruct {
            il,
            version,
            sssss,
            wallet,
        } => {
            let wallet = &*wallet;
            let shares =
                futures_util::future::try_join_all(sssss.iter().map(|url_str| async move {
                    let url: url::Url = url_str.parse()?;
                    SsssClient::new(url)
                        .get_share("omni", il.into(), *version, wallet, None)
                        .await
                }))
                .await?;

            let secret = vsss_rs::combine_shares::<p384::Scalar, u8, Vec<u8>>(
                &shares.into_iter().map(|s| s.1).collect::<Vec<_>>(),
            )
            .map_err(|_| eyre::eyre!("failed to reconstruct shares"))?;

            println!("{:x}", Bytes::from(secret.to_bytes().to_vec()))
        }
        cli::Command::AcquireIdentity {
            ssss,
            il,
            wallet,
            duration,
            authorization,
            context,
            permitter,
            recipient,
        } => {
            let permit_created = SsssClient::new(ssss.parse()?)
                .acquire_identity(
                    il.into(),
                    &AcqRelIdentityRequest {
                        duration,
                        authorization,
                        context,
                        permitter: (*permitter),
                        recipient,
                    },
                    wallet.as_deref(),
                )
                .await?;
            if permit_created {
                info!("SSSS optimistically created permit");
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
