// use itertools::Itertools;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashSet;

use crate::piece::Piece;

fn get_valid_directions(source: NodeIndex) -> Vec<i32> {
    let mut result = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let mut invalid = HashSet::new();
    if source.index() % 8 == 0 {
        invalid.insert(3);
        invalid.insert(4);
        invalid.insert(5);
    } else if source.index() % 8 == 7 {
        invalid.insert(0);
        invalid.insert(1);
        invalid.insert(7);
    }
    if source.index() <= 7 {
        invalid.insert(5);
        invalid.insert(6);
        invalid.insert(7);
    } else if source.index() >= 56 {
        invalid.insert(1);
        invalid.insert(2);
        invalid.insert(3);
    }
    for direction in invalid {
        result.retain(|element| element != &direction);
    }
    return result
}

fn get_node_index_shift(direction: i32) -> i32 {
    let sign = match &direction {
        0..=3 => 1,
        4..=7 => -1,
        _ => 0
    };
    let shift = match direction % 4 {
        0 => 1,
        1 => 9,
        2 => 8,
        3 => 7,
        _ => 0
    };
    return shift * sign
}

#[derive(Debug)]
pub struct GraphBoard {
    pub board_graph: Graph::<Option<Piece>, i32>,
    pub num_directions: i32
}

impl GraphBoard {
    pub fn empty_traditional() -> GraphBoard {
        let mut board_graph = Graph::<Option<Piece>, i32>::new();
        let num_directions = 8;
        for _node in 0..64 {
            board_graph.add_node(None);
        }
        for node_idx in board_graph.node_indices() {
            for direction in get_valid_directions(node_idx) {
                let other_idx = NodeIndex::from((node_idx.index() as i32 + get_node_index_shift(direction)) as u32);
                board_graph.add_edge(node_idx, other_idx, direction);
            }
        }
        return GraphBoard { board_graph, num_directions }
    }

    fn get_next_node_in_direction(&self, source_node: NodeIndex, direction: i32) -> Option<NodeIndex> {
        self.board_graph.edges_directed(source_node, petgraph::Direction::Outgoing)
            .find(|edge| edge.weight() == &direction)
            .map(|edge| edge.target())
    }

    pub fn knight_move_from(&self, source_node: NodeIndex) -> HashSet<NodeIndex> {
        let mut result: HashSet<NodeIndex> = HashSet::new();
        for direction in 0..self.num_directions {
            if let Some(next_node) = self.get_next_node_in_direction(source_node, direction) {
                for next_direction in [(direction - 1) % self.num_directions, (direction + 1) % self.num_directions] {
                    if let Some(final_node) = self.get_next_node_in_direction(next_node, next_direction) {
                        result.insert(final_node);
                    }
                }
            }
        }
        return result
    }

    pub fn slide_move_from(&self, source_node: NodeIndex, direction: i32, limit: u32) -> HashSet<NodeIndex> {
        let mut result: HashSet<NodeIndex> = HashSet::new();
        let mut current_node = source_node;
        let mut distance_traveled = 0;

        while let Some(n) = self.get_next_node_in_direction(current_node, direction) {
            result.insert(n);
            distance_traveled += 1;
            if distance_traveled == limit {
                break
            }
            current_node = n;
        }
        return result
    }

    // pub fn new_traditional() -> Board {
    //     let result = Board::new();
    //     *result.board_graph.node_weight_mut(0.into()).unwrap() = Some(Piece::Rook);
    //     *result.board_graph.node_weight_mut(0.into()).unwrap() = Some(Piece::Rook);
    //     *result.board_graph.node_weight_mut(0.into()).unwrap() = Some(Piece::Rook);
    //     *result.board_graph.node_weight_mut(0.into()).unwrap() = Some(Piece::Rook);
    //     return result
    // }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_board() -> GraphBoard {
        return GraphBoard::empty_traditional()
    }

    #[test]
    fn test_get_next_node_in_direction_returns_node() {
        let board = test_board();
        assert_eq!(
            board.get_next_node_in_direction(NodeIndex::new(0), 0).unwrap(),
            NodeIndex::new(1)
        );
    }

    #[test]
    fn test_get_next_node_in_direction_returns_none() {
        let board = test_board();
        assert_eq!(
            board.get_next_node_in_direction(NodeIndex::new(0), 6),
            None
        )
    }

    #[test]
    fn test_knight_move_from() {
        let board = test_board();
        let source_node = 27;
        assert_eq!(
            board.knight_move_from(NodeIndex::new(source_node)),
            HashSet::from_iter(
                [
                    NodeIndex::new(source_node + 10),
                    NodeIndex::new(source_node - 10),
                    NodeIndex::new(source_node + 6),
                    NodeIndex::new(source_node - 6),
                    NodeIndex::new(source_node + 17),
                    NodeIndex::new(source_node - 17),
                    NodeIndex::new(source_node + 15),
                    NodeIndex::new(source_node - 15)
                ]
            )
        )
    }

    #[test]
    fn test_slide_move_from_no_limit() {
        let board = test_board();
        let source_node = 1;
        assert_eq!(
            board.slide_move_from(NodeIndex::new(source_node), 0, 0),
            HashSet::from_iter(
                [
                    NodeIndex::new(2),
                    NodeIndex::new(3),
                    NodeIndex::new(4),
                    NodeIndex::new(5),
                    NodeIndex::new(6),
                    NodeIndex::new(7),
                ]
            )
        )
    }

    #[test]
    fn test_slide_move_with_limit() {
        let board = test_board();
        let source_node = 1;
        assert_eq!(
            board.slide_move_from(NodeIndex::new(source_node), 0, 1),
            HashSet::from_iter([NodeIndex::new(2)])
        )
    }
}
