use std::sync::Arc;
use lazy_static::lazy_static;

use crate::bit_board::{BitBoard, BitBoardTiles};
use crate::graph_boards::graph_board::{TileIndex};
use crate::chess_move::{EnPassantData, Move};
use crate::move_generator::MoveTables;
use crate::piece_set::{Color, Piece, PieceType, PieceSet};
use crate::zobrist::ZobristTable;

lazy_static! {
    static ref ZOBRIST_TABLE: ZobristTable = ZobristTable::generate();
}

// static ZOBRIST_TABLE: ZobristTable = ZobristTable::generate();

#[derive(Debug, PartialEq)]
pub enum GameOver {
    Checkmate,
    Draw
}

impl GameOver {
    pub fn display(&self, winning_player: Color) -> String {
        match self {
            GameOver::Checkmate => format!("{} wins!", winning_player),
            GameOver::Draw => format!("Draw!")
        }
    }
}

#[derive(Debug)]
pub struct PositionRecord {
    pub en_passant_data: Option<EnPassantData>,
    pub captured_piece: Option<PieceType>,
    pub previous_record: Option<Arc<PositionRecord>>,
    pub zobrist: u64,
    pub fifty_move_counter: u32,
}

impl PositionRecord {
    pub fn default(initial_zobrist: u64) -> PositionRecord {
        PositionRecord {
            en_passant_data: None,
            captured_piece: None,
            previous_record: None,
            zobrist: initial_zobrist,
            fifty_move_counter: 0,
        }
    }

    pub fn from_string(fen: String) -> PositionRecord {
        let tile_indices: Vec<&str> = fen.split(",").collect();
        let en_passant_data = Some(EnPassantData {
            source_tile: TileIndex::new(tile_indices[0].parse().unwrap()),
            passed_tile: TileIndex::new(tile_indices[1].parse().unwrap()),
            occupied_tile: TileIndex::new(tile_indices[2].parse().unwrap())
        });
        PositionRecord { en_passant_data, captured_piece: None, previous_record: None, zobrist: 0, fifty_move_counter: 0 }
    }
   
    pub fn get_previous_record(&self) -> Option<Arc<PositionRecord>> {
        self.previous_record.as_ref().cloned()
    }
}


#[derive(Debug)]
pub struct Position {
    pub active_player: Color,
    pub pieces: [PieceSet; 2],
    pub record: Arc<PositionRecord>
    // pub board_type
    // pub properties
}

impl Position {
    pub fn get_occupant(&self, tile_index: &TileIndex) -> Option<Piece> {
        if let Some(piece) = self.pieces[0].get_piece_at(tile_index) {
            return Some(Piece { piece, color: Color::White })
        } else if let Some(piece) = self.pieces[1].get_piece_at(tile_index) {
            return Some(Piece { piece, color: Color::Black })
        } else {
            return None
        }
    }

    pub fn get_zobrist(&self) -> u64 {
        let mut output = 0;
        for tile_index in 0..128 {
            if let Some(occupant) = self.get_occupant(&TileIndex::new(tile_index)) {
                let piece_idx = occupant.piece.as_idx();
                output ^= ZOBRIST_TABLE.pieces[occupant.color.as_idx()][piece_idx][tile_index]
            }
        }
        if let Some(en_passant_data) = &self.record.en_passant_data {
            output ^= ZOBRIST_TABLE.en_passant[en_passant_data.passed_tile.index()]
        }
        if self.active_player == Color::Black {
            output ^= ZOBRIST_TABLE.black_to_move
        }
        return output
    }

    pub fn from_string(fen: String) -> Self {
        // fen format: <piece_info> <active_player> <passed_tile_index,occupied_tile_index>
        let mut zobrist_hash = 0;
        let components: Vec<&str> = fen.split(" ").collect();
        let mut pieces = [
            PieceSet::empty(),
            PieceSet::empty()
        ];
        let mut tile_counter = 0;
        let mut skip_tiles = "".to_string();

        for symbol in components[0].chars() {
            match symbol.is_numeric() {
                true => {
                    skip_tiles.push(symbol);
                },
                false => {
                    if skip_tiles.len() > 0 {
                        tile_counter += skip_tiles.parse::<usize>().unwrap();
                        skip_tiles = "".to_string();
                    }
                    let tile_index = TileIndex::new(tile_counter);
                    let color = match symbol == symbol.to_ascii_lowercase() {
                        false => Color::White,
                        true => Color::Black
                    };
                    pieces[color.as_idx()].piece_boards[PieceType::from_char(symbol).as_idx()]
                        .flip_bit_at_tile_index(tile_index);
                    tile_counter += 1;
                }
            }
        }
        pieces[0].update_occupied();
        pieces[1].update_occupied();
        let active_player = match components[1] {
            "w" => Color::White,
            _ => Color::Black
        };
        let record = match components[2] {
            "-" => PositionRecord::default(),
            _ => PositionRecord::from_string(components[2].to_string())
        };
        Self { active_player, pieces, record: record.into() }
    }

