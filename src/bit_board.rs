use std::collections::HashSet;
use std::ops::{Sub, BitAnd, BitOr, Not, BitAndAssign, BitOrAssign};

use crate::piece_set::PieceType;
use crate::chess_move::{EnPassantData, Move};
use crate::graph_boards::graph_board::TileIndex;


#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct BitBoard(pub u128);

impl BitBoard {
    pub fn from_tile_indices(tile_indices: HashSet<TileIndex>) -> BitBoard {
        let mut result: u128 = 0;
        for tile in tile_indices {
            result += 1 << tile.index();
        }
        return BitBoard(result)
    }

    pub fn from_ints(ints: Vec<u128>) -> BitBoard {
        let mut result: u128 = 0;
        for tile in ints {
            result += 1 << tile;
        }
        return BitBoard(result)
    }

    pub fn new(n: u128) -> BitBoard {
        return BitBoard(n)
    }

    pub fn empty() -> BitBoard {
        return BitBoard(0)
    }

    pub fn get_bit_at_tile(self, tile: &TileIndex) -> bool {
        let mask: u128 = 1 << tile.index();
        return (self.0 & mask) != 0
    }

    pub fn flip_bit_at_tile_index(&mut self, tile: TileIndex){
        let mask: u128 = 1 << tile.index();
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

impl BitAndAssign<BitBoard> for BitBoard {
    fn bitand_assign(&mut self, rhs: BitBoard) {
        self.0 &= rhs.0
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

impl BitOrAssign<BitBoard> for BitBoard {
    fn bitor_assign(&mut self, rhs: BitBoard) {
        self.0 |= rhs.0
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
pub struct BitBoardTiles {
    remaining_tiles: BitBoard
}

impl BitBoardTiles {
    pub fn new(remaining_tiles: BitBoard) -> Self {
        Self { remaining_tiles }
    }
}

impl Iterator for BitBoardTiles {
    type Item = TileIndex;
   
    fn next(&mut self) -> Option<Self::Item> {
        let next_tile = self.remaining_tiles.lowest_one();
        if let Some(tile) = next_tile {
            self.remaining_tiles.flip_bit_at_tile_index(tile)
        }
        next_tile
    }
}

#[derive(Debug)]
pub struct BitBoardMoves {
    source_tile: TileIndex,
    is_pawn: bool,
    remaining_moves: BitBoardTiles,
    next_ep_data: Option<EnPassantData>,
    promotable_tiles: BitBoard,
    current_promotion_tile: Option<TileIndex>,
    current_promotion_counter: u32
}

impl BitBoardMoves {
    pub fn new(source_tile: TileIndex, is_pawn: bool, remaining_move_board: BitBoard, next_ep_data: Option<EnPassantData>, promotable_tiles: BitBoard) -> BitBoardMoves {
        BitBoardMoves {
            source_tile,
            is_pawn,
            remaining_moves: BitBoardTiles::new(remaining_move_board),
            next_ep_data,
            promotable_tiles,
            current_promotion_tile: None,
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
        if let Some(destination_tile) = self.current_promotion_tile {
            self.current_promotion_counter += 1;
            let promotion = match self.current_promotion_counter {
                1 => Some(PieceType::Bishop), // 0 will already be handled for the Knight
                2 => Some(PieceType::Rook),
                _ => { // Reset after Queen
                    self.current_promotion_tile.take();
                    self.current_promotion_counter = 0;
                    Some(PieceType::Queen)
                }
            };
            Some(Move::new(self.source_tile, destination_tile, promotion, en_passant_tile))
        } else if let Some(destination_tile) = self.remaining_moves.next() {
            if self.is_pawn {
                if let Some(data) = &self.next_ep_data {
                    if data.occupied_tile == destination_tile {
                        en_passant_tile = Some(data.passed_tile)
                    }
                }
                if self.promotable_tiles.get_bit_at_tile(&destination_tile) { // Handles promotion to Knight
                    self.current_promotion_tile = Some(destination_tile);
                    promotion = Some(PieceType::Knight);
                }
            }
            Some(Move::new(self.source_tile, destination_tile, promotion, en_passant_tile))
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
            BitBoard::from_tile_indices(HashSet::from_iter([TileIndex::new(0), TileIndex::new(25)])),
            BitBoard(33554433)
        )
    }

    #[test]
    fn test_get_bit_at_tile() {
        assert_eq!(
            BitBoard(33554433).get_bit_at_tile(&TileIndex::new(25)),
            true
        )
    }

    #[test]
    fn test_flip_bit_at_tile() {
        let mut bitboard = BitBoard::empty();
        bitboard.flip_bit_at_tile_index(TileIndex::new(0));
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
    fn test_bitboard_tiles() {
        let bitboard = BitBoard::from_ints(vec![1, 3, 4]);
        let mut bitboard_tiles = BitBoardTiles::new(bitboard);
        assert_eq!(
            bitboard_tiles.next().unwrap(),
            TileIndex::new(1)
        );
        assert_eq!(
            bitboard_tiles.next().unwrap(),
            TileIndex::new(3)
        );
        assert_eq!(
            bitboard_tiles.next().unwrap(),
            TileIndex::new(4)
        );
        assert_eq!(
            bitboard_tiles.next(),
            None
        );
    }

    #[test]
    fn test_bitboard_tiles_empty() {
        assert_eq!(
            BitBoardTiles::new(BitBoard::empty()).next(),
            None
        )
    }

    #[test]
    fn test_bitboard_moves_knight() {
        let source_tile = TileIndex::new(0);
        let remaining_moves = BitBoard::from_ints(vec![10, 17]);
        let mut bitboard_moves = BitBoardMoves::new(
            source_tile, false, remaining_moves, None, BitBoard::empty()
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(10), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(17), None, None)
        );
        assert_eq!(
            bitboard_moves.next(),
            None
        );
    }

    #[test]
    fn test_bitboard_moves_rook() {
        let source_tile = TileIndex::new(63);
        let remaining_moves = BitBoard::from_ints(vec![60, 61, 62]);
        let mut bitboard_moves = BitBoardMoves::new(
            source_tile, false, remaining_moves, None, BitBoard::empty()
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(60), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(61), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(62), None, None)
        );
        assert_eq!(
            bitboard_moves.next(),
            None
        );
    }

    #[test]
    fn test_bitboard_moves_pawn_no_promotion() {
        let source_tile = TileIndex::new(8);
        let remaining_moves = BitBoard::from_ints(vec![16, 17, 24]);
        let en_passant_data = Some(EnPassantData { 
            passed_tile: TileIndex::new(16),
            occupied_tile: TileIndex::new(24) 
        });
        let mut bitboard_moves = BitBoardMoves::new(
            source_tile, true, remaining_moves, en_passant_data, BitBoard::empty()
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(16), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(17), None, None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(24), None, Some(TileIndex::new(16)))
        );
        assert_eq!(
            bitboard_moves.next(),
            None
        );
    }

    #[test]
    fn test_bitboard_moves_pawn_with_promotion() {
        let source_tile = TileIndex::new(48);
        let remaining_moves = BitBoard::from_ints(vec![56, 57]);
        let mut bitboard_moves = BitBoardMoves::new(
            source_tile, true, remaining_moves, None, BitBoard::from_ints(vec![
                56,
                57
            ])
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(56), Some(PieceType::Knight), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(56), Some(PieceType::Bishop), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(56), Some(PieceType::Rook), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(56), Some(PieceType::Queen), None)
        );
        assert_eq!(
            bitboard_moves.next().unwrap(),
            Move::new(source_tile, TileIndex::new(57), Some(PieceType::Knight), None)
        );
    }
}
