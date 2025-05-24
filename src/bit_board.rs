use std::collections::HashSet;
use std::ops::{Sub, BitAnd, BitOr, Not};

use crate::piece_set::PieceType;
use crate::chess_move::{EnPassantData, Move};
use crate::graph_board::TileIndex;


#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct BitBoard(u128);

impl BitBoard {
    pub fn from_node_indices(node_indices: HashSet<TileIndex>) -> BitBoard {
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

    pub fn get_bit_at_node(self, node: TileIndex) -> bool {
        let mask: u128 = 1 << node.index();
        return (self.0 & mask) != 0
    }

    pub fn flip_bit_at_node(&mut self, node: TileIndex){
        let mask: u128 = 1 << node.index();
        self.0 = self.0 ^ mask
    }

    pub fn is_zero(&self) -> bool {
        if self.0 == 0 {
            return true
        }
        false
    }

    pub fn lowest_one(&self) -> Option<TileIndex> {
        if self.is_zero() == true {
            None
        } else {
            Some(TileIndex::new(self.0.trailing_zeros() as usize))
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
pub struct BitBoardNodes {
    remaining_nodes: BitBoard
}

impl BitBoardNodes {
    pub fn new(remaining_nodes: BitBoard) -> Self {
        Self { remaining_nodes }
    }
}

impl Iterator for BitBoardNodes {
    type Item = TileIndex;
   
    fn next(&mut self) -> Option<Self::Item> {
        let next_node = self.remaining_nodes.lowest_one();
        match next_node {
            Some(node) => self.remaining_nodes.flip_bit_at_node(node),
            _ => {}
        }
        next_node
    }
}

#[derive(Debug)]
pub struct BitBoardMoves {
    source_node: TileIndex,
    is_pawn: bool,
    remaining_moves: BitBoardNodes,
    next_ep_data: Option<EnPassantData>,
    promotable_nodes: Option<Vec<TileIndex>>,
    current_promotion_node: Option<TileIndex>,
    current_promotion_counter: u32
}

impl BitBoardMoves {
    pub fn new(source_node: TileIndex, is_pawn: bool, remaining_move_board: BitBoard, next_ep_data: Option<EnPassantData>, promotable_nodes: Option<Vec<TileIndex>>) -> BitBoardMoves {
        BitBoardMoves {
            source_node,
            is_pawn,
            remaining_moves: BitBoardNodes::new(remaining_move_board),
            next_ep_data,
            promotable_nodes,
            current_promotion_node: None,
            current_promotion_counter: 0
        }
    }
}

impl Iterator for BitBoardMoves {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let mut promotion = None;
        let mut en_passant_tile = None;
       
        // Need to iterate through the possible promotions if possible
        if let Some(to_node) = self.current_promotion_node {
            self.current_promotion_counter += 1;
            let promotion = match self.current_promotion_counter {
                1 => Some(PieceType::Bishop), // 0 will already be handled for the Knight
                2 => Some(PieceType::Rook),
                _ => { // Reset after Queen
                    self.current_promotion_node.take();
                    self.current_promotion_counter = 0;
                    Some(PieceType::Queen)
                }
            };
            Some(Move::new(self.source_node, to_node, promotion, en_passant_tile))
        } else if let Some(to_node) = self.remaining_moves.next() {
            if self.is_pawn {
                match &self.next_ep_data {
                    Some(data) if data.piece_tile == to_node => {
                        en_passant_tile = Some(data.capturable_tile)
                    },
                    _ => {}
                }

                match &self.promotable_nodes { // Handles promotion to Knight
                    Some(nodes) if nodes.contains(&to_node) => {
                        self.current_promotion_node = Some(to_node);
                        promotion = Some(PieceType::Knight);
                    },
                    _ => {}
                }
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
            BitBoard::from_node_indices(HashSet::from_iter([TileIndex::new(0), TileIndex::new(25)])),
            BitBoard(33554433)
        )
    }

    #[test]
    fn test_get_bit_at_node() {
        assert_eq!(
            BitBoard(33554433).get_bit_at_node(TileIndex::new(25)),
            true
        )
    }

    #[test]
    fn test_flip_bit_at_node() {
        let mut bitboard = BitBoard::empty();
        bitboard.flip_bit_at_node(TileIndex::new(0));
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
            Some(TileIndex::new(3))
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
    fn test_bitboard_nodes() {
        let bitboard = BitBoard::from_ints(vec![1, 3, 4]);
        let mut bitboard_nodes = BitBoardNodes::new(bitboard);
        assert_eq!(
            bitboard_nodes.next().unwrap(),
            TileIndex::new(1)
        );
        assert_eq!(
            bitboard_nodes.next().unwrap(),
            TileIndex::new(3)
        );
        assert_eq!(
            bitboard_nodes.next().unwrap(),
            TileIndex::new(4)
        );
        assert_eq!(
            bitboard_nodes.next(),
            None
        );
    }

    #[test]
    fn test_bitboard_nodes_empty() {
        assert_eq!(
            BitBoardNodes::new(BitBoard::empty()).next(),
            None
        )
    }

    #[test]
    fn test_bitboard_moves_knight() {
        let source_node = TileIndex::new(0);
        let remaining_moves = BitBoard::from_ints(vec![10, 17]);
        let mut bitboard_moves = BitBoardMoves::new(
            source_node, false, remaining_moves, None, None
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(10), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(17), None, None)
        );
        assert_eq!(
            bitboard_moves.next(),
            None
        );
    }

    #[test]
    fn test_bitboard_moves_rook() {
        let source_node = TileIndex::new(63);
        let remaining_moves = BitBoard::from_ints(vec![60, 61, 62]);
        let mut bitboard_moves = BitBoardMoves::new(
            source_node, false, remaining_moves, None, None
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(60), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(61), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(62), None, None)
        );
        assert_eq!(
            bitboard_moves.next(),
            None
        );
    }

    #[test]
    fn test_bitboard_moves_pawn_no_promotion() {
        let source_node = TileIndex::new(8);
        let remaining_moves = BitBoard::from_ints(vec![16, 17, 24]);
        let en_passant_data = Some(EnPassantData { 
            capturable_tile: TileIndex::new(16),
            piece_tile: TileIndex::new(24) 
        });
        let mut bitboard_moves = BitBoardMoves::new(
            source_node, true, remaining_moves, en_passant_data, None
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(16), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(17), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(24), None, Some(TileIndex::new(16)))
        );
        assert_eq!(
            bitboard_moves.next(),
            None
        );
    }

    #[test]
    fn test_bitboard_moves_pawn_with_promotion() {
        let source_node = TileIndex::new(48);
        let remaining_moves = BitBoard::from_ints(vec![56, 57]);
        let mut bitboard_moves = BitBoardMoves::new(
            source_node, true, remaining_moves, None, Some(vec![
                TileIndex::new(56),
                TileIndex::new(57)
            ])
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(56), Some(PieceType::Knight), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(56), Some(PieceType::Bishop), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(56), Some(PieceType::Rook), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(56), Some(PieceType::Queen), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_node, TileIndex::new(57), Some(PieceType::Knight), None)
        );
    }
}