    pub fn to_string(&self) -> String {
        let mut output = "".to_string();
        let mut empty_tile_counter = 0;
        for tile in 0..128 {
            let tile_index = TileIndex::new(tile);
            if let Some(piece) = self.pieces[0].get_piece_at(&tile_index) {
                let symbol = match piece {
                    PieceType::King => 'K',
                    PieceType::Queen => 'Q',
                    PieceType::Rook => 'R',
                    PieceType::Bishop => 'B',
                    PieceType::Knight => 'N',
                    PieceType::Pawn => 'P',
                };
                if empty_tile_counter > 0 {
                    output.push_str(&empty_tile_counter.to_string());
                    empty_tile_counter = 0;
                }
                output.push(symbol);
            } else if let Some(piece) = self.pieces[1].get_piece_at(&tile_index) {
                let symbol = match piece {
                    PieceType::King => 'k',
                    PieceType::Queen => 'q',
                    PieceType::Rook => 'r',
                    PieceType::Bishop => 'b',
                    PieceType::Knight => 'n',
                    PieceType::Pawn => 'p',
                };
                if empty_tile_counter > 0 {
                    output.push_str(&empty_tile_counter.to_string());
                    empty_tile_counter = 0;
                }
                output.push(symbol);
            } else {
                empty_tile_counter += 1;
            }
        }
        output.push(' ');
        match self.active_player {
            Color::White => output.push('w'),
            Color::Black => output.push('b'),
        }
        output.push(' ');
        if let Some(data) = &self.record.en_passant_data {
            output.push_str(&data.passed_tile.index().to_string());
            output.push(',');
            output.push_str(&data.occupied_tile.index().to_string());
        } else {
            output.push('-')
        }
        output
    }

    pub fn new_traditional() -> Self {
        return Position::from_string("RNBQKBNRPPPPPPPP32pppppppprnbqkbnr w -".to_string())
    }

    pub fn new_hexagonal() -> Self {
        return Position::from_string("BKNRP1QB2P2N1B1P3R3P4PPPPP21ppppp4p3r3p1b1n2p2bq1prnkb w -".to_string())
    }

    pub fn new_triangular() -> Self {
        return Position::from_string("RKNP2pnkrQBP3pbqNP4pnP5p21 w -".to_string())
    }

    pub fn is_in_check(&self, move_tables: &MoveTables, color: &Color) -> bool {
        let opponent_idx = color.opponent().as_idx();
        let king_tile = self.pieces[color.as_idx()].piece_boards[PieceType::King.as_idx()].lowest_one().unwrap();
       
        let enemy_occupants = self.pieces[opponent_idx].occupied;
        let all_occupants = enemy_occupants | self.pieces[color.as_idx()].occupied;

        // Orthogonals
        for rev_direction_table in move_tables.reverse_slide_tables.iter().step_by(2) {
            let candidates = rev_direction_table[king_tile] & (
                self.pieces[opponent_idx].piece_boards[PieceType::Rook.as_idx()] | self.pieces[opponent_idx].piece_boards[PieceType::Queen.as_idx()]
            );
            for candidate in BitBoardTiles::new(candidates) {
                if move_tables.slide_tables.query(&candidate, &all_occupants, true, false).get_bit_at_tile(&king_tile) {
                    return true
                }
            }
        }
       
        // Diagonals
        for rev_direction_table in move_tables.reverse_slide_tables.iter().skip(1).step_by(2) {
            let candidates = rev_direction_table[king_tile] & (
                self.pieces[opponent_idx].piece_boards[PieceType::Bishop.as_idx()] | self.pieces[opponent_idx].piece_boards[PieceType::Queen.as_idx()]
            );
            for candidate in BitBoardTiles::new(candidates) {
                if move_tables.slide_tables.query(&candidate, &all_occupants, false, true).get_bit_at_tile(&king_tile) {
                    return true
                }
            }
        }
       
        // Knights
        if !(move_tables.reverse_knight_table[king_tile] & self.pieces[opponent_idx].piece_boards[PieceType::Knight.as_idx()]).is_zero() {
            return true
        }

        // Pawns
        let pawn_threats = match color {
            Color::White => &move_tables.reverse_black_pawn_table,
            Color::Black => &move_tables.reverse_white_pawn_table
        };
        if !(pawn_threats[king_tile] & self.pieces[opponent_idx].piece_boards[PieceType::Pawn.as_idx()]).is_zero() {
            return true
        };

        false // Don't need to check for King-to-King threats
    }

