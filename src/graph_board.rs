use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashSet, HashMap};

use crate::piece::Piece;
use crate::bit_board::{BitBoard, CarryRippler};

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

    pub fn knight_jump_from(&self, source_node: NodeIndex) -> HashSet<NodeIndex> {
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

    pub fn slide_from_in_direction(&self, source_node: NodeIndex, direction: i32, limit: u32, obstructions: BitBoard) -> HashSet<NodeIndex> {
        let mut result: HashSet<NodeIndex> = HashSet::new();
        let mut current_node = source_node;
        let mut distance_traveled = 0;

        while let Some(n) = self.get_next_node_in_direction(current_node, direction) {
            if BitBoard::new(1 << n.index()) & obstructions != BitBoard::empty() {
                break
            }
            result.insert(n);
            distance_traveled += 1;
            if distance_traveled == limit {
                break
            }
            current_node = n;
        }
        return result
    }

    pub fn cast_slides_from(
        &self,
        source_node: NodeIndex,
        obstructions: BitBoard,
        limit: u32,
        diagonals: bool,
        orthogonals: bool
    ) -> HashSet<NodeIndex> {
        
        let initital_direction = match orthogonals {
            true => 0,
            false => 1
        };
        let direction_step = match orthogonals & diagonals {
            true => 1,
            false => 2
        };

        let mut result: HashSet<NodeIndex> = HashSet::new();
        for even_direction in (initital_direction..self.num_directions).step_by(direction_step) {
            result.extend(self.slide_from_in_direction(
                source_node,
                even_direction,
                limit,
                obstructions
            ))
        }
        return result
    }

    pub fn knight_jumps_table(&self) -> Vec<BitBoard> {
        let mut result: Vec<BitBoard> = vec![];
        for source_node in self.board_graph.node_indices() {
            result.push(BitBoard::from_node_indices(self.knight_jump_from(source_node)))
        }
        return result
    }

    pub fn slides_table(
        &self,
        diagonals: bool,
        orthogonals: bool
    ) -> (Vec<BitBoard>, Vec<HashMap<BitBoard, BitBoard>>) {
        let mut mask_table: Vec<BitBoard> = vec![];
        let mut attack_table: Vec<HashMap<BitBoard, BitBoard>> = vec![];
        for source_node in self.board_graph.node_indices() {
            let mask = BitBoard::from_node_indices(
                self.cast_slides_from(
                    source_node,
                    BitBoard::empty(),
                    0,
                    diagonals,
                    orthogonals
                )
            );
            mask_table.push(mask);
            let mut attack_map = HashMap::new();
            for subset in CarryRippler::new(mask) {
                attack_map.insert(
                    subset,
                    BitBoard::from_node_indices(
                        self.cast_slides_from(
                            source_node,
                            subset,
                            0,
                            diagonals,
                            orthogonals
                        )
                    ));
            }
            attack_table.push(attack_map);
        }
        return (mask_table, attack_table)
    }

    pub fn diagonal_slides_table(&self) -> (Vec<BitBoard>, Vec<HashMap<BitBoard, BitBoard>>) {
        return self.slides_table(true, false)
    }

    pub fn orthogonal_slides_table(&self) -> (Vec<BitBoard>, Vec<HashMap<BitBoard, BitBoard>>) {
        return self.slides_table(false, true)
    }

    pub fn king_move_table(&self) -> Vec<BitBoard> {
        let mut result: Vec<BitBoard> = vec![];
        for source_node in self.board_graph.node_indices() {
            result.push(BitBoard::from_node_indices(self.cast_slides_from(
                source_node,
                BitBoard::empty(),
                1,
                true,
                true
            )))
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
    use petgraph::graph::Node;

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
        let source_node = NodeIndex::new(27);
        assert_eq!(
            board.knight_jump_from(source_node),
            HashSet::from_iter([
                NodeIndex::new(27 + 10),
                NodeIndex::new(27 - 10),
                NodeIndex::new(27 + 6),
                NodeIndex::new(27 - 6),
                NodeIndex::new(27 + 17),
                NodeIndex::new(27 - 17),
                NodeIndex::new(27 + 15),
                NodeIndex::new(27 - 15)
            ])
        )
    }

    #[test]
    fn test_slide_move_from_no_limit_no_obstructions() {
        let board = test_board();
        let source_node = NodeIndex::new(1);
        assert_eq!(
            board.slide_from_in_direction(source_node, 0, 0, BitBoard::empty()),
            HashSet::from_iter([
                NodeIndex::new(2),
                NodeIndex::new(3),
                NodeIndex::new(4),
                NodeIndex::new(5),
                NodeIndex::new(6),
                NodeIndex::new(7),
            ])
        )
    }

    #[test]
    fn test_slide_move_with_limit() {
        let board = test_board();
        let source_node = NodeIndex::new(1);
        assert_eq!(
            board.slide_from_in_direction(source_node, 0, 1, BitBoard::empty()),
            HashSet::from_iter([NodeIndex::new(2)])
        )
    }

    #[test]
    fn test_slide_move_with_obstructions() {
        let board = test_board();
        let source_node = NodeIndex::new(1);
        let obstructions = BitBoard::new(32);
        assert_eq!(
            board.slide_from_in_direction(source_node, 0, 0, obstructions),
            HashSet::from_iter([
                NodeIndex::new(2),
                NodeIndex::new(3),
                NodeIndex::new(4)
            ])
        )
    }

    #[test]
    fn test_diagonal_slides_unobstructed() {
        let board = test_board();
        let source_node = NodeIndex::new(27);
        assert_eq!(
            board.cast_slides_from(source_node, BitBoard::empty(), 0, true, false),
            HashSet::from_iter([    
                NodeIndex::new(0),
                NodeIndex::new(9),
                NodeIndex::new(18),
                NodeIndex::new(36),
                NodeIndex::new(45),
                NodeIndex::new(54),
                NodeIndex::new(63),
                NodeIndex::new(34),
                NodeIndex::new(41),
                NodeIndex::new(48),
                NodeIndex::new(20),
                NodeIndex::new(13),
                NodeIndex::new(6)
            ])
        )
    }

    #[test]
    fn test_diagonal_slides_obstructed() {
        let board = test_board();
        let source_node = NodeIndex::new(27);
        let blockers = BitBoard::from_node_indices(HashSet::from_iter([
            NodeIndex::new(36),
            NodeIndex::new(34),
            NodeIndex::new(20)
        ]));
        assert_eq!(
            board.cast_slides_from(source_node, blockers, 0, true, false),
            HashSet::from_iter([    
                NodeIndex::new(0),
                NodeIndex::new(9),
                NodeIndex::new(18),
            ])
        )
    }

    #[test]
    fn test_orthogonal_slides_unobstructed() {
        let board = test_board();
        let source_node = NodeIndex::new(27);
        assert_eq!(
            board.cast_slides_from(source_node, BitBoard::empty(), 0, false, true),
            HashSet::from_iter([    
                NodeIndex::new(24),
                NodeIndex::new(25),
                NodeIndex::new(26),
                NodeIndex::new(28),
                NodeIndex::new(29),
                NodeIndex::new(30),
                NodeIndex::new(31),
                NodeIndex::new(3),
                NodeIndex::new(19),
                NodeIndex::new(11),
                NodeIndex::new(35),
                NodeIndex::new(43),
                NodeIndex::new(51),
                NodeIndex::new(59)
            ])
        )
    }

    #[test]
    fn test_both_orthogonal_slides_unobstructed() {
        let board = test_board();
        let source_node = NodeIndex::new(27);
        assert_eq!(
            board.cast_slides_from(source_node, BitBoard::empty(), 0, true, true),
            HashSet::from_iter([    
                NodeIndex::new(24),
                NodeIndex::new(25),
                NodeIndex::new(26),
                NodeIndex::new(28),
                NodeIndex::new(29),
                NodeIndex::new(30),
                NodeIndex::new(31),
                NodeIndex::new(3),
                NodeIndex::new(19),
                NodeIndex::new(11),
                NodeIndex::new(35),
                NodeIndex::new(43),
                NodeIndex::new(51),
                NodeIndex::new(59),
                NodeIndex::new(0),
                NodeIndex::new(9),
                NodeIndex::new(18),
                NodeIndex::new(36),
                NodeIndex::new(45),
                NodeIndex::new(54),
                NodeIndex::new(63),
                NodeIndex::new(34),
                NodeIndex::new(41),
                NodeIndex::new(48),
                NodeIndex::new(20),
                NodeIndex::new(13),
                NodeIndex::new(6)
            ])
        )
    }

    #[test]
    fn test_cast_slides_with_limit() {
        let board = test_board();
        let source_node = NodeIndex::new(27);
        assert_eq!(
            board.cast_slides_from(source_node, BitBoard::empty(), 1, true, true),
            HashSet::from_iter([
                NodeIndex::new(36),
                NodeIndex::new(35),
                NodeIndex::new(34),
                NodeIndex::new(28),
                NodeIndex::new(26),
                NodeIndex::new(20),
                NodeIndex::new(19),
                NodeIndex::new(18),
            ])
        )
    }

    #[test]
    fn test_knight_table() {
        let board = test_board();
        assert_eq!(
            board.knight_jumps_table()[63], // Only testing last node
            BitBoard::from_node_indices(HashSet::from_iter([
                NodeIndex::new(53),
                NodeIndex::new(46)
            ]))
        )
    }

    #[test]
    fn test_diagonal_table() {
        let board = test_board();
        let diag_slides_table = board.diagonal_slides_table();
        let mask = diag_slides_table.0;
        assert_eq!(
            mask[63], // Only testing last node
            BitBoard::from_node_indices(HashSet::from_iter([
                NodeIndex::new(54),
                NodeIndex::new(45),
                NodeIndex::new(36),
                NodeIndex::new(27),
                NodeIndex::new(18),
                NodeIndex::new(9),
                NodeIndex::new(0),
            ]))
        );
        let blocked_map = diag_slides_table.1[63].clone();
        assert_eq!(
            *blocked_map.get(&BitBoard::from_node_indices(HashSet::from_iter([
                NodeIndex::new(45)
            ]))).unwrap(),
            BitBoard::from_node_indices(HashSet::from_iter([
                NodeIndex::new(54)
            ]))
        )
    }

    #[test]
    fn test_orthogonal_table() {
        let board = test_board();
        let orthog_slides_table = board.orthogonal_slides_table();
        let mask = orthog_slides_table.0;
        assert_eq!(
            mask[63], // Only testing last node
            BitBoard::from_node_indices(HashSet::from_iter([
                NodeIndex::new(62),
                NodeIndex::new(61),
                NodeIndex::new(60),
                NodeIndex::new(59),
                NodeIndex::new(58),
                NodeIndex::new(57),
                NodeIndex::new(56),
                NodeIndex::new(55),
                NodeIndex::new(47),
                NodeIndex::new(39),
                NodeIndex::new(31),
                NodeIndex::new(23),
                NodeIndex::new(15),
                NodeIndex::new(7),
            ]))
        );
        let blocked_map = orthog_slides_table.1[63].clone();
        assert_eq!(
            *blocked_map.get(&BitBoard::from_node_indices(HashSet::from_iter([
                NodeIndex::new(62)
            ]))).unwrap(),
            BitBoard::from_node_indices(HashSet::from_iter([
                NodeIndex::new(55),
                NodeIndex::new(47),
                NodeIndex::new(39),
                NodeIndex::new(31),
                NodeIndex::new(23),
                NodeIndex::new(15),
                NodeIndex::new(7),
            ]))
        )
    }

    #[test]
    fn test_king_table() {
        let board = test_board();
        assert_eq!(
            board.king_move_table()[63], // Only testing last node
            BitBoard::from_node_indices(HashSet::from_iter([
                NodeIndex::new(62),
                NodeIndex::new(55),
                NodeIndex::new(54)
            ]))
        )
    }
}
