mod cli;

use ethers::{
    middleware::MiddlewareBuilder,
    providers::{Http, Middleware, Provider},
    signers::Signer as _,
    types::transaction::eip712::Eip712 as _,
};
use eyre::{Result, WrapErr as _};
use ssss::{eth::SsssPermitter, types::ChainId};

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
            args,
            policy_path,
            private_key,
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

            let (chain, provider) = get_provider(&args.gateway).await?;
            let provider = provider.with_signer(private_key.with_chain_id(chain));
            let ssss = SsssPermitter::new(chain, args.permitter, provider);

            ssss.set_policy(args.identity.into(), cpolicy).await?;
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
