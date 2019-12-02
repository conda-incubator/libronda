use crate::{Repodata, Record};
use petgraph::graph::DiGraph;

use crate::graph::combine::ComboMethod;


pub fn extend_graph_with_repodata(g: &mut DiGraph<&str, &Record>, repodata: &Repodata) {
    ()
}

pub fn populate_graph(repodatas: Vec<&Repodata>, combo_method: ComboMethod) -> DiGraph<&str, &Record> {
    // TODO: make nodes/edges configurable, or auto-scale based on repodata input size
    let mut graph = DiGraph::with_capacity(50_000, 1_000_000);
    graph
}