mod cli;

use aes_gcm_siv::AeadInPlace as _;
use ethers::{
    middleware::MiddlewareBuilder,
    providers::{Http, Middleware, Provider},
    signers::Signer as _,
    types::{transaction::eip712::Eip712 as _, Bytes},
};
use eyre::{Result, WrapErr as _};
use rand::RngCore as _;
use ssss::{eth::SsssHub, identity, types::ChainId};
use tracing::{debug, warn};

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
            args:
                cli::WritePermitterArgs {
                    private_key,
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
            let mut cpolicy = Vec::with_capacity(policy_bytes.len());
            brotli::BrotliCompress(
                &mut policy_bytes.as_slice(),
                &mut cpolicy,
                &brotli::enc::backward_references::BrotliEncoderParams {
                    quality: 11,
                    size_hint: policy_bytes.len(),
                    magic_number: true,
                    ..Default::default()
                },
            )?;

            let (chain, provider) = get_provider(&gateway).await?;
            let provider = provider.with_signer(private_key.with_chain_id(chain));
            let ssss = SsssHub::new(chain, permitter, provider);

            ssss.set_policy(identity.into(), cpolicy).await?;
        }
        cli::Command::SignOmniKeyRequest {
            ssss,
            chain,
            registry,
            identity,
            share_version,
            private_key,
        } => {
            let req = ssss::types::SsssRequest {
                method: "GET".into(),
                uri: format!(
                    "{ssss}/v1/shares/omni/{chain}/{registry}/{identity}?version={share_version}"
                ),
                body: Default::default(),
            };
            let req_hash = req.encode_eip712()?;
            let sig = private_key.sign_hash(req_hash.into())?;
            eprintln!("{:x}", ethers::types::Bytes::from(&req_hash));
            println!("0x{sig}");
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
                    private_key,
                },
        } => {
            let ssss_identities =
                futures::future::try_join_all(sssss.iter().map(|maybe_ssss_url| async {
                    let url: url::Url = maybe_ssss_url.parse()?;
                    #[derive(serde::Deserialize)]
                    struct SsssIdentity {
                        persistent: elliptic_curve::JwkEcKey,
                    }
                    let ssss_identity: SsssIdentity =
                        reqwest::get(url.join("/v1/identity").unwrap())
                            .await?
                            .error_for_status()?
                            .json()
                            .await?;
                    Ok::<_, eyre::Error>(ssss_identity.persistent.to_public_key()?)
                }))
                .await?;

            let (chain, provider) = get_provider(&gateway).await?;
            let provider = provider.with_signer(private_key.with_chain_id(chain));
            let ssss = SsssHub::new(chain, permitter, provider);

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
                let threshold = if threshold > 1.0 {
                    threshold as usize
                } else {
                    (threshold * (limit as f64)).ceil() as usize
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
                let cipher =
                    my_identity.derive_shared_cipher(ssss_identity, identity::SHARES_DOMAIN_SEP);
                cipher
                    .encrypt_in_place(&shares_nonce, &[], &mut shares[i])
                    .unwrap();
            }

            ssss.deal_shares_sss(
                identity.into(),
                version,
                my_identity.public_key().to_sec1_bytes().into_vec(),
                nonce,
                shares.into_iter().map(Bytes::from).collect(),
            )
            .await?;
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

// async fn get_ssss_caller(gateway: &str, addr: Address, signer: LocalWallet) -> Result<SsssPermitter<Provider<Http>>> {
//     let provider = Provider::<Http>::try_from(gateway).wrap_err("failed to connect to gateway")?;
//     let chain = provider
//         .get_chainid()
//         .await
//         .wrap_err("failed to get chainid from gateway")?
//         .as_u64();
//     Ok(SsssPermitter::new(chain, addr, provider))
// }