    pub fn is_checkmate(&mut self, move_tables: &MoveTables) -> bool {
        self.is_in_check(move_tables, &self.active_player) && !move_tables.has_legal_moves( self)
    }

    pub fn is_stalemate(&mut self, move_tables: &MoveTables) -> bool {
        !self.is_in_check(move_tables, &self.active_player) && !move_tables.has_legal_moves(self)
    }

    pub fn fifty_move_draw(&self) -> bool {
        self.record.fifty_move_counter >= 50
    }

    pub fn is_legal_move(&mut self, chess_move: &Move, move_tables: &MoveTables) -> bool {
        // Could check other parameters:
        // Kings cannot be captured, allies cannot be captured
        // Could check the validity of the move wrt the move tables
        let moving_player = self.active_player.clone();
        self.make_legal_move(chess_move);
        let legality = !self.is_in_check(move_tables, &moving_player);
        self.unmake_legal_move(chess_move);
        return legality
    }
   
    pub fn is_playable_move(&mut self, chess_move: &Move, move_tables: &MoveTables) -> bool {
        let player_idx = self.active_player.as_idx();
        let opponent_idx = self.active_player.opponent().as_idx();
        let selected_piece = self.pieces[player_idx].get_piece_at(&chess_move.source_tile);
        
        let movement_options = match selected_piece {
            None => return false, // The moving player must have a piece at source_tile
            Some(PieceType::Pawn) => move_tables.query_pawn(
                &self.active_player,
                chess_move.source_tile,
                &self.pieces[opponent_idx].occupied,
                self.get_occupied(),
                &self.record.en_passant_data
            ),
            _ => move_tables.query_piece(&selected_piece.unwrap(), chess_move.source_tile, self.get_occupied())
        };

        if movement_options.get_bit_at_tile(&chess_move.destination_tile) == false {
            return false // The selected piece must be able to move to to_tile
        }
        if self.is_legal_move(chess_move, move_tables) == false {
            return false // The selected move must be legal
        }
        let promotion_board = match player_idx {
            0 => move_tables.white_pawn_tables.promotion_board,
            _ => move_tables.black_pawn_tables.promotion_board
        };

        if promotion_board.get_bit_at_tile(&chess_move.destination_tile) && self.pieces[player_idx].get_piece_at(&chess_move.source_tile) == Some(PieceType::Pawn) && chess_move.promotion == None {
            return false // Promotion must be provided if a pawn is moving to a promotion tile
        }
        return true
    }

    fn get_occupied(&self) -> BitBoard {
        return self.pieces[0].occupied | self.pieces[1].occupied
    }

