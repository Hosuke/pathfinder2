use crate::graph::Node;
use crate::types::edge::EdgeDB;
use crate::types::{Edge, U256};
use std::cmp::{max, Reverse};
use std::collections::{HashMap, VecDeque};

pub struct Adjacencies<'a> {
    edges: &'a EdgeDB,
    lazy_adjacencies: HashMap<Node, HashMap<Node, U256>>,
    capacity_adjustments: HashMap<Node, HashMap<Node, U256>>,
}

// fn pseudo_node(edge: Edge) -> Node {
//     Node::TokenEdge(edge.from, edge.token)
// }

fn balance_node(edge: &Edge) -> Node {
    Node::BalanceNode(edge.from, edge.token)
}

fn trust_node(edge: &Edge) -> Node {
    Node::TrustNode(edge.to, edge.token)
}

// fn source_address_of(node: &Node) -> &Address {
//     match node {
//         Node::Node(addr) => addr,
//         Node::TokenEdge(from, _) => from,
//     }
// }

impl<'a> Adjacencies<'a> {
    pub fn new(edges: &'a EdgeDB) -> Self {
        Adjacencies {
            edges,
            lazy_adjacencies: HashMap::new(),
            capacity_adjustments: HashMap::new(),
        }
    }

    /// Uses Breadth-First Search (BFS) to construct a level graph from the source to the sink.
    ///
    /// This function explores the flow network and assigns a level to each node based on its distance
    /// from the source. Nodes that are unreachable from the source in the residual network will not be
    /// assigned a level. The level graph is used in the Dinic algorithm to find blocking flows.
    ///
    /// # Arguments
    ///
    /// * `source` - The source node of the flow network.
    /// * `sink` - The sink node of the flow network.
    /// * `max_distance` - An optional maximum distance constraint.
    ///
    /// # Returns
    ///
    /// * `Some(HashMap<Node, usize>)` - A HashMap containing the levels of each node if there exists a path from the source to the sink.
    /// * `None` - If no path from the source to the sink is found in the residual network.
    pub fn bfs_level_graph(
        &mut self,
        source: &Node,
        sink: &Node,
        max_distance: Option<u64>,
    ) -> Option<HashMap<Node, usize>> {
        let mut levels: HashMap<Node, usize> = HashMap::new(); // Initialize levels of all nodes
        let mut queue = VecDeque::new();

        levels.insert(source.clone(), 0); // Set level of source node to 0
        queue.push_back(source.clone());

        while let Some(current) = queue.pop_front() {
            if let Some(max_dist) = max_distance {
                if levels[&current] >= max_dist as usize {
                    continue; // Skip exploring neighbors if current distance exceeds max_distance
                }
            }
            let neighbors_and_capacities = self.adjacencies_from(&current);
            for (neighbor, capacity) in neighbors_and_capacities {
                if !levels.contains_key(&neighbor) && capacity > U256::from(0) {
                    levels.insert(neighbor.clone(), levels[&current] + 1);
                    queue.push_back(neighbor);
                }
            }
        }

        if levels.contains_key(sink) {
            Some(levels)
        } else {
            None // If the level of the sink is not found, no path from source to sink was found
        }
    }

