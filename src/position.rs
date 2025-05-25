use std::sync::Arc;

use crate::graph_board::{TileIndex};
use crate::chess_move::{EnPassantData, Move};
use crate::piece_set::{Color, PieceType, PieceSet};

#[derive(Debug)]
pub struct PositionRecord {
    pub en_passant_data: Option<EnPassantData>,
    pub captured_piece: Option<PieceType>,
    pub previous_record: Option<Arc<PositionRecord>>,
    // previous_zobrist_key??
}

impl PositionRecord {
    pub fn default() -> PositionRecord {
        PositionRecord {
            en_passant_data: None,
            captured_piece: None,
            previous_record: None
        }
    }

    pub fn from_string(fen: String) -> PositionRecord {
        let tile_indices: Vec<&str> = fen.split(",").collect();
        let en_passant_data = Some(EnPassantData {
            capturable_tile: TileIndex::new(tile_indices[0].parse().unwrap()),
            piece_tile: TileIndex::new(tile_indices[1].parse().unwrap())
        });
        PositionRecord { en_passant_data, captured_piece: None, previous_record: None }
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
    pub fn from_string(fen: String) -> Self {
        // fen format: <piece_info> <active_player> <EP capturable_tile_index,piece_tile_index>
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
                    match symbol {
                        'K' => pieces[0].king.flip_bit_at_tile_index(tile_index),
                        'Q' => pieces[0].queen.flip_bit_at_tile_index(tile_index),
                        'R' => pieces[0].rook.flip_bit_at_tile_index(tile_index),
                        'B' => pieces[0].bishop.flip_bit_at_tile_index(tile_index),
                        'N' => pieces[0].knight.flip_bit_at_tile_index(tile_index),
                        'P' => pieces[0].pawn.flip_bit_at_tile_index(tile_index),
                        'k' => pieces[1].king.flip_bit_at_tile_index(tile_index),
                        'q' => pieces[1].queen.flip_bit_at_tile_index(tile_index),
                        'r' => pieces[1].rook.flip_bit_at_tile_index(tile_index),
                        'b' => pieces[1].bishop.flip_bit_at_tile_index(tile_index),
                        'n' => pieces[1].knight.flip_bit_at_tile_index(tile_index),
                        'p' => {
                            pieces[1].pawn.flip_bit_at_tile_index(tile_index)
                        },
                        _ => {}
                    };
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

    pub fn new_traditional() -> Self {
        return Position::from_string("RNBQKBNRPPPPPPPP32pppppppprnbqkbnr w -".to_string())
    }

    pub fn new_hexagonal() -> Self {
        return Position::from_string("BKNRP1QB2P2N1B1P3R3P4PPPPP21ppppp4p3r3p1b1n2p2bq1prnkb w -".to_string())
    }

    pub fn make_legal_move(&mut self, legal_move: &Move) {
        // Assumes the move is legal?
        let player_idx = self.active_player.as_idx();
        let opponent_idx = self.active_player.opponent().as_idx();

        let from_tile = legal_move.from_tile;
        let to_tile = legal_move.to_tile;

        let moving_piece = self.pieces[player_idx].get_piece_at(from_tile).unwrap();
        self.pieces[player_idx].move_piece(from_tile, to_tile);

        let mut target_piece = self.pieces[opponent_idx].get_piece_at(to_tile);
        match target_piece {
            Some(ref _t) => self.pieces[opponent_idx].capture_piece(to_tile),
            None => {}
        }

        match &legal_move.promotion {
            Some(promotion_target) => self.pieces[player_idx].promote_piece(to_tile, promotion_target),
            None => {}
        }

        if moving_piece == PieceType::Pawn {
            match &self.record.en_passant_data {
                Some(en_passant_data) if to_tile == en_passant_data.capturable_tile => {
                    target_piece = Some(PieceType::Pawn);
                    self.pieces[opponent_idx].capture_piece(en_passant_data.piece_tile)
                },
                _ => {}
            }
        }

        self.record = PositionRecord {
            en_passant_data: legal_move.en_passant_data.clone(), // TODO: Candidate 1
            captured_piece: target_piece,
            previous_record: Some(self.record.clone()) // TODO: Candidate 2
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
       
        let from_tile = legal_move.from_tile;
        let to_tile = legal_move.to_tile;
       
        self.pieces[player_idx].move_piece(to_tile, from_tile);

        let captured_piece = self.record.captured_piece.to_owned();
        if let Some(ref piece_type) = captured_piece {
            self.pieces[opponent_idx].return_piece(to_tile, &piece_type)
        }
        match &legal_move.promotion {
            Some(_t) => self.pieces[player_idx].demote_piece(from_tile),
            None => {} // TODO: Use better syntax for cases like this, if Some(_t) = legal_move.promotion {}
        }
        if let Some(prev_record) = self.record.get_previous_record() {
            self.record = prev_record
        } else {
            self.record = PositionRecord::default().into();
        }
        if captured_piece == Some(PieceType::Pawn) {
            if let Some(en_passant_data) = &self.record.en_passant_data {
                if to_tile == en_passant_data.capturable_tile {
                    self.pieces[opponent_idx].capture_piece(to_tile);
                    self.pieces[opponent_idx].return_piece(en_passant_data.piece_tile, &PieceType::Pawn)
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
        let from_tile = TileIndex::new(1);
        let to_tile = TileIndex::new(18);
        let legal_move = Move::new(from_tile, to_tile, None, None);
        position.make_legal_move(&legal_move);
        assert_eq!(
            position.pieces[0].knight,
            BitBoard::from_ints(vec![6, 18])
        );
    }

    #[test]
    fn test_en_passant_move() {
        let mut position = Position::new_traditional();
        let to_tile = TileIndex::new(24);
        let legal_move = Move::new(
            TileIndex::new(8),
            to_tile,
            None,
            Some(TileIndex::new(16))
        );
        position.make_legal_move(&legal_move);
        assert_eq!(
            *position.record.en_passant_data.as_ref().unwrap(),
            EnPassantData::new(TileIndex::new(16), to_tile)
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
            position.pieces[0].pawn.get_bit_at_tile(TileIndex::new(24)),
            false
        );
        assert_eq!(
            position.pieces[1].pawn.get_bit_at_tile(TileIndex::new(16)),
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
            EnPassantData { capturable_tile: TileIndex::new(43), piece_tile: TileIndex::new(35) }
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
        let mut position = Position::from_string("RNBQKBNRPPPPPPP16P16pppppppprnbqkbnr w 23,31".to_string());
        
        let from_tile = TileIndex::new(1);
        let to_tile = TileIndex::new(18);
        let legal_move = Move::new(from_tile, to_tile, None, None);
        position.make_legal_move(&legal_move);
        position.unmake_legal_move(&legal_move);
        assert_eq!(
            position.pieces[0].knight,
            BitBoard::from_ints(vec![1, 6])
        );
        assert_eq!(
            position.record.en_passant_data,
            Some(EnPassantData { capturable_tile: TileIndex::new(23), piece_tile: TileIndex::new(31) })
        );

        let from_tile = TileIndex::new(8);
        let to_tile = TileIndex::new(16);
        let demotion_move = Move::new(from_tile, to_tile, Some(PieceType::Knight), None);
        position.make_legal_move(&demotion_move);
        position.unmake_legal_move(&demotion_move);
        assert_eq!(
            position.pieces[0].knight,
            BitBoard::from_ints(vec![1, 6])
        );
        assert_eq!(
            position.pieces[0].pawn,
            BitBoard::from_ints(vec![8, 9, 10, 11, 12, 13, 14, 31])
        );

        let from_tile = TileIndex::new(0);
        let to_tile = TileIndex::new(56);
        let capture_move = Move::new(from_tile, to_tile, None, None);
        position.make_legal_move(&capture_move);
        assert_eq!(
            position.record.captured_piece,
            Some(PieceType::Rook)
        );
        position.unmake_legal_move(&capture_move);
        assert_eq!(
            position.pieces[0].rook,
            BitBoard::from_ints(vec![0, 7])
        );
        assert_eq!(
            position.pieces[1].rook,
            BitBoard::from_ints(vec![56, 63])
        );
    }
}
