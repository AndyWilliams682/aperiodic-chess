use petgraph::graph::{Graph, NodeIndex};

use crate::piece::Piece;


#[derive(Debug, PartialEq, Eq)]
pub struct BitBoard(u64);

impl BitBoard {
    pub fn from_node_indices(node_indices: Vec<NodeIndex>) -> BitBoard {
        let mut output: u64 = 0;
        for node in node_indices {
            output += 1 << node.index();
        }
        return BitBoard(output)
    }

    pub fn get_knight_movement_bbs(num_directions: i32, graph: Graph<Piece, i32>) -> BitBoard {
        let mut output: u64 = 0;
        for node in graph.node_indices() {
            for direction in 0..num_directions {
                
            }
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
            BitBoard::from_node_indices(vec![NodeIndex::new(0), NodeIndex::new(25)]),
            BitBoard(33554433)
        )
    }
}