    /// Performs a Depth-First Search (DFS) on the level graph to find a blocking flow.
    ///
    /// This function searches for augmenting paths in the level graph. It ensures that we only consider
    /// edges that go from a node to a node of a higher level. The search stops when a blocking flow is found.
    ///
    /// # Arguments
    ///
    /// * `current` - The current node being explored.
    /// * `sink` - The sink node of the flow network.
    /// * `levels` - A reference to a HashMap containing the levels of each node, as determined by BFS.
    /// * `flow` - The current flow value being pushed through the path.
    /// * `flow_distribution` - A mutable reference to a HashMap tracking the flow distribution across edges.
    ///
    /// # Returns
    ///
    /// * `Some(U256)` - The flow value of the found blocking flow.
    /// * `None` - If no blocking flow is found from the current node to the sink.
    pub fn dfs_search_blocking_flow(
        &mut self,
        current: &Node,
        sink: &Node,
        levels: &HashMap<Node, usize>,
        flow: U256,
        flow_distribution: &mut HashMap<Node, HashMap<Node, U256>>,
    ) -> Option<U256> {
        if current == sink {
            return Some(flow);
        }

        let neighbors_and_capacities = self.adjacencies_from(current);
        for (neighbor, capacity) in neighbors_and_capacities {
            if levels[&neighbor] == levels[current] + 1 && capacity > U256::from(0) {
                // Compute the flow for current branch
                let new_flow = U256::min(flow, capacity);

                // Recursive call
                if let Some(path_flow) = self.dfs_search_blocking_flow(
                    &neighbor,
                    sink,
                    levels,
                    new_flow,
                    flow_distribution,
                ) {
                    // Update capacities in the residual network
                    self.adjust_capacity(current, &neighbor, -path_flow);
                    self.adjust_capacity(&neighbor, current, path_flow);

                    // Update flow distribution
                    *flow_distribution
                        .entry(current.clone())
                        .or_default()
                        .entry(neighbor.clone())
                        .or_insert(U256::from(0)) += path_flow;

                    return Some(path_flow);
                }
            }
        }

        None
    }

    #[allow(dead_code)]
    pub fn outgoing_edges_sorted_by_capacity(&mut self, from: &Node) -> Vec<(Node, U256)> {
        let mut adjacencies = self.adjacencies_from(from);
        if let Some(adjustments) = self.capacity_adjustments.get(from) {
            for (node, c) in adjustments {
                *adjacencies.entry(node.clone()).or_default() += *c;
            }
        }
        let mut result = adjacencies
            .into_iter()
            .filter(|(_, cap)| *cap != U256::from(0))
            .collect::<Vec<(Node, U256)>>();
        result.sort_unstable_by_key(|(addr, capacity)| (Reverse(*capacity), addr.clone()));
        result
    }

    pub fn adjust_capacity(&mut self, from: &Node, to: &Node, adjustment: U256) {
        *self
            .capacity_adjustments
            .entry(from.clone())
            .or_default()
            .entry(to.clone())
            .or_default() += adjustment;
    }

    #[allow(clippy::wrong_self_convention)]
    #[allow(dead_code)]
    pub fn is_adjacent(&mut self, from: &Node, to: &Node) -> bool {
        // TODO More efficiently?
        if let Some(capacity) = self.adjacencies_from(from).get(to) {
            *capacity > U256::from(0)
        } else {
            false
        }
    }

    fn adjacencies_from(&mut self, from: &Node) -> HashMap<Node, U256> {
        self.lazy_adjacencies
            .entry(from.clone())
            .or_insert_with(|| {
                let mut result: HashMap<Node, U256> = HashMap::new();
                // Plain edges are (from, to, token) labeled with capacity
                match from {
                    Node::Node(from) => {
                        for edge in self.edges.outgoing(from) {
                            // One edge from "from" to "from x token" with a capacity
                            // as the max over all "to" addresses (the balance of the sender)
                            result
                                .entry(balance_node(edge))
                                .and_modify(|c| {
                                    if edge.capacity > *c {
                                        *c = edge.capacity;
                                    }
                                })
                                .or_insert(edge.capacity);
                        }
                    }
                    Node::BalanceNode(from, token) => {
                        for edge in self.edges.outgoing(from) {
                            // The actual capacity of the edge / the send limit.
                            if edge.from == *from && edge.token == *token {
                                result.insert(trust_node(edge), edge.capacity);
                            }
                        }
                    }
                    Node::TrustNode(to, token) => {
                        let is_return_to_owner = *to == *token;
                        // If token is to's token: send back to owner, infinite capacity.
                        // Otherwise, the max of the incoming edges (the trust limit)
                        let mut capacity = U256::from(0);
                        for edge in self.edges.incoming(to) {
                            if edge.token == *token {
                                if is_return_to_owner {
                                    capacity += edge.capacity
                                } else {
                                    capacity = max(capacity, edge.capacity)
                                }
                            }
                            result.insert(Node::Node(*to), capacity);
                        }
                    }
                }
                result
            })
            .clone()
    }
}
