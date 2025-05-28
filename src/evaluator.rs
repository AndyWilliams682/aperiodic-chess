use crate::movement_tables::{JumpTable, SlideTables};
use crate::bit_board::BitBoard;
use crate::piece_set::PieceType;


// All measured in centipawns
// TODO: May need a better way to evaluate individual pieces as the board changes
const KING_SCORE: isize = 9999;
const QUEEN_SCORE: isize = 900;
const ROOK_SCORE: isize = 500;
const BISHOP_SCORE: isize = 350;
const KNIGHT_SCORE: isize = 300;
const PAWN_SCORE: isize = 100;
const CHECKMATED_SCORE: isize = -99999;

// Primitive evaluator will use # of possible moves from each square on an empty board
pub struct ScoreTable(Vec<u32>);

impl ScoreTable {
    fn from_jump(table: JumpTable) -> Self {
        let mut output: Vec<u32> = vec![];
        for bitboard in table.0 {
            output.push(bitboard.0.count_ones())
        }
        Self(output)
    }

    fn from_slides(table: SlideTables, piece_type: PieceType) -> Self {
        let initial_direction = match piece_type == PieceType::Bishop {
            true => 1,
            false => 0
        };
        let direction_step = match piece_type == PieceType::Queen {
            true => 1,
            false => 2
        };
        let mut output: Vec<u32> = Vec::with_capacity(128);
        for direction in (initial_direction..table.0.len()).step_by(direction_step) {
            let mut tile_idx = 0;
            for tile in &table[direction].0 {
                output[tile_idx] += tile.get(&BitBoard::empty()).unwrap().0.count_ones();
                tile_idx += 1;
            }
        }
        Self(output)
    }
}