
use crate::chess_move::Move;


const TABLE_SIZE: usize = 1_000_000;

#[derive(Debug, Clone)]
pub enum Flag {
    Exact,
    UpperBound,
    LowerBound
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub zobrist_key: u64,
    pub score: i32,
    pub depth: u8,
    pub flag: Flag,
    pub best_move: Option<Move>
}

pub struct TranspositionTable {
    entries: Vec<Option<Entry>>
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable { entries: vec![None; TABLE_SIZE] }
    }

    pub fn get_index(&self, zobrist_key: u64) -> usize {
        (zobrist_key % TABLE_SIZE as u64) as usize
    }

    pub fn retrieve(&self, zobrist_key: u64, depth: u8, alpha: i32, beta: i32) -> Option<i32> {
        let index = self.get_index(zobrist_key);
        if let Some(entry) = &self.entries[index] {
            if entry.zobrist_key == zobrist_key {
                if entry.depth >= depth {
                    match entry.flag {
                        Flag::Exact => return Some(entry.score),
                        Flag::LowerBound => if entry.score >= beta {
                            return Some(entry.score);
                        }
                        Flag::UpperBound => if entry.score <= alpha {
                            return Some(entry.score);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn store(&mut self, zobrist_key: u64, score: i32, depth: u8, flag: Flag, best_move: Option<Move>) {
        let index = self.get_index(zobrist_key);
        let new_entry = Entry { zobrist_key, score, depth, flag, best_move };
        if let Some(existing) = &self.entries[index] {
            if existing.zobrist_key == zobrist_key || depth >= existing.depth {
                self.entries[index] = Some(new_entry);
            }
        } else {
            self.entries[index] = Some(new_entry)
        }
    }
}


mod tests {
    use crate::{chess_move::Move, graph_boards::graph_board::TileIndex, transposition_table::{Flag, TranspositionTable}};

    fn test_table() -> TranspositionTable {
        let mut table = TranspositionTable::new();
        table.store(
            1,
            100,
            8,
            Flag::Exact,
            Some(Move::new(TileIndex::new(0), TileIndex::new(1), None, None))
        );
        table.store(
            2,
            200,
            8,
            Flag::LowerBound,
            Some(Move::new(TileIndex::new(0), TileIndex::new(1), None, None))
        );
        table.store(
            3,
            50,
            8,
            Flag::UpperBound,
            Some(Move::new(TileIndex::new(0), TileIndex::new(1), None, None))
        );
        return table
    }

    #[test]
    fn test_match_and_retrieval() {
        let table = test_table();
        assert_eq!(table.retrieve(1, 8, 50, 150), Some(100))
    }

    #[test]
    fn test_key_mismatch() {
        let table = test_table();
        assert_eq!(table.retrieve(1000001, 8, 50, 150), None)
    }

    #[test]
    fn test_insufficient_depth() {
        let table = test_table();
        assert_eq!(table.retrieve(1, 9, 50, 150), None)
    }

    #[test]
    fn test_beta_cutoff() {
        let table = test_table();
        assert_eq!(table.retrieve(2, 8, 50, 150), Some(200))
    }

    #[test]
    fn test_alpha_cutoff() {
        let table = test_table();
        assert_eq!(table.retrieve(3, 8, 70, 150), Some(50))
    }

    #[test]
    fn test_depth_replacement() {
        let mut table = test_table();
        table.store(
            1000001,
            300,
            9,
            Flag::Exact,
            Some(Move::new(TileIndex::new(1), TileIndex::new(2), None, None))
        );
        assert_eq!(table.retrieve(1, 8, 50, 150), None);
        assert_eq!(table.retrieve(1000001, 9, 50, 150), Some(300))
    }
}