    pub fn make_legal_move(&mut self, legal_move: &Move) {
        // Assumes the move is legal?
        let player_idx = self.active_player.as_idx();
        let opponent_idx = self.active_player.opponent().as_idx();

        let mut new_zobrist = self.record.zobrist;

        let source_tile = legal_move.source_tile;
        let destination_tile = legal_move.destination_tile;

        let mut fifty_move_counter = self.record.fifty_move_counter + 1;

        let moving_piece = self.pieces[player_idx].get_piece_at(&source_tile).unwrap();
        new_zobrist ^= ZOBRIST_TABLE.pieces[player_idx][moving_piece.as_idx()][source_tile.index()];
        new_zobrist ^= ZOBRIST_TABLE.pieces[player_idx][moving_piece.as_idx()][destination_tile.index()];
        self.pieces[player_idx].move_piece(source_tile, destination_tile);

        let mut target_piece = self.pieces[opponent_idx].get_piece_at(&destination_tile);
        if let Some(captured_piece) = target_piece {
            fifty_move_counter = 0;
            new_zobrist ^= ZOBRIST_TABLE.pieces[opponent_idx][captured_piece.as_idx()][destination_tile.index()];
            self.pieces[opponent_idx].capture_piece(destination_tile)
        };

        if let Some(promotion_target) =  &legal_move.promotion {
            new_zobrist ^= ZOBRIST_TABLE.pieces[player_idx][PieceType::Pawn.as_idx()][destination_tile.index()];
            new_zobrist ^= ZOBRIST_TABLE.pieces[player_idx][promotion_target.as_idx()][destination_tile.index()];
            self.pieces[player_idx].promote_piece(destination_tile, promotion_target)
        }

        if moving_piece == PieceType::Pawn {
            fifty_move_counter = 0;
            if let Some(en_passant_data) = &self.record.en_passant_data {
                if destination_tile == en_passant_data.passed_tile {
                    target_piece = Some(PieceType::Pawn);
                    self.pieces[opponent_idx].capture_piece(en_passant_data.occupied_tile)
                }
            }
        }

        if let Some(prev_en_passant_data) = &self.record.en_passant_data {
            new_zobrist ^= ZOBRIST_TABLE.en_passant[prev_en_passant_data.source_tile.index()]
        } // TODO: Redesign en passant data entirely

        if legal_move.en_passant_data != None {
            new_zobrist ^= ZOBRIST_TABLE.en_passant[source_tile.index()];
        }

        self.record = PositionRecord {
            en_passant_data: legal_move.en_passant_data.clone(),
            captured_piece: target_piece,
            previous_record: Some(self.record.clone()),
            zobrist: new_zobrist,
            fifty_move_counter: fifty_move_counter
        }.into();

        self.pieces[player_idx].update_occupied();
        self.pieces[opponent_idx].update_occupied();
        self.active_player = self.active_player.opponent();
    }

    pub fn unmake_legal_move(&mut self, legal_move: &Move) {
        // Assumes the move was legal
        self.active_player = self.active_player.opponent();
        let player_idx = self.active_player.as_idx();
        let opponent_idx = self.active_player.opponent().as_idx();
       
        let source_tile = legal_move.source_tile;
        let destination_tile = legal_move.destination_tile;
       
        self.pieces[player_idx].move_piece(destination_tile, source_tile);

        let captured_piece = self.record.captured_piece.to_owned();
        if let Some(ref piece_type) = captured_piece {
            self.pieces[opponent_idx].return_piece(destination_tile, &piece_type)
        }
        if let Some(_t) = &legal_move.promotion {
            self.pieces[player_idx].demote_piece(source_tile)
        }
        if let Some(prev_record) = self.record.get_previous_record() {
            self.record = prev_record
        } else {
            self.record = PositionRecord::default().into();
        }
        if captured_piece == Some(PieceType::Pawn) {
            if let Some(en_passant_data) = &self.record.en_passant_data {
                if destination_tile == en_passant_data.passed_tile {
                    self.pieces[opponent_idx].capture_piece(destination_tile);
                    self.pieces[opponent_idx].return_piece(en_passant_data.occupied_tile, &PieceType::Pawn)
                }
            }
        }
        self.pieces[player_idx].update_occupied();
        self.pieces[opponent_idx].update_occupied();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::bit_board::BitBoard;
    use crate::graph_boards::traditional_board::TraditionalBoardGraph;

    #[test]
    fn test_new_traditional_occupied() {
        let position = Position::new_traditional();
        let occupied = position.pieces[0].occupied | position.pieces[1].occupied;
        assert_eq!(
            occupied,
            BitBoard::from_ints(vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63
            ])
        )
    }

    fn test_move_tables() -> MoveTables {
        let board = TraditionalBoardGraph::new();
        board.0.move_tables()
    }

    #[test]
    fn test_new_hexagonal_occupied() {
        let position: Position = Position::new_hexagonal();
        let occupied = position.pieces[0].occupied | position.pieces[1].occupied;
        assert_eq!(
            occupied,
            BitBoard::from_ints(vec![
                0, 1, 2, 3, 4, 6, 7, 10, 13, 15, 17, 21, 25, 30, 31, 32, 33, 34,
                56, 57, 58, 59, 60, 65, 69, 73, 75, 77, 80, 83, 84, 86, 87, 88, 89, 90
            ])
        )
    }

