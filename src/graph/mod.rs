use crate::types::Address;
use std::fmt::{Display, Formatter};

mod adjacencies;
mod flow;

// An edge from the capacity network is represented as:
// from, token, to -> capacity
//
// In the transformation into the flow network, two intermediate nodes are added
// per edge that might be shared with other edges:
//
// from -A-> BalanceNode(from) -B-> TrustNode(to, token) -C-> to
//
// The capacities (A, B, C) are defined as:
// A: the max of all capacity-network edges of the form (from, token, *) or A's balance of "token" tokens.
// B: the actual capacity of the capacity-network edge (from, token, to) or the "send limit" from "from" to "to" in "token" tokens.
// C: if "token" is C's token (this is a "send to owner" edge): infinity or the sum of all incoming edges.
//    otherwise: the max of all capacity-network edges of the form (*, token, to) or the trust limit of "to" for "token" tokens.

#[derive(Debug, Eq, PartialEq, Hash, Clone, PartialOrd, Ord)]
pub enum Node<'a> {
    /// Represents a trust relationship between two addresses for a specific token.
    TrustNode(&'a Address, &'a Address),
    /// Represents the balance of an address.
    BalanceNode(&'a Address),
}

/// Retrieves the address from a node.
/// For a BalanceNode, it returns the associated address.
/// For a TrustNode, it returns the "to" address.
pub fn node_as_address<'a>(node: &Node<'a>) -> &'a Address {
    match node {
        Node::BalanceNode(address) => address,
        Node::TrustNode(to, _) => to,
    }
}

/// Extracts the addresses from a TrustNode.
/// Returns a tuple containing the "to" address and the token address.
pub fn as_trust_node<'a>(node: &Node<'a>) -> (&'a Address, &'a Address) {
    if let Node::TrustNode(to, token) = node {
        (to, token)
    } else {
        panic!("Expected a TrustNode variant")
    }
}

impl<'a> Display for Node<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Node::BalanceNode(address) => write!(f, "{}", address),
            Node::TrustNode(to, token) => write!(f, "(trust {} x {})", to, token),
        }
    }
}

pub use crate::graph::flow::compute_flow;
pub use crate::graph::flow::transfers_to_dot;
