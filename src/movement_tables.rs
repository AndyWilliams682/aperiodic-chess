use std::ops::{Index, IndexMut};
use std::collections::HashMap;

use crate::bit_board::{BitBoard, BitBoardTiles};
use crate::graph_board::TileIndex;


#[derive(Debug, PartialEq, Clone)]
pub struct JumpTable(pub Vec<BitBoard>);
// JumpTables are a list of BitBoards (one for each tile) for UNBLOCKABLE movement

impl JumpTable {
    pub fn new(val: Vec<BitBoard>) -> Self {
        Self(val)
    }

    pub fn empty(num_tiles: usize) -> Self {
        Self::new(vec![BitBoard::empty(); num_tiles])
    }

    pub fn num_tiles(&self) -> usize {
        return self.0.len()
    }

    pub fn reverse(&self) -> Self {
        let num_tiles = self.num_tiles();
        let mut output = Self::empty(num_tiles);

        let mut source_tile = 0;
        for source_tile_moves in &self.0 {
            for to_tile in BitBoardTiles::new(*source_tile_moves) {
                output[to_tile].flip_bit_at_tile_index(TileIndex::new(source_tile));
            }
            source_tile += 1;
        }
        output
    }
}

impl Index<TileIndex> for JumpTable {
    type Output = BitBoard;
   
    fn index(&self, index: TileIndex) -> &Self::Output {
        &self.0[index.index()]
    }
}

impl IndexMut<TileIndex> for JumpTable {
    fn index_mut(&mut self, index: TileIndex) -> &mut Self::Output {
        &mut self.0[index.index()]
    }
}

#[derive(Debug, Clone)]
pub struct DirectionalSlideTable(pub Vec<HashMap<BitBoard, BitBoard>>);

impl DirectionalSlideTable {
    pub fn new(val: Vec<HashMap<BitBoard, BitBoard>>) -> Self {
        return Self(val)
    }

    pub fn reverse(&self) -> JumpTable {
        // Returning a JumpTable because this does not care about blockers (will be handled later)
        let num_tiles = self.0.len();
        let mut output = JumpTable::empty(num_tiles);
       
        let mut source_tile = 0;
        for source_tile_moves in &self.0 {
            let unblocked_moves = source_tile_moves.get(&BitBoard::empty()).unwrap();
            for to_tile in BitBoardTiles::new(*unblocked_moves) {
                output[to_tile].flip_bit_at_tile_index(TileIndex::new(source_tile));
            }
            source_tile += 1;
        }
        output
    }
}

impl Index<TileIndex> for DirectionalSlideTable {
    type Output = HashMap<BitBoard, BitBoard>;
   
    fn index(&self, index: TileIndex) -> &Self::Output {
        &self.0[index.index()]
    }
}

#[derive(Debug, Clone)]
pub struct SlideTables(pub Vec<DirectionalSlideTable>);

impl SlideTables {
    pub fn new(val: Vec<DirectionalSlideTable>) -> Self {
        return Self(val)
    }
   
    pub fn query(&self, source_tile: &TileIndex, occupied: &BitBoard, orthogonals: bool, diagonals: bool) -> BitBoard {
        let mut result = BitBoard::empty();
        let initial_direction = match orthogonals {
            true => 0,
            false => 1
        };
        let direction_step = match orthogonals & diagonals {
            true => 1,
            false => 2
        };
        for direction in (initial_direction..self.0.len()).step_by(direction_step) {
            let directional_map = &self[direction][*source_tile];
            let unblocked_attacks = *directional_map.get(&BitBoard::empty()).unwrap();
            let blocked_attacks = *directional_map.get(&(*occupied & unblocked_attacks)).unwrap(); 
            result = result | blocked_attacks;
        }
        result
    }

    pub fn reverse(&self) -> Vec<JumpTable> {
        let mut output = vec![];
        for directional_table in &self.0 {
            output.push(directional_table.reverse())
        }
        output
    }
}

impl Index<usize> for SlideTables {
    type Output = DirectionalSlideTable;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[derive(Debug, Clone)]
pub struct PawnTables {
    pub single_table: JumpTable,
    pub double_table: DirectionalSlideTable,
    pub attack_table: JumpTable,
}

impl PawnTables {
    pub fn new(single_table: JumpTable, double_table: DirectionalSlideTable, attack_table: JumpTable) -> Self {
        Self { single_table, double_table, attack_table }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_board::{TraditionalBoardGraph, HexagonalBoardGraph, TraditionalDirection};
    use crate::piece_set::Color;

    fn test_traditional_board() -> TraditionalBoardGraph {
        TraditionalBoardGraph::new()
    }
    
    fn test_hexagonal_board() -> HexagonalBoardGraph {
        return HexagonalBoardGraph::new();
    }

    fn traditional_slide_tables() -> SlideTables {
        return test_traditional_board().0.all_slide_tables()
    }

    fn hexagonal_slide_tables() -> SlideTables {
        return test_hexagonal_board().0.all_slide_tables()
    }

    #[test]
    fn test_jump_table_empty_and_len() {
        let test = JumpTable::empty(64);
        assert_eq!(
            test.num_tiles(),
            64
        )
    }

    #[test]
    fn test_diagonal_slide_table() {
        let source_tile = TileIndex::new(63);
        assert_eq!(
            traditional_slide_tables().query(&source_tile, &BitBoard::empty(), false, true),
            BitBoard::from_ints(vec![54, 45, 36, 27, 18, 9, 0])
        );
        let occupied = BitBoard::from_ints(vec![45]);
        assert_eq!(
            traditional_slide_tables().query(&source_tile, &occupied, false, true),
            BitBoard::from_ints(vec![54, 45])
        );
    }

    #[test]
    fn test_orthogonal_table() {
        let source_tile = TileIndex::new(63);
        assert_eq!(
            traditional_slide_tables().query(&source_tile, &BitBoard::empty(), true, false),
            BitBoard::from_ints(vec![62, 61, 60, 59, 58, 57, 56, 55, 47, 39, 31, 23, 15, 7])
        );
        let occupied = BitBoard::from_ints(vec![62]);
        assert_eq!(
            traditional_slide_tables().query(&source_tile, &occupied, true, false),
            BitBoard::from_ints(vec![62, 55, 47, 39, 31, 23, 15, 7])
        )
    }

    #[test]
    fn test_knight_table() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(63);
        assert_eq!(
            board.0.knight_jumps_table()[source_tile],
            BitBoard::from_ints(vec![53, 46])
        )
    }