    #[test]
    fn test_make_legal_move() {
        let mut position = Position::new_traditional();
        let source_tile = TileIndex::new(1);
        let destination_tile = TileIndex::new(18);
        let legal_move = Move::new(source_tile, destination_tile, None, None);
        position.make_legal_move(&legal_move);
        assert_eq!(
            position.pieces[0].piece_boards[PieceType::Knight.as_idx()],
            BitBoard::from_ints(vec![6, 18])
        );
    }

    #[test]
    fn test_en_passant_move() {
        let mut position = Position::new_traditional();
        let destination_tile = TileIndex::new(24);
        let legal_move = Move::new(
            TileIndex::new(8),
            destination_tile,
            None,
            Some(TileIndex::new(16))
        );
        position.make_legal_move(&legal_move);
        assert_eq!(
            *position.record.en_passant_data.as_ref().unwrap(),
            EnPassantData::new(TileIndex::new(8), TileIndex::new(16), destination_tile)
        )
    }

    #[test]
    fn test_en_passant_capture() {
        let mut position = Position::new_traditional();
        let en_passant_tile = TileIndex::new(16);
        let captured_tile = TileIndex::new(24);
        let first_move = Move::new(
            TileIndex::new(8),
            captured_tile,
            None,
            Some(en_passant_tile)
        );
        position.make_legal_move(&first_move);
        let capturing_move = Move::new(
            TileIndex::new(48),
            en_passant_tile,
            None,
            None
        );
        position.make_legal_move(&capturing_move);
        assert_eq!(
            position.pieces[0].piece_boards[PieceType::Pawn.as_idx()].get_bit_at_tile(&TileIndex::new(24)),
            false
        );
        assert_eq!(
            position.pieces[1].piece_boards[PieceType::Pawn.as_idx()].get_bit_at_tile(&TileIndex::new(16)),
            true
        )
    }

    #[test]
    fn test_sequential_moves() {
        let mut position = Position::new_traditional();
        let first_move = Move::new(
            TileIndex::new(12),
            TileIndex::new(28),
            None,
            Some(TileIndex::new(20))
        );
        let second_move = Move::new(
            TileIndex::new(51),
            TileIndex::new(35),
            None,
            Some(TileIndex::new(43))
        );
        let third_move = Move::new(
            TileIndex::new(28),
            TileIndex::new(35),
            None,
            None
        );
        position.make_legal_move(&first_move);
        position.make_legal_move(&second_move);
        assert_eq!(
            *position.record.en_passant_data.as_ref().unwrap(),
            EnPassantData { source_tile: TileIndex::new(51), passed_tile: TileIndex::new(43), occupied_tile: TileIndex::new(35) }
        );
        position.make_legal_move(&third_move);
        assert_eq!(
            position.pieces[0].occupied,
            BitBoard::new(2_u128.pow(16) - 1 - 2_u128.pow(12) + 2_u128.pow(35))
        );
        assert_eq!(
            position.pieces[1].occupied,
            BitBoard::new(2_u128.pow(64) - 2_u128.pow(48) - 2_u128.pow(51))
        )
    }

    #[test]
    fn test_unmake_legal_move() {
        let mut position = Position::from_string("RNBQKBNRPPPPPPP16P16pppppppprnbqkbnr w 15,23,31".to_string());
        
        let source_tile = TileIndex::new(1);
        let destination_tile = TileIndex::new(18);
        let legal_move = Move::new(source_tile, destination_tile, None, None);
        position.make_legal_move(&legal_move);
        position.unmake_legal_move(&legal_move);
        assert_eq!(
            position.pieces[0].piece_boards[PieceType::Knight.as_idx()],
            BitBoard::from_ints(vec![1, 6])
        );
        assert_eq!(
            position.record.en_passant_data,
            Some(EnPassantData { source_tile: TileIndex::new(15), passed_tile: TileIndex::new(23), occupied_tile: TileIndex::new(31) })
        );

        let source_tile = TileIndex::new(8);
        let destination_tile = TileIndex::new(16);
        let demotion_move = Move::new(source_tile, destination_tile, Some(PieceType::Knight), None);
        position.make_legal_move(&demotion_move);
        position.unmake_legal_move(&demotion_move);
        assert_eq!(
            position.pieces[0].piece_boards[PieceType::Knight.as_idx()],
            BitBoard::from_ints(vec![1, 6])
        );
        assert_eq!(
            position.pieces[0].piece_boards[PieceType::Pawn.as_idx()],
            BitBoard::from_ints(vec![8, 9, 10, 11, 12, 13, 14, 31])
        );

        let source_tile = TileIndex::new(0);
        let destination_tile = TileIndex::new(56);
        let capture_move = Move::new(source_tile, destination_tile, None, None);
        position.make_legal_move(&capture_move);
        assert_eq!(
            position.record.captured_piece,
            Some(PieceType::Rook)
        );
        position.unmake_legal_move(&capture_move);
        assert_eq!(
            position.pieces[0].piece_boards[PieceType::Rook.as_idx()],
            BitBoard::from_ints(vec![0, 7])
        );
        assert_eq!(
            position.pieces[1].piece_boards[PieceType::Rook.as_idx()],
            BitBoard::from_ints(vec![56, 63])
        );
    }

