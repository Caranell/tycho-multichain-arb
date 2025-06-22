mod configuration;
mod tycho_api;
mod types;
mod utils;
mod stream_builder;

use configuration::load_config;
use std::process;

fn main() {
    setup_tracing();

    let config = match load_config() {
        Ok(config) => {
            println!("Config loaded successfully: {:?}", config);
            config
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            process::exit(1);
        }
    };
}

pub fn setup_tracing() {
    let filter = tracing_subscriber::EnvFilter::from_default_env(); 
    tracing_subscriber::fmt().with_env_filter(filter).init(); 
}
