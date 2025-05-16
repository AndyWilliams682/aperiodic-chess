use petgraph::graph::NodeIndex;
use std::collections::HashSet;
use std::ops::{Sub, BitAnd, BitOr, Not};

use crate::position::PieceType;
use crate::chess_move::{EnPassantData, Move};


#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct BitBoard(u128);

impl BitBoard {
    pub fn from_node_indices(node_indices: HashSet<NodeIndex>) -> BitBoard {
        let mut result: u128 = 0;
        for node in node_indices {
            result += 1 << node.index();
        }
        return BitBoard(result)
    }

    pub fn from_ints(ints: Vec<u128>) -> BitBoard {
        let mut result: u128 = 0;
        for node in ints {
            result += 1 << node;
        }
        return BitBoard(result)
    }

    pub fn new(n: u128) -> BitBoard {
        return BitBoard(n)
    }

    pub fn empty() -> BitBoard {
        return BitBoard(0)
    }

    pub fn get_bit_at_node(self, node: NodeIndex) -> bool {
        let mask: u128 = 1 << node.index();
        return (self.0 & mask) != 0
    }

    pub fn flip_bit_at_node(&mut self, node: NodeIndex){
        let mask: u128 = 1 << node.index();
        self.0 = self.0 ^ mask
    }

    pub fn is_zero(&self) -> bool {
        if self.0 == 0 {
            return true
        }
        false
    }

    pub fn lowest_one(&self) -> Option<NodeIndex> {
        if self.is_zero() == true {
            None
        } else {
            Some(NodeIndex::new(self.0.trailing_zeros() as usize))
        }
    }
}

impl Sub for BitBoard {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        BitBoard(
            (self.0 | !other.0) + 1
        )
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(
            self.0 & rhs.0
        )
    }
}

impl BitOr for BitBoard {
    type Output = Self;
   
    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoard(
            self.0 | rhs.0
        )
    }
}

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

pub struct CarryRippler {
    mask: BitBoard,
    current_subset: BitBoard,
}

impl CarryRippler {
    pub fn new(mask: BitBoard) -> CarryRippler {
        return CarryRippler {
            mask,
            current_subset: BitBoard(0)
        }
    }
}

impl Iterator for CarryRippler {
    type Item = BitBoard;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_subset == self.mask {
            return None
        }
        self.current_subset = (self.current_subset - self.mask) & self.mask;
        Some(self.current_subset)
    }
}

#[derive(Debug)]
pub struct BitBoardMoves {
    source_node: NodeIndex,
    piece_type: PieceType,
    remaining_moves: BitBoard,
    next_ep_data: Option<EnPassantData>,
    promotable_nodes: Option<Vec<NodeIndex>>,
    current_promotion_values: Vec<PieceType>
}

impl BitBoardMoves {
    pub fn new(source_node: NodeIndex, piece_type: PieceType, remaining_moves: BitBoard, next_ep_data: Option<EnPassantData>, promotable_nodes: Option<Vec<NodeIndex>>) -> BitBoardMoves {
        BitBoardMoves { source_node, piece_type, remaining_moves, next_ep_data, promotable_nodes, current_promotion_values: vec![] }
    }
}

impl Iterator for BitBoardMoves {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(to_node) = self.remaining_moves.lowest_one() {
            let mut promotion = None;
            let mut en_passant_tile = None;
            if self.piece_type == PieceType::Pawn {
                match &self.next_ep_data {
                    Some(data) if data.piece_tile == to_node => {
                        en_passant_tile = Some(data.capturable_tile)
                    },
                    _ => {}
                }

                match &self.promotable_nodes {
                    Some(nodes) if nodes.contains(&to_node) => {
                        promotion = match self.current_promotion_values.len() {
                            0 => {
                                self.current_promotion_values.push(PieceType::Knight);
                                Some(PieceType::Knight)
                            },
                            1 => {
                                self.current_promotion_values.push(PieceType::Bishop);
                                Some(PieceType::Bishop)
                            },
                            2 => {
                                self.current_promotion_values.push(PieceType::Rook);
                                Some(PieceType::Rook)
                            },
                            _ => {
                                self.current_promotion_values = vec![];
                                self.remaining_moves.flip_bit_at_node(to_node);
                                Some(PieceType::Queen)
                            }
                        };
                    },
                    _ => { self.remaining_moves.flip_bit_at_node(to_node); }
                }
            } else {
                self.remaining_moves.flip_bit_at_node(to_node);
            }
            Some(Move::new(self.source_node, to_node, promotion, en_passant_tile))
        } else {
            None
        }
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