    #[test]
    fn test_string_conversion() {
        let position = Position::new_traditional();
        assert_eq!(
            position.to_string(),
            "RNBQKBNRPPPPPPPP32pppppppprnbqkbnr w -".to_string()
        )
    }

    #[test]
    fn test_is_in_check() {
        let mut position = Position::new_traditional();
        let move_tables = test_move_tables();
        assert_eq!(
            position.is_in_check(&move_tables, &Color::White),
            false
        ); // Initial position, not in check for white
        assert_eq!(
            position.is_in_check(&move_tables, &Color::Black),
            false
        ); // Initial position, not in check for black
        position.make_legal_move(&Move::new(
            TileIndex::new(1),
            TileIndex::new(43),
            None, None
        ));
        assert_eq!(
            position.is_in_check(&move_tables, &Color::Black),
            true
        ); // Black in check by Knight
        position.make_legal_move(&Move::new(
            TileIndex::new(59),
            TileIndex::new(20),
            None, None
        ));
        assert_eq!(
            position.is_in_check(&move_tables, &Color::White),
            false
        ); // White not in check by blocked orthogonal queen
        position.make_legal_move(&Move::new(
            TileIndex::new(12),
            TileIndex::new(28),
            None, None
        ));
        assert_eq!(
            position.is_in_check(&move_tables, &Color::White),
            true
        ); // White in check by unblocked orthogonal queen
        position.make_legal_move(&Move::new(
            TileIndex::new(20),
            TileIndex::new(18),
            None, None
        ));
        assert_eq!(
            position.is_in_check(&move_tables, &Color::White),
            false
        ); // White not in check by blocked diagonal queen
        position.make_legal_move(&Move::new(
            TileIndex::new(11),
            TileIndex::new(19),
            None, None
        ));
        assert_eq!(
            position.is_in_check(&move_tables, &Color::White),
            true
        ); // White in check by unblocked diagonal queen
    }

    #[test]
    fn test_zobrist_unmade_moves() {
        // Testing that prev_record stores the zobrist hash correctly
        let mut position = Position::new_traditional();
        let move_tables = TraditionalBoardGraph::new().0.move_tables();
        let init_hash = position.get_zobrist();
        for move_1 in move_tables.get_legal_moves(&mut position) {
            position.make_legal_move(&move_1);
            for move_2 in move_tables.get_legal_moves(&mut position) {
                position.make_legal_move(&move_2);
                for move_3 in move_tables.get_legal_moves(&mut position) {
                    position.make_legal_move(&move_3);
                    position.unmake_legal_move(&move_3);
                }
                position.unmake_legal_move(&move_2);
            }
            position.unmake_legal_move(&move_1);
        };
        let first_move = Move::new(
            TileIndex::new(8),
            TileIndex::new(16),
            None, None
        );
        position.make_legal_move(&first_move);
        assert_eq!(init_hash, position.record.zobrist)
    }
        
    #[test]
    fn test_zobrist_repeat_position() {
        let mut position = Position::new_traditional();
        let init_hash = position.get_zobrist();

        let move_1 = Move::new(
            TileIndex::new(1),
            TileIndex::new(18),
            None, None
        );
        let move_2 = Move::new(
            TileIndex::new(62),
            TileIndex::new(53),
            None, None
        );
        let move_3 = Move::new(
            TileIndex::new(18),
            TileIndex::new(1),
            None, None
        );
        let move_4 = Move::new(
            TileIndex::new(53),
            TileIndex::new(62),
            None, None
        );
        position.make_legal_move(&move_1);
        position.make_legal_move(&move_2);
        position.make_legal_move(&move_3);
        position.make_legal_move(&move_4);
        assert_eq!(init_hash, position.get_zobrist())
    }
}
