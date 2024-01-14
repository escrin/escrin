#![forbid(unsafe_code)]

mod api;
mod conf;

use tracing::info;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true);
    if cfg!(not(debug_assertions)) {
        subscriber.json().with_ansi(false).init();
    } else {
        subscriber.without_time().init();
    }

    let cfg = config::Config::builder().add_source(config::Environment::with_prefix("SSSS"));
    let cfg: conf::Config = match std::env::args().nth(1) {
        Some(conf_file) => cfg.add_source(config::File::with_name(&conf_file)),
        None => cfg,
    }
    .build()
    .unwrap()
    .try_deserialize()
    .unwrap();

    info!(config = ?cfg, "loaded config");
    // let config = aws_config::load_defaults(aws_config::BehaviorVersion::v2023_11_09()).await;
    // let client = aws_sdk_kms::Client::new(&config);
    // println!("Hello, world!");

    let api_task = api::serve(cfg.api_port);

    tokio::join!(api_task);
}