    #[test]
    fn test_get_bit_at_node() {
        assert_eq!(
            BitBoard(33554433).get_bit_at_node(NodeIndex::new(25)),
            true
        )
    }

    #[test]
    fn test_flip_bit_at_node() {
        let mut bitboard = BitBoard::empty();
        bitboard.flip_bit_at_node(NodeIndex::new(0));
        assert_eq!(
            bitboard,
            BitBoard::new(1)
        )
    }

    #[test]
    fn test_is_zero() {
        assert_eq!(
            BitBoard::empty().is_zero(),
            true
        );
        assert_eq!(
            BitBoard::new(1).is_zero(),
            false
        )
    }

    #[test]
    fn test_lowest_one() {
        let bitboard = BitBoard::new(24);
        assert_eq!(
            bitboard.lowest_one(),
            Some(NodeIndex::new(3))
        );
        assert_eq!(
            BitBoard::empty().lowest_one(),
            None
        )
    }

    #[test]
    fn test_bitboard_not() {
        assert_eq!(
            !BitBoard::empty(),
            BitBoard(340282366920938463463374607431768211455) // 2 ** 128 - 1
        )
    }

    #[test]
    fn test_carry_ripple() {
        let mut test = CarryRippler::new(BitBoard(3));
        assert_eq!(
            test.next().unwrap(),
            BitBoard(1)
        );
        assert_eq!(
            test.next().unwrap(),
            BitBoard(2)
        );
        assert_eq!(
            test.next().unwrap(),
            BitBoard(3)
        );
        assert_eq!(
            test.next(),
            None
        )
    }

    #[test]
    fn test_bitboard_moves_knight() {
        let source_node = NodeIndex::new(0);
        let piece_type = PieceType::Knight;
        let remaining_moves = BitBoard::from_ints(vec![10, 17]);
        let mut bitboard_moves = BitBoardMoves::new(
            source_node, piece_type, remaining_moves, None, None
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(10), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(17), None, None)
        );
        assert_eq!(
            bitboard_moves.next(),
            None
        );
    }

    #[test]
    fn test_bitboard_moves_rook() {
        let source_node = NodeIndex::new(63);
        let piece_type = PieceType::Rook;
        let remaining_moves = BitBoard::from_ints(vec![60, 61, 62]);
        let mut bitboard_moves = BitBoardMoves::new(
            source_node, piece_type, remaining_moves, None, None
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(60), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(61), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(62), None, None)
        );
        assert_eq!(
            bitboard_moves.next(),
            None
        );
    }

    #[test]
    fn test_bitboard_moves_pawn_no_promotion() {
        let source_node = NodeIndex::new(8);
        let piece_type = PieceType::Pawn;
        let remaining_moves = BitBoard::from_ints(vec![16, 17, 24]);
        let en_passant_data = Some(EnPassantData { 
            capturable_tile: NodeIndex::new(16),
            piece_tile: NodeIndex::new(24) 
        });
        let mut bitboard_moves = BitBoardMoves::new(
            source_node, piece_type, remaining_moves, en_passant_data, None
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(16), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(17), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(24), None, Some(NodeIndex::new(16)))
        );
        assert_eq!(
            bitboard_moves.next(),
            None
        );
    }

    #[test]
    fn test_bitboard_moves_pawn_with_promotion() {
        let source_node = NodeIndex::new(48);
        let piece_type = PieceType::Pawn;
        let remaining_moves = BitBoard::from_ints(vec![56, 57]);
        let mut bitboard_moves = BitBoardMoves::new(
            source_node, piece_type, remaining_moves, None, Some(vec![
                NodeIndex::new(56),
                NodeIndex::new(57)
            ])
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(56), Some(PieceType::Knight), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(56), Some(PieceType::Bishop), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(56), Some(PieceType::Rook), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(56), Some(PieceType::Queen), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, NodeIndex::new(57), Some(PieceType::Knight), None)
        );
    }
}
