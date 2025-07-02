use crate::types::{ArbitrageGraph, PriceEdge, Protocol, TokenNode};
use crate::utils::graph::GraphIndexUpdateTrait;
use petgraph::graph::DiGraph;
use petgraph::Direction::{Incoming, Outgoing};
use petgraph::prelude::{EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use tycho_common::{models::Chain, Bytes};
use tycho_simulation::protocol::models::{BlockUpdate, ProtocolComponent};
use tycho_simulation::{models::Token, protocol::state::ProtocolSim};

impl ArbitrageGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            edges_map: HashMap::new(),
            nodes_map: HashMap::new(),
        }
    }

    // TODO: decide if we need to call component (edge) synchronization
    pub fn initialize(&mut self, chain_tokens: HashMap<Chain, HashMap<Bytes, Token>>) {
        let mut symbol_to_tokens: HashMap<String, HashMap<Chain, Token>> = HashMap::new();

        for (chain, tokens) in chain_tokens {
            for (_, token) in tokens {
                symbol_to_tokens
                    .entry(token.symbol.clone())
                    .or_insert_with(HashMap::new)
                    .insert(chain, token);
            }
        }

        for (symbol, tokens_map) in symbol_to_tokens {
            let node_index = self.graph.add_node(TokenNode {
                symbol: symbol.clone(),
                tokens: tokens_map,
            });
            self.nodes_map.insert(symbol, node_index);
        }
    }

    pub fn add_edge(&mut self, edge: PriceEdge, from: NodeIndex, to: NodeIndex) -> EdgeIndex {
        let edge_index = self.graph.add_edge(from, to, edge);
        edge_index
    }

    pub fn update_edge(&mut self, index: EdgeIndex, edge: PriceEdge) {
        self.graph.update_edge_by_index(index, edge);
    }

    pub fn handle_block_update(&mut self, msg: BlockUpdate, _chain: Chain) {
        for (_, pair) in msg.new_pairs {
            let state = msg.states.get(&pair.id.to_string()).unwrap().clone();
            self.handle_new_pair(pair, state);
        }

        for (address, state) in msg.states {
            self.handle_state_update(state, address);
        }

        self.format_pairs();
    }

    pub fn format_pairs(&self) {
        let node_indices = self.graph.node_indices().collect::<Vec<_>>();

        for node_index in node_indices {
            let node_symbol = self.graph.node_weight(node_index).unwrap().symbol.clone();
            let node_edges = self.graph.edges_directed(node_index, Incoming).collect::<Vec<_>>();

            println!("{}", node_symbol);

            for edge in node_edges {
                let edge_weight = edge.weight().clone();

                println!(
                    "{} -> {} | Chain: {:?} | Protocol: {} | Pool: {} | Price: {:.6}",
                    edge_weight.from_token.symbol,
                    edge_weight.to_token.symbol,
                    edge_weight.chain,
                    edge_weight.protocol.to_str(),
                    edge_weight.pool_address,
                    edge_weight.price
                );
            }
        }
    }

    pub fn handle_new_pair(&mut self, pair: ProtocolComponent, state: Box<dyn ProtocolSim>) {
        let from_node = self.nodes_map.get(&pair.tokens[0].symbol.clone()).unwrap().clone();
        let to_node = self.nodes_map.get(&pair.tokens[1].symbol.clone()).unwrap().clone();

        let edge_index_first = self.add_edge(
            PriceEdge {
                chain: pair.chain,
                protocol: Protocol::from_str(&pair.protocol_system).unwrap(),
                price: state.spot_price(&pair.tokens[0], &pair.tokens[1]).unwrap(),
                state: state.clone(),
                pool_address: pair.id.to_string(),
                to_token: pair.tokens[0].clone(),
                from_token: pair.tokens[1].clone(),
            },
            from_node,
            to_node,
        );

        let edge_index_second = self.add_edge(
            PriceEdge {
                chain: pair.chain,
                protocol: Protocol::from_str(&pair.protocol_system).unwrap(),
                price: state.spot_price(&pair.tokens[1], &pair.tokens[0]).unwrap(),
                state: state.clone(),
                pool_address: pair.id.to_string(),
                to_token: pair.tokens[1].clone(),
                from_token: pair.tokens[0].clone(),
            },
            to_node,
            from_node,
        );

        self.edges_map.insert(pair.id.to_string(), vec![edge_index_first, edge_index_second]);
    }

    pub fn handle_state_update(&mut self, state: Box<dyn ProtocolSim>, address: String) {
        if let Some(edge_indices) = self.edges_map.get(&address) {
            let indices = edge_indices.clone();
            for idx in indices {
                self.update_edge_weight(idx, state.clone());
            }
        }
    }

    pub fn update_edge_weight(&mut self, idx: EdgeIndex, state: Box<dyn ProtocolSim>) {
        let mut edge_weight = self.graph.edge_weight(idx).unwrap().clone();

        edge_weight.price = state
            .spot_price(&edge_weight.from_token, &edge_weight.to_token)
            .unwrap();
        edge_weight.state = state;

        self.update_edge(idx, edge_weight);
    }
}
