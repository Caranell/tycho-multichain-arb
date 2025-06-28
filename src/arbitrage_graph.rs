use crate::types::{ArbitrageGraph, PriceEdge, Protocol, TokenNode};
use crate::utils::graph::GraphIndexUpdateTrait;
use petgraph::graph::DiGraph;
use petgraph::prelude::{EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use tycho_common::{Bytes, models::Chain};
use tycho_simulation::protocol::models::{BlockUpdate, ProtocolComponent};
use tycho_simulation::{models::Token, protocol::state::ProtocolSim};

impl ArbitrageGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
        }
    }

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
            self.graph.add_node(TokenNode {
                symbol,
                tokens: tokens_map,
            });
        }
    }

    pub fn add_edge(&mut self, edge: PriceEdge, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, edge);
    }

    pub fn update_edge(&mut self, index: EdgeIndex, edge: PriceEdge) {
        self.graph.update_edge_by_index(index, edge);
    }

    pub fn handle_block_update(&mut self, msg: BlockUpdate) {
        for (_, pair) in msg.new_pairs {
            let state = msg.states.get(&pair.id.to_string()).unwrap().clone();
            self.handle_new_pair(pair, state);
        }

        for (address, state) in msg.states {
            self.handle_state_update(state, address);
        }

        self.format_edges();
    }

    pub fn format_edges(&self) {
        // Collect edge information (from_symbol, to_symbol, chain, protocol, price)
        let mut edges_info: Vec<(String, String, Chain, Protocol, f64, String)> = self
            .graph
            .edge_references()
            .map(|edge_ref| {
                let edge = edge_ref.weight();
                (
                    edge.from_token.symbol.clone(),
                    edge.to_token.symbol.clone(),
                    edge.chain,
                    edge.protocol,
                    edge.price,
                    edge.pool_address.clone(),
                )
            })
            .collect();

        // Sort first by `from` token symbol, then by `to` token symbol
        edges_info.sort_by(|a, b| match a.0.cmp(&b.0) {
            std::cmp::Ordering::Equal => a.1.cmp(&b.1),
            other => other,
        });

        // Print formatted edges
        for (from_symbol, to_symbol, chain, protocol, price, pool_address) in edges_info {
            println!(
                "{} -> {} | Chain: {:?} | Protocol: {} | Pool: {} | Price: {:.6}",
                from_symbol,
                to_symbol,
                chain,
                protocol.to_str(),
                pool_address,
                price
            );
        }
    }

    pub fn handle_new_pair(&mut self, pair: ProtocolComponent, state: Box<dyn ProtocolSim>) {
        let from_node = self
            .graph
            .node_indices()
            .find(|&i| self.graph.node_weight(i).unwrap().symbol == pair.tokens[0].symbol);
        let to_node = self
            .graph
            .node_indices()
            .find(|&i| self.graph.node_weight(i).unwrap().symbol == pair.tokens[1].symbol);

        self.add_edge(
            PriceEdge {
                chain: pair.chain,
                protocol: Protocol::from_str(&pair.protocol_system).unwrap(),
                price: state.spot_price(&pair.tokens[0], &pair.tokens[1]).unwrap(),
                state: state.clone(),
                pool_address: pair.id.to_string(),
                to_token: pair.tokens[0].clone(),
                from_token: pair.tokens[1].clone(),
            },
            from_node.unwrap(),
            to_node.unwrap(),
        );

        self.add_edge(
            PriceEdge {
                chain: pair.chain,
                protocol: Protocol::from_str(&pair.protocol_system).unwrap(),
                price: state.spot_price(&pair.tokens[1], &pair.tokens[0]).unwrap(),
                state: state.clone(),
                pool_address: pair.id.to_string(),
                to_token: pair.tokens[1].clone(),
                from_token: pair.tokens[0].clone(),
            },
            to_node.unwrap(),
            from_node.unwrap(),
        );
    }

    pub fn handle_state_update(&mut self, state: Box<dyn ProtocolSim>, address: String) {
        let edge_indices: Vec<EdgeIndex> = self
            .graph
            .edge_references()
            .filter(|e| e.weight().pool_address == address)
            .map(|e| e.id())
            .collect();

        println!("UPDATING EDGES: {:?}", edge_indices.len());
        let start = std::time::Instant::now();
        // TODO: ONLY UPDATE THE EDGES THAT ARE AFFECTED BY THE STATE UPDATE
        for idx in edge_indices {
            self.update_edge_weight(idx, state.clone());
        }
        let duration = start.elapsed();
        println!("Time taken: {:?}", duration);
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
