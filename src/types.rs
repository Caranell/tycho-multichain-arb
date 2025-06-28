use serde::{Deserialize, Serialize};
use tycho_common::models::Chain;
use tycho_simulation::{models::Token, protocol::state::ProtocolSim};
use petgraph::graph::DiGraph;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenConfig {
    pub symbol: String,
    pub address: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ChainConfig {
    pub name: String,
    pub tokens: Vec<TokenConfig>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub private_key: String,
    pub risk_param: u64,
    pub rpc_url: String,
    pub chains: Vec<ChainConfig>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub name: String,
    pub chain: Chain,
    pub chainid: u64,
    pub rpc: String,
    pub explorer: String,
    pub tycho_url: String,
    pub router: String,
    pub permit2: String,
    pub block_time_ms: u64,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Protocol {
    UniswapV2,
    UniswapV3,
    UniswapV4,
    VmBalancerV2,
    VmCurve,
    SushiswapV2,
    PancakeswapV2,
    PancakeswapV3,
    EkuboV2,
}

impl Protocol {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "uniswap_v2" => Some(Protocol::UniswapV2),
            "uniswap_v3" => Some(Protocol::UniswapV3),
            "uniswap_v4" => Some(Protocol::UniswapV4),
            "vm:balancer_v2" => Some(Protocol::VmBalancerV2),
            "vm:curve" => Some(Protocol::VmCurve),
            "sushiswap_v2" => Some(Protocol::SushiswapV2),
            "pancakeswap_v2" => Some(Protocol::PancakeswapV2),
            "pancakeswap_v3" => Some(Protocol::PancakeswapV3),
            "ekubo_v2" => Some(Protocol::EkuboV2),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Protocol::UniswapV2 => "uniswap_v2",
            Protocol::UniswapV3 => "uniswap_v3",
            Protocol::UniswapV4 => "uniswap_v4",
            Protocol::VmBalancerV2 => "vm:balancer_v2",
            Protocol::VmCurve => "vm:curve",
            Protocol::SushiswapV2 => "sushiswap_v2",
            Protocol::PancakeswapV2 => "pancakeswap_v2",
            Protocol::PancakeswapV3 => "pancakeswap_v3",
            Protocol::EkuboV2 => "ekubo_v2",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TokenNode {
  pub symbol: String,
  pub tokens: HashMap<Chain, Token>,
}

#[derive(Debug, Clone)]
pub struct PriceEdge {
    pub chain: Chain,
    pub protocol: Protocol,
    pub pool_address: String,
    pub state: Box<dyn ProtocolSim>,
    pub to_token: Token,
    pub from_token: Token,
    pub price: f64,
}
  

#[derive(Debug, Clone)]
pub struct ArbitrageGraph {
    pub graph: DiGraph<TokenNode, PriceEdge>,
}
