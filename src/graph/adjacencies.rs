use crate::graph::Node;
use crate::types::edge::EdgeDB;
use crate::types::U256;
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
    pub fn bfs(&mut self, source: &Node<'a>, target: &Node<'a>) -> bool {
        self.level.clear();
        self.level.insert(source.clone(), 0);
        let mut queue = VecDeque::new();
        queue.push_back(source.clone());

        while !queue.is_empty() {
            let current = queue.pop_front().unwrap();
            for (neighbor, _) in self.outgoing_edges_sorted_by_capacity(&current) {
                if !self.level.contains_key(&neighbor) && self.is_adjacent(&current, &neighbor) {
                    self.level
                        .insert(neighbor.clone(), self.level[&current] + 1);
                    queue.push_back(neighbor);
                }
            }
        }

        self.level.contains_key(target)
    }

    /// Find an augmenting path using DFS.
    pub fn dfs(&mut self, node: &Node<'a>, target: &Node<'a>, flow: U256) -> U256 {
        if node == target {
            return flow;
        }

        while let Some(&(neighbor, capacity)) = self
            .outgoing_edges_sorted_by_capacity(node)
            .get(self.ptr[&node])
        {
            if self.level[&neighbor] == self.level[node] + 1 && capacity > U256::from(0) {
                let current_flow = self.dfs(&neighbor, target, std::cmp::min(flow, capacity));
                if current_flow > U256::from(0) {
                    self.adjust_capacity(node, &neighbor, -current_flow);
                    self.adjust_capacity(&neighbor, node, current_flow);
                    return current_flow;
                }
            }
            self.ptr.insert(node.clone(), self.ptr[&node] + 1);
        }

        U256::from(0)
    }

    /// Main function for the Dinic algorithm.
    pub fn dinic_max_flow(&mut self, source: &Node<'a>, target: &Node<'a>) -> U256 {
        let mut max_flow = U256::from(0);
        while self.bfs(source, target) {
            self.ptr.clear();
            for node in self.edges.keys() {
                self.ptr.insert(node.clone(), 0);
            }

            let mut flow = self.dfs(source, target, U256::MAX);
            while flow != U256::from(0) {
                max_flow += flow;
                flow = self.dfs(source, target, U256::MAX);
            }
        }

        max_flow
    }
}
