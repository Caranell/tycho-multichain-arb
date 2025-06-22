use crate::types::Config;
use serde_yaml;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use tycho_common::models::Chain;

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_file = std::fs::File::open("config.yml")?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    validate_config(&config)?;

    return Ok(config);
}

fn validate_config(config: &Config) -> Result<(), String> {
    if config.private_key.is_empty() {
        return Err("Private key is required".into());
    }

    // TODO: add risk param validation

    if config.chains.is_empty() {
        return Err("Chain configuration is required".into());
    }

    if config.chains.len() < 2 {
        return Err("At least two chains are required".into());
    }

    for chain_config in &config.chains {
        if Chain::from_str(&chain_config.name).is_err() {
            return Err(format!("Unknown chain name: {}", chain_config.name).into());
        }
    }

    let mut token_locations: HashMap<String, HashSet<String>> = HashMap::new();

    // map through tokens and validate all fields are present
    for chain in &config.chains {
        if chain.tokens.is_empty() {
            return Err(format!("At least one token is required for chain: {}", chain.name).into());
        }

        for token in &chain.tokens {
            if token.symbol.is_empty() {
                return Err(format!("Symbol is required for token: {}", token.address).into());
            }

            if token.address.is_empty() {
                return Err(format!("Address is required for token: {}", token.symbol).into());
            }

            if token.decimals == 0 {
                return Err(format!("Decimals is required for token: {}", token.symbol).into());
            }

            token_locations
                .entry(token.symbol.clone())
                .or_insert(HashSet::new())
                .insert(chain.name.clone());
        }
    }

    // validate that each token is present on multiple chains
    for (symbol, chains) in &token_locations {
        if chains.len() < 2 {
            return Err(format!("Token {} is not present on multiple chains", symbol).into());
        }
    }

    return Ok(());
}
