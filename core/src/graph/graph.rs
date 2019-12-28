use crate::{Repodata, Record};
use petgraph::graph::DiGraph;
use petgraph::visit::IntoNodeReferences;

use crate::graph::combine::ComboMethod;


pub fn extend_graph_with_repodata(g: &mut DiGraph<&Record, i16>, repodata: &Repodata) {
    for collection in (&repodata.packages, &repodata.packages_conda) {
        for (pkg_name, pkg_dict) in collection.iter() {
            g.add_node(pkg_dict);
        }
    }
}

pub fn resolve_edges(g: &mut DiGraph<&Record, i16>) {
    for (idx, node) in g.node_references() {
        for matchspec in node.depends.iter() {
            // match package name and version with other packages
        }
    }
}

pub fn populate_graph(repodatas: Vec<&Repodata>, combo_method: ComboMethod) -> DiGraph<&Record, i16> {
    // TODO: make nodes/edges configurable, or auto-scale based on repodata input size
    let mut graph = DiGraph::with_capacity(50_000, 1_000_000);
    for repodata in repodatas {
        extend_graph_with_repodata(&graph, repodata)
    }
    graph
}