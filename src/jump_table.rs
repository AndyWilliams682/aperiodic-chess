use std::ops::{Index, IndexMut};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jump_table_empty_and_len() {
        let test = JumpTable::empty(64);
        assert_eq!(
            test.num_tiles(),
            64
        )
    }
}