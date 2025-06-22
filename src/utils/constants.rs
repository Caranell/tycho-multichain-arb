use crate::types::Network;

pub const TYCHO_API_KEY: &str = "sampletoken";
pub const TVL_LOWER_BOUND: f64 = 50.0;
pub const TVL_UPPER_BOUND: f64 = 100_000_000.0;

pub fn network(name: String) -> Option<Network> {
    networks().into_iter().find(|n| n.name == name)
}

pub fn networks() -> Vec<Network> {
    vec![
        Network {
            chainid: 1,
            name: "ethereum".to_string(),
            rpc: "https://ethereum-rpc.publicnode.com".to_string(),
            explorer: "https://etherscan.io/".to_string(),
            tycho_url: "tycho-beta.propellerheads.xyz".to_string(),
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3".to_string(),
            router: "0xfD0b31d2E955fA55e3fa641Fe90e08b677188d35".to_string(),
            block_time_ms: 12000,
        },
        Network {
            chainid: 8453,
            name: "base".to_string(),
            rpc: "https://base.llamarpc.com".to_string(),
            explorer: "https://basescan.io/".to_string(),
            tycho_url: "tycho-base-beta.propellerheads.xyz".to_string(),
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3".to_string(),
            router: "0xea3207778e39EB02D72C9D3c4Eac7E224ac5d369".to_string(),
            block_time_ms: 250,
        },
        Network {
            chainid: 130,
            name: "unichain".to_string(),
            rpc: "https://unichain.drpc.org".to_string(),
            explorer: "https://uniscan.xyz/".to_string(),
            tycho_url: "tycho-unichain-beta.propellerheads.xyz".to_string(),
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3".to_string(),
            router: "0xFfA5ec2e444e4285108e4a17b82dA495c178427B".to_string(),
            block_time_ms: 1000,
        },
    ]
}
