mod arbitrage_graph;
mod configuration;
mod stream_builder;
mod tycho_api;
mod types;
mod utils;

use configuration::load_config;
use futures::StreamExt;
use futures::future::select_all;
use std::collections::HashMap;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use stream_builder::create_protocol_stream_builder;
use tokio::sync::Mutex;
use tycho_api::get_tokens;
use tycho_common::models::Chain;
use tycho_simulation::tycho_client::feed::component_tracker::ComponentFilter;
use types::ArbitrageGraph;
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
    let mut chain_tokens = HashMap::new();

    for chain_config in config.chains {
        tracing::info!("Processing chain: {}", chain_config.name);

        let chain = Chain::from_str(&chain_config.name).unwrap();
        let network = network(chain_config.name.clone()).unwrap().clone();
        let tokens = get_tokens(&network, TYCHO_API_KEY.to_string(), chain_config.tokens)
            .await
            .unwrap();

        chain_tokens.insert(chain, tokens.clone());

        let stream_builder = create_protocol_stream_builder(
            network,
            config.rpc_url.clone(),
            tvl_filter.clone(),
            TYCHO_API_KEY.to_string(),
            tokens,
        )
        .await;

        stream_builders.push((stream_builder, chain));
    }

    let arbitrage_graph = Arc::new(Mutex::new(ArbitrageGraph::new()));
    {
        let mut graph = arbitrage_graph.lock().await;
        graph.initialize(chain_tokens);
    }

    let mut tasks = vec![];

    for (stream_builder, chain) in stream_builders {
        let arbitrage_graph = Arc::clone(&arbitrage_graph);
        let task = tokio::spawn(async move {
            let mut stream = stream_builder
                .build()
                .await
                .expect("Failed building protocol stream");

            while let Some(message_result) = stream.next().await {
                match message_result {
                    Ok(msg) => {
                        let mut graph = arbitrage_graph.lock().await;
                        graph.handle_block_update(msg, chain);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Error receiving message: {e:?}. Continuing to next message..."
                        );
                        continue;
                    }
                };
            }
        });

        tasks.push(task);
    }

    let _ = select_all(tasks).await;
}

pub fn setup_tracing() {
    let filter =
        tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse().unwrap()); // Default to info level if RUST_LOG is not set
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
