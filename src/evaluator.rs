use crate::constants::NUM_PIECE_TYPES;
use crate::movement_tables::{JumpTable, PawnTables, SlideTables};
use crate::bit_board::{BitBoard, BitBoardTiles};
use crate::piece_set::{Color, PieceSet, PieceType};
use crate::move_generator::MoveTables;
use crate::position::Position;


// All measured in centipawns
const PIECE_SCORES: [isize; 6] = [
    9999, // King
    900,  // Queen
    500,  // Rook
    350,  // Bishop
    350,  // Knight
    100   // Pawn
];
const CHECKMATED_SCORE: isize = -99999;
const POSITIONAL_MULTIPLIER: isize = 5;

// Primitive evaluator will use # of possible moves from each square on an empty board
pub struct MobilityTable(Vec<u32>);

impl MobilityTable {
    fn from_jumps(table: &JumpTable) -> Self {
        let mut output: Vec<u32> = vec![];
        for bitboard in &table.0 {
            output.push(bitboard.0.count_ones())
        }
        Self(output)
    }

    fn from_slides(table: &SlideTables, piece_type: PieceType) -> Self {
        let initial_direction = match piece_type == PieceType::Bishop {
            true => 1,
            false => 0
        };
        let direction_step = match piece_type == PieceType::Queen {
            true => 1,
            false => 2
        };
        let mut output: Vec<u32> = vec![0; 128];
        for direction in (initial_direction..table.0.len()).step_by(direction_step) {
            let mut tile_idx = 0;
            for tile in &table[direction].0 {
                output[tile_idx] += tile.get(&BitBoard::empty()).unwrap().0.count_ones();
                tile_idx += 1;
            }
        }
        Self(output)
    }

    fn from_pawn(table: &PawnTables) -> Self {
        Self::from_jumps(&table.single_table)
    }
}

pub struct Evaluator {
    king: MobilityTable,
    queen: MobilityTable,
    rook: MobilityTable,
    bishop: MobilityTable,
    knight: MobilityTable,
    white_pawn: MobilityTable,
    black_pawn: MobilityTable
}

impl Evaluator {
    pub fn new(move_tables: &MoveTables) -> Self {
        Self {
            king: MobilityTable::from_jumps(&move_tables.king_table),
            queen: MobilityTable::from_slides(&move_tables.slide_tables, PieceType::Queen),
            rook: MobilityTable::from_slides(&move_tables.slide_tables, PieceType::Rook),
            bishop: MobilityTable::from_slides(&move_tables.slide_tables, PieceType::Bishop),
            knight: MobilityTable::from_jumps(&move_tables.knight_table),
            white_pawn: MobilityTable::from_pawn(&move_tables.white_pawn_tables),
            black_pawn: MobilityTable::from_pawn(&move_tables.black_pawn_tables)
        }
    }
   
    fn pieceset_material_score(&self, piece_set: &PieceSet) -> isize {
        let mut material_score = 0;
        for piece_idx in 0..NUM_PIECE_TYPES {
            material_score += piece_set.piece_boards[piece_idx].0.count_ones() as isize * PIECE_SCORES[piece_idx]
        }
        material_score
    }
   
    fn piece_positional_score(&self, piece_board: BitBoard, piece_type: PieceType, color: &Color) -> isize {
        let mobility_table = match piece_type {
            PieceType::King => &self.king,
            PieceType::Queen => &self.queen,
            PieceType::Rook => &self.rook,
            PieceType::Bishop => &self.bishop,
            PieceType::Knight => &self.knight,
            PieceType::Pawn => match color {
                Color::White => &self.white_pawn,
                Color::Black => &self.black_pawn
            },
        };
        let mut score = 0;
        for tile_idx in BitBoardTiles::new(piece_board) {
            score += mobility_table.0[tile_idx.index()]
        }
        score as isize * POSITIONAL_MULTIPLIER
    }
   
    fn pieceset_positional_score(&self, piece_set: &PieceSet, is_endgame: bool, color: &Color) -> isize {
        let mut score = 0;
        let king_multi = match is_endgame {
            true => 1,
            false => -1
        };
        for piece_idx in 0..NUM_PIECE_TYPES {
            let mut piece_positional_score = self.piece_positional_score(
                piece_set.piece_boards[piece_idx],
                PieceType::from_idx(piece_idx),
                color
            );
            if PieceType::from_idx(piece_idx) == PieceType::King {
                piece_positional_score *= king_multi
            }
            score += piece_positional_score
        }
        score
    }
   
    fn evaluate(&self, position: Position) -> isize {
        let mut score = 0;
        let player_idx = position.active_player.as_idx();
        let player_pieceset = &position.pieces[player_idx];
        let opponent_idx = position.active_player.opponent().as_idx();
        let opponent_pieceset = &position.pieces[opponent_idx];
        let mut total_material_score = 0;
       
        let player_material = self.pieceset_material_score(player_pieceset);
        score += player_material;
        total_material_score += player_material;
       
        let opponent_material = self.pieceset_material_score(opponent_pieceset);
        score -= opponent_material;
        total_material_score += opponent_material;
       
        let is_endgame = total_material_score < 2 * PIECE_SCORES[PieceType::King.as_idx()]
                                                    + 2 * PIECE_SCORES[PieceType::Queen.as_idx()]
                                                    + 2 * PIECE_SCORES[PieceType::Rook.as_idx()];
       
        score += self.pieceset_positional_score(player_pieceset, is_endgame, &position.active_player);
        score -= self.pieceset_positional_score(opponent_pieceset, is_endgame, &position.active_player.opponent());
        score
    }
}
