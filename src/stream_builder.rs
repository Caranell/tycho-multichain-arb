use crate::types::{Network, Protocol};
use std::collections::HashMap;
use tycho_client::feed::component_tracker::ComponentFilter;
use tycho_simulation::evm::{
    engine_db::tycho_db::PreCachedDB,
    protocol::{
        ekubo::state::EkuboState,
        filters::{balancer_v2_pool_filter, curve_pool_filter, uniswap_v4_pool_with_hook_filter},
        uniswap_v2::state::UniswapV2State,
        uniswap_v3::state::UniswapV3State,
        uniswap_v4::state::UniswapV4State,
        vm::state::EVMPoolState,
    },
    stream::ProtocolStreamBuilder,
    tycho_models::Chain,
};
use tycho_simulation::models::Token;
use tycho_simulation::tycho_core::Bytes;

pub async fn create_protocol_stream_builder(
    network: Network,
    rpc_url: String,
    tvl_filter: ComponentFilter,
    api_key: String,
    tokens: HashMap<Bytes, Token>,
) -> ProtocolStreamBuilder {
    tracing::info!(
        "Creating protocol stream builder for chain {}",
        network.name
    );

    let mut builder = ProtocolStreamBuilder::new(network.tycho_url.as_str(), network.chain);
    builder = add_exchanges(builder, &network.chain, tvl_filter, rpc_url);
    builder = setup_stream_builder(builder, api_key, tokens).await;

    builder
}

pub fn add_exchanges(
    mut builder: ProtocolStreamBuilder,
    chain: &Chain,
    tvl_filter: ComponentFilter,
    rpc_url: String,
) -> ProtocolStreamBuilder {
    match chain {
        Chain::Ethereum => {
            builder = builder
                .exchange::<UniswapV2State>(Protocol::UniswapV2.to_str(), tvl_filter.clone(), None)
                .exchange::<UniswapV3State>(Protocol::UniswapV3.to_str(), tvl_filter.clone(), None)
                .exchange::<UniswapV4State>(
                    Protocol::UniswapV4.to_str(),
                    tvl_filter.clone(),
                    Some(uniswap_v4_pool_with_hook_filter),
                )
                .exchange::<EkuboState>(Protocol::EkuboV2.to_str(), tvl_filter.clone(), None)
                .exchange::<UniswapV2State>(
                    Protocol::SushiswapV2.to_str(),
                    tvl_filter.clone(),
                    None,
                )
                .exchange::<UniswapV2State>(
                    Protocol::PancakeswapV2.to_str(),
                    tvl_filter.clone(),
                    None,
                )
                .exchange::<UniswapV3State>(
                    Protocol::PancakeswapV3.to_str(),
                    tvl_filter.clone(),
                    None,
                );

            if !rpc_url.is_empty() {
                builder = builder
                    .exchange::<EVMPoolState<PreCachedDB>>(
                        Protocol::VmBalancerV2.to_str(),
                        tvl_filter.clone(),
                        Some(balancer_v2_pool_filter),
                    )
                    .exchange::<EVMPoolState<PreCachedDB>>(
                        Protocol::VmCurve.to_str(),
                        tvl_filter.clone(),
                        Some(curve_pool_filter),
                    )
            }
        }
        Chain::Base => {
            builder = builder
                .exchange::<UniswapV2State>(Protocol::UniswapV2.to_str(), tvl_filter.clone(), None)
                .exchange::<UniswapV3State>(Protocol::UniswapV3.to_str(), tvl_filter.clone(), None)
                .exchange::<UniswapV4State>(
                    Protocol::UniswapV4.to_str(),
                    tvl_filter.clone(),
                    Some(uniswap_v4_pool_with_hook_filter),
                );
        }
        Chain::Unichain => {
            builder = builder
                .exchange::<UniswapV2State>(Protocol::UniswapV2.to_str(), tvl_filter.clone(), None)
                .exchange::<UniswapV3State>(Protocol::UniswapV3.to_str(), tvl_filter.clone(), None)
                .exchange::<UniswapV4State>(
                    Protocol::UniswapV4.to_str(),
                    tvl_filter.clone(),
                    Some(uniswap_v4_pool_with_hook_filter),
                );
        }
        _ => {}
    };

    builder
}

pub async fn setup_stream_builder(
    mut builder: ProtocolStreamBuilder,
    api_key: String,
    tokens: HashMap<Bytes, Token>,
) -> ProtocolStreamBuilder {
    builder = builder
        .auth_key(Some(api_key.clone()))
        .skip_state_decode_failures(true)
        .set_tokens(tokens.clone())
        .await;

    builder
}
