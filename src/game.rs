use crate::{bit_board::{BitBoard, BitBoardTiles}, chess_move::Move, engine::Engine, graph_boards::{graph_board::TileIndex, traditional_board::TraditionalBoardGraph}, piece_set::{Color, PieceType}, position::{GameOver, Position}};



pub struct Game {
    pub engine: Engine,
    pub are_players_cpu: Vec<bool>,
    pub current_position: Position,
    pub board: TraditionalBoardGraph
}

impl Game {
    pub fn is_over(&mut self) -> Option<GameOver> {
        if self.current_position.is_checkmate(&self.engine.move_tables) {
            return Some(GameOver::Checkmate)
        } else if self.current_position.is_stalemate(&self.engine.move_tables) || self.current_position.fifty_move_draw() { // TODO: Add more draw conditions here
            return Some(GameOver::Draw)
        } else {
            None
        }
    }

    pub fn make_cpu_move(&mut self) {
        let cpu_move = self.engine.search_for_move(&mut self.current_position);
        self.current_position.make_legal_move(&cpu_move);
    }

    pub fn query_tile(&mut self, tile_index: &TileIndex) -> BitBoard {
        let white_pieces = &self.current_position.pieces[0];
        let black_pieces = &self.current_position.pieces[1];
        let occupied = white_pieces.occupied | black_pieces.occupied; // TODO: Occupied stored somewhere??
        
        let selected_white = white_pieces.get_piece_at(tile_index);
        let selected_black = black_pieces.get_piece_at(tile_index);
        let selected_piece = selected_white.or(selected_black);
        
        let (selected_color, allied_occupied, enemy_occupied, pawn_tables) = match black_pieces.get_piece_at(tile_index) {
            Some(_t) => (Color::Black, black_pieces.occupied, white_pieces.occupied, &self.engine.move_tables.black_pawn_tables),
            _ => (Color::White, white_pieces.occupied, black_pieces.occupied, &self.engine.move_tables.white_pawn_tables)
        };

        let mut pseudo_moves =  match selected_piece {
            Some(PieceType::Pawn) => {
                self.engine.move_tables.query_pawn(&selected_color, *tile_index, &enemy_occupied, occupied, &self.current_position.record.en_passant_data)
            },
            None => BitBoard::empty(),
            _ => { // All non-Pawn PieceTypes
                self.engine.move_tables.query_piece(&selected_piece.unwrap(), *tile_index, occupied) & !allied_occupied
            }
        };

        // TODO: Playable move is breaking on pawn promotion
        // Need to make the move a promotion if applicable
        // If pawn, if destination_tile == a promotion tile, set promotion = Queen

        for destination_tile in BitBoardTiles::new(pseudo_moves) {
            let mut promotion: Option<PieceType> = None;
            if pawn_tables.promotion_board.get_bit_at_tile(&destination_tile) && selected_piece == Some(PieceType::Pawn) {
                promotion = Some(PieceType::Queen);
            }
            // TODO: Redesign and use BitBoardMoves
            let chess_move = Move::new(*tile_index, destination_tile, promotion, None);
            if !self.current_position.is_playable_move(&chess_move, &self.engine.move_tables) {
                pseudo_moves.flip_bit_at_tile_index(destination_tile);
            }
        }

        return pseudo_moves
    }

    pub fn attempt_move_input(&mut self, source_tile: &TileIndex, destination_tile: &TileIndex) -> Result<(), ChessError> {
        let chess_move = self.parse_move_input(source_tile, destination_tile)?;
        match self.current_position.is_playable_move(&chess_move, &self.engine.move_tables) {
            true => {
                self.current_position.make_legal_move(&chess_move);
                return Ok(())
            },
            false => return Err(ChessError::InvalidMoveError)
        }
    }

    // TODO: Rename equivalent things to source_tile and destination_tile
    fn parse_move_input(&self, source_tile: &TileIndex, destination_tile: &TileIndex) -> Result<Move, ChessError> {
        // Assumes destination is valid due to limiting the selectable tiles
        let active_pieces = &self.current_position.pieces[self.current_position.active_player.as_idx()];

        let en_passant_data = match active_pieces.get_piece_at(source_tile) {
            Some(PieceType::Pawn) => {
                self.engine.move_tables.white_pawn_tables.en_passant_table[source_tile.index()].clone().or(
                    self.engine.move_tables.black_pawn_tables.en_passant_table[source_tile.index()].clone()
                )
            },
            None => return Err(ChessError::InvalidMoveError), // Source could be enemy pieces
            _ => None
        };

        let en_passant_data = match en_passant_data {
            Some(epd) => {
                if &epd.occupied_tile != destination_tile {
                    None
                } else {
                    Some(epd)
                }
            },
            None => None
        };

        let mut promotion = None;
        if active_pieces.get_piece_at(source_tile) == Some(PieceType::Pawn) {
            let promotion_board = match self.current_position.active_player.as_idx() {
                0 => self.engine.move_tables.white_pawn_tables.promotion_board,
                _ => self.engine.move_tables.black_pawn_tables.promotion_board
            };
            if promotion_board.get_bit_at_tile(destination_tile) {
                promotion = Some(PieceType::Queen)
            }
        }
        
        return Ok(Move::from_input(
            *source_tile,
            *destination_tile,
            promotion,
            en_passant_data
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChessError {
    InvalidMoveError
}