    #[test]
    fn test_slide_table_for_direction() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(0);
        assert_eq!(
            *board.0.slide_table_for_direction(&TraditionalDirection(0))[source_tile].get(&BitBoard::new(65536)).unwrap(),
            BitBoard::from_ints(vec![8, 16])
        )
    }

    #[test]
    fn test_king_table() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(63);
        assert_eq!(
            board.0.king_move_table()[source_tile],
            BitBoard::from_ints(vec![62, 55, 54])
        )
    }

    #[test]
    fn test_pawn_double_table_forward() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(8);
        assert_eq!(
            *board.0.pawn_double_table(&Color::White)[source_tile].get(&BitBoard::empty()).unwrap(),
            BitBoard::from_ints(vec![24])
        );
        assert_eq!(
            *board.0.pawn_double_table(&Color::White)[source_tile].get(&BitBoard::from_ints(vec![16])).unwrap(),
            BitBoard::empty()
        );
    }

    #[test]
    fn test_pawn_double_table_backward() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(48);
        assert_eq!(
            *board.0.pawn_double_table(&Color::Black)[source_tile].get(&BitBoard::empty()).unwrap(),
            BitBoard::from_ints(vec![32])
        );
        assert_eq!(
            *board.0.pawn_double_table(&Color::Black)[source_tile].get(&BitBoard::from_ints(vec![40])).unwrap(),
            BitBoard::empty()
        );
    }

    #[test]
    fn test_pawn_single_table() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(48);
        assert_eq!(
            board.0.pawn_single_table(&Color::White)[source_tile],
            BitBoard::from_ints(vec![56])
        );
        assert_eq!(
            board.0.pawn_single_table(&Color::White)[TileIndex::new(56)],
            BitBoard::empty()
        );
    }

    #[test]
    fn test_pawn_attack_table() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(49);
        assert_eq!(
            board.0.pawn_attack_table(&Color::White)[source_tile],
            BitBoard::from_ints(vec![56, 58])
        )
    }

    #[test]
    fn test_pawn_attack_table_at_edge() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(48);
        assert_eq!(
            board.0.pawn_attack_table(&Color::Black)[source_tile],
            BitBoard::from_ints(vec![41])
        )
    }

    #[test]
    fn test_hex_knight_table() {
        let board = test_hexagonal_board();
        let source_tile = TileIndex::new(0);
        assert_eq!(
            board.0.knight_jumps_table()[source_tile],
            BitBoard::from_ints(vec![9, 16, 22, 23])
        )
    }

    #[test]
    fn test_hex_queen_table() {
        let source_tile = TileIndex::new(0);
        assert_eq!(
            hexagonal_slide_tables().query(&source_tile, &BitBoard::empty(), true, true),
            BitBoard::from_ints(vec![
                1, 2, 3, 4, 5, // Direction 10
                6, 13, 21, 30, 40, // 2
                7, 15, 24, 34, 45, 56, 66, 75, 83, 90, // 0
                14, 32, 53, 71, 85, // 1
                8, 17, 27, 38, 50 // 11
            ])
        );
    }

    #[test]
    fn test_hex_king_table() {
        let board = test_hexagonal_board();
        let source_tile = TileIndex::new(0);
        assert_eq!(
            board.0.king_move_table()[source_tile],
            BitBoard::from_ints(vec![1, 6, 7, 8, 14])
        )
    }

    #[test]
    fn test_hex_pawn_double_table_backward() {
        let board = test_hexagonal_board();
        let source_tile = TileIndex::new(56);
        assert_eq!(
            *board.0.pawn_double_table(&Color::Black)[source_tile].get(&BitBoard::empty()).unwrap(),
            BitBoard::from_ints(vec![34])
        )
    }

    #[test]
    fn test_reverse_knight_table() {
        let board = test_traditional_board();
        let knight_table = board.0.knight_jumps_table();
        let output = knight_table.reverse();
        // For traditional/hexagonal boards, these are equal
        assert_eq!(
            output,
            knight_table
        )
    }
   
    #[test]
    fn test_reverse_pawn_table() {
        let board = test_traditional_board();
        // For traditional/hexagonal boards, rev(White)=Black
        assert_eq!(
            board.0.pawn_attack_table(&Color::White).reverse(),
            board.0.pawn_attack_table(&Color::Black)
        )
    }

    #[test]
    fn test_reverse_directional_slide_table() {
        let board = test_traditional_board();
        let directional_slide_table = board.0.slide_table_for_direction(
            &TraditionalDirection(0)
        );
        assert_eq!(
            directional_slide_table.reverse()[TileIndex::new(56)],
            BitBoard::from_ints(vec![0, 8, 16, 24, 32, 40, 48])
        );
        assert_eq!(
            directional_slide_table.reverse()[TileIndex::new(0)],
            BitBoard::empty()
        )
    }
}