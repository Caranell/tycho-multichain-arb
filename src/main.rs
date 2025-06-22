mod configuration;
mod stream_builder;
mod tycho_api;
mod types;
mod utils;

use configuration::load_config;
use std::process;
use std::str::FromStr;
use stream_builder::create_protocol_stream_builder;
use tycho_api::get_tokens;
use tycho_common::models::Chain;
use tycho_simulation::tycho_client::feed::component_tracker::ComponentFilter;

use utils::constants::{TVL_LOWER_BOUND, TVL_UPPER_BOUND, TYCHO_API_KEY, network};

#[tokio::main]
async fn main() {
    setup_tracing();

    let config = match load_config() {
        Ok(config) => {
            tracing::info!("Config loaded successfully: {:?}", config);
            config
        }
        Err(e) => {
            tracing::error!("Error loading config: {}", e);
            process::exit(1);
        }
    };

    let mut stream_builders = vec![];
    let tvl_filter = ComponentFilter::with_tvl_range(TVL_LOWER_BOUND, TVL_UPPER_BOUND);

    for chain_config in config.chains {
        println!("Processing chain: {}", chain_config.name);

        let chain = Chain::from_str(&chain_config.name).unwrap();
        let network = network(chain_config.name.clone()).unwrap().clone();
        let tokens = get_tokens(&network, TYCHO_API_KEY.to_string(), chain_config.tokens).await;

        let stream_builder = create_protocol_stream_builder(
            network,
            chain,
            tvl_filter.clone(),
            TYCHO_API_KEY.to_string(),
            tokens.unwrap(),
        )
        .await;

        stream_builders.push(stream_builder);
    }
}

// pub async fn handle_stream(stream_builder: ProtocolStreamBuilder) {
//     let stream = stream_builder.build().await.unwrap();
// }

pub fn setup_tracing() {
    let filter =
        tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse().unwrap()); // Default to info level if RUST_LOG is not set
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
