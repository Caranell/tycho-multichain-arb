use crate::types::Network;
use num_bigint::BigUint;
use std::collections::HashMap;
use std::str::FromStr;
use tycho_client::rpc::{HttpRPCClient, RPCClient};
use tycho_common::dto::{Chain, PaginationParams, TokensRequestBody};
use tycho_simulation::models::Token;
use tycho_simulation::tycho_core::Bytes;

use crate::types::TokenConfig;

pub fn create_tycho_client(
    network: &Network,
    api_key: String,
) -> Result<HttpRPCClient, anyhow::Error> {
    match HttpRPCClient::new(
        format!("https://{}", &network.tycho_url).as_str(),
        Some(api_key.as_str()),
    ) {
        Ok(client) => Ok(client),
        Err(e) => {
            tracing::error!("Failed to create client: {:?}", e.to_string());
            Err(anyhow::anyhow!(
                "Failed to create client: {:?}",
                e.to_string()
            ))
        }
    }
}

pub async fn get_tokens(
    network: &Network,
    api_key: String,
    config_tokens: Vec<TokenConfig>,
) -> Result<HashMap<Bytes, Token>, anyhow::Error> {
    let client = create_tycho_client(network, api_key).unwrap();

    tracing::info!("Getting tokens for network {}", network.name);

    let chain = Chain::from_str(&network.name).unwrap();

    let token_addresses: Vec<Bytes> = config_tokens
        .iter()
        .map(|s| Bytes::from_str(s.address.as_str()).unwrap())
        .collect();

    let request = TokensRequestBody {
        token_addresses: Some(token_addresses),
        chain: chain,
        min_quality: None,
        traded_n_days_ago: None,
        pagination: PaginationParams {
            page: 0,
            page_size: 100,
        },
    };

    let tokens_response = client.get_tokens(&request).await;

    let mut tokens: HashMap<Bytes, Token> = HashMap::new();

    for token in tokens_response.unwrap().tokens {
        tokens.insert(
            token.address.clone(),
            Token {
                address: token.address.clone(),
                decimals: token.decimals as usize,
                symbol: token.symbol.clone(),
                gas: BigUint::from(token.gas.first().unwrap_or(&Some(0u64)).unwrap_or_default()),
            },
        );
    }

    tracing::info!("Got {} tokens", tokens.len());

    Ok(tokens)
}
