use petgraph::graph::DiGraph;
use petgraph::prelude::EdgeIndex;

pub trait GraphIndexUpdateTrait<T, E> {
  fn update_edge_by_index(&mut self, index: EdgeIndex, edge: E);
}

impl<T,E> GraphIndexUpdateTrait<T, E> for DiGraph<T, E> {
  fn update_edge_by_index(&mut self, index: EdgeIndex, edge: E) {
      let current_weight = self.edge_weight_mut(index).unwrap();
      *current_weight = edge;
  }
}