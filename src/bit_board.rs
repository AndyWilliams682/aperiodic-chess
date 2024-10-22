use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashSet;

use crate::piece::Piece;


#[derive(Debug, PartialEq, Eq)]
pub struct BitBoard(u64);

impl BitBoard {
    pub fn from_node_indices(node_indices: HashSet<NodeIndex>) -> BitBoard {
        let mut output: u64 = 0;
        for node in node_indices {
            output += 1 << node.index();
        }
        return BitBoard(output)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        assert_eq!(
            BitBoard::from_node_indices(HashSet::from_iter([NodeIndex::new(0), NodeIndex::new(25)])),
            BitBoard(33554433)
        )
    }
}
