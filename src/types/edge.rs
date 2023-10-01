use std::collections::HashMap;

use crate::graph::Node;
use crate::types::Address;
use crate::types::U256;

/// Represents an edge in the graph with associated metadata.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct Edge<'a> {
    pub from: Node<'a>,
    pub to: Node<'a>,
    pub token: Address,
    pub capacity: U256,
}

/// Compares two edges for equality, ignoring their capacity.
/// This is useful when checking if an edge already exists in the graph, regardless of its capacity.
pub fn eq_up_to_capacity<'a>(e1: &Edge<'a>, e2: &Edge<'a>) -> bool {
    e1.from == e2.from && e1.to == e2.to && e1.token == e2.token
}

/// Database structure to manage and query edges.
#[derive(Debug, Default, Clone)]
pub struct EdgeDB<'a> {
    edges: Vec<Edge<'a>>,
    outgoing: HashMap<Node<'a>, Vec<usize>>,
    incoming: HashMap<Node<'a>, Vec<usize>>,
}

impl<'a> EdgeDB<'a> {
    /// Constructs a new EdgeDB from a vector of edges.
    pub fn new(edges: Vec<Edge<'a>>) -> EdgeDB<'a> {
        let outgoing = outgoing_index(&edges);
        let incoming = incoming_index(&edges);
        EdgeDB {
            edges,
            outgoing,
            incoming,
        }
    }

    /// Returns an iterator over the edges.
    pub fn iter(&self) -> std::slice::Iter<'_, Edge<'a>> {
        self.edges.iter()
    }

    /// Returns the total number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns a reference to the vector of edges.
    pub fn edges(&self) -> &Vec<Edge<'a>> {
        &self.edges
    }

    /// Updates an edge's capacity or inserts it if it doesn't exist.
    pub fn update(&mut self, update: Edge<'a>) {
        match self.index_of(&update) {
            Some(i) => self.edges[i].capacity = update.capacity,
            None => {
                let i = self.edges.len();
                self.outgoing
                    .entry(update.from.clone())
                    .or_default()
                    .push(i);
                self.incoming.entry(update.to.clone()).or_default().push(i);
                self.edges.push(update);
            }
        }
    }

    /// Returns a vector of outgoing edges from a given source node.
    pub fn outgoing(&self, source: &Node<'a>) -> Vec<&Edge<'a>> {
        match self.outgoing.get(source) {
            Some(out) => out
                .iter()
                .map(|i| self.edges.get(*i).unwrap())
                .filter(|e| e.capacity != U256::from(0))
                .collect(),
            None => vec![],
        }
    }

    /// Returns a vector of incoming edges to a given destination node.
    pub fn incoming(&self, to: &Node<'a>) -> Vec<&Edge<'a>> {
        match self.incoming.get(to) {
            Some(incoming) => incoming
                .iter()
                .map(|i| self.edges.get(*i).unwrap())
                .filter(|e| e.capacity != U256::from(0))
                .collect(),
            None => vec![],
        }
    }

    /// Returns the index of an edge in the edges vector, if it exists.
    fn index_of(&self, e: &Edge<'a>) -> Option<usize> {
        self.outgoing.get(&e.from).and_then(|out| {
            for i in out {
                if eq_up_to_capacity(&self.edges[*i], e) {
                    return Some(*i);
                }
            }
            None
        })
    }
}

/// Constructs an index for outgoing edges.
fn outgoing_index<'a>(edges: &[Edge<'a>]) -> HashMap<Node<'a>, Vec<usize>> {
    let mut index: HashMap<Node<'a>, Vec<usize>> = HashMap::new();
    for (i, e) in edges.iter().enumerate() {
        index.entry(e.from.clone()).or_default().push(i)
    }
    index
}

/// Constructs an index for incoming edges.
fn incoming_index<'a>(edges: &[Edge<'a>]) -> HashMap<Node<'a>, Vec<usize>> {
    let mut index: HashMap<Node<'a>, Vec<usize>> = HashMap::new();
    for (i, e) in edges.iter().enumerate() {
        index.entry(e.to.clone()).or_default().push(i)
    }
    index
}
