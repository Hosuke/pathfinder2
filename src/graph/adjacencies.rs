use crate::graph::Node;
use crate::types::edge::EdgeDB;
use crate::types::U256;
use std::cmp::min;
use std::collections::{HashMap, VecDeque};

pub struct Adjacencies<'a> {
    edges: HashMap<Node<'a>, HashMap<Node<'a>, U256>>,
    level: HashMap<Node<'a>, usize>,
    ptr: HashMap<Node<'a>, usize>,
}

impl<'a> Adjacencies<'a> {
    /// Create a new Adjacencies structure from the given EdgeDB.
    pub fn new(edges: &EdgeDB<'a>) -> Self {
        let mut adjacencies = Adjacencies {
            edges: HashMap::new(),
            level: HashMap::new(),
            ptr: HashMap::new(),
        };

        // Initialization code: Populate the edges based on the EdgeDB.
        // For each edge in the EdgeDB, extract the 'from' and 'to' nodes and the capacity.
        // Then, update the 'edges' HashMap in the Adjacencies structure.
        for edge in edges.iter() {
            adjacencies
                .edges
                .entry(edge.from.clone())
                .or_default()
                .insert(edge.to.clone(), edge.capacity);
        }

        adjacencies
    }

    /// Get all outgoing edges from a node, sorted by their capacity.
    pub fn outgoing_edges_sorted_by_capacity(&self, node: &Node<'a>) -> Vec<(Node<'a>, U256)> {
        let mut edges: Vec<_> = self
            .edges
            .get(node)
            .unwrap_or(&HashMap::new())
            .clone()
            .into_iter()
            .collect();
        edges.sort_by_key(|(_, capacity)| *capacity);
        edges.reverse();
        edges
    }

    /// Adjust the capacity between two nodes by a given amount.
    pub fn adjust_capacity(&mut self, from: &Node<'a>, to: &Node<'a>, amount: U256) {
        if let Some(outgoing) = self.edges.get_mut(from) {
            if let Some(capacity) = outgoing.get_mut(to) {
                *capacity += amount;
            }
        }
    }

    /// Check if two nodes are adjacent.
    pub fn is_adjacent(&self, from: &Node<'a>, to: &Node<'a>) -> bool {
        if let Some(outgoing) = self.edges.get(from) {
            outgoing.contains_key(to)
        } else {
            false
        }
    }

    /// Build a level graph using BFS.
    fn bfs(
        &self,
        source: &Node,
        sink: &Node,
        max_distance: Option<u64>,
    ) -> HashMap<Node<'a>, usize> {
        let mut level = HashMap::new();
        let mut queue = VecDeque::new();
        level.insert(source.clone(), 0);
        queue.push_back(source.clone());

        while let Some(node) = queue.pop_front() {
            if let Some(max) = max_distance {
                if level[&node] >= max as usize {
                    continue;
                }
            }

            for (next, &capacity) in self.edges[&node].iter() {
                if capacity > U256::from(0) && !level.contains_key(next) {
                    level.insert(next.clone(), level[&node] + 1);
                    queue.push_back(next.clone());
                }
            }
        }

        level
    }

    /// Find an augmenting path using DFS.
    fn dfs(
        &mut self,
        node: &Node,
        sink: &Node,
        flow: U256,
        level: &HashMap<Node<'a>, usize>,
    ) -> U256 {
        if node == sink {
            return flow;
        }

        for (next, capacity) in self.edges[node].iter_mut() {
            if *capacity > U256::from(0) && level[node] + 1 == level[next] {
                let current_flow = min(flow, *capacity);
                let temp_flow = self.dfs(next, sink, current_flow, level);
                if temp_flow > U256::from(0) {
                    *capacity -= temp_flow;
                    if let Some(reverse_capacity) = self.edges[next].get_mut(node) {
                        *reverse_capacity += temp_flow;
                    } else {
                        self.edges[next].insert(node.clone(), temp_flow);
                    }
                    return temp_flow;
                }
            }
        }

        U256::from(0)
    }

    /// Main function for the Dinic algorithm.
    pub fn dinic_max_flow(
        &mut self,
        source: &Node<'a>,
        target: &Node<'a>,
        max_distance: Option<u64>,
    ) -> U256 {
        let mut max_flow = U256::from(0);
        while let level = self.bfs(source, target, max_distance) {
            self.ptr.clear();
            for node in self.edges.keys() {
                self.ptr.insert(node.clone(), 0);
            }

            let mut flow = self.dfs(source, target, U256::MAX, &level);
            while flow != U256::from(0) {
                max_flow += flow;
                flow = self.dfs(source, target, U256::MAX, &level);
            }
        }

        max_flow
    }
}
