use std::io;

use crate::{bit_board::{BitBoard, BitBoardTiles}, chess_move::Move, engine::Engine, graph_boards::{graph_board::{Tile, TileIndex}, traditional_board::TraditionalBoardGraph}, piece_set::{Color, Piece}, position::{GameOver, Position}};



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

    pub fn play_game(&mut self) {
        let mut turn_count = 0;
        while self.current_position.is_over(&self.engine.move_tables) == None {
            let active_player = self.current_position.active_player.as_idx();
            clearscreen::clear().expect("failed to clear screen");
            println!("{}", self.board.display(&self.current_position, None, &self.engine.move_tables, true));
            println!("Turn {}, {} to move", turn_count, self.current_position.active_player);
            if self.current_position.is_in_check(&self.engine.move_tables, &self.current_position.active_player) {
                println!("You are in check!")
            }

            let selected_move = match self.are_players_cpu[active_player] {
                true => self.engine.search_for_move(&mut self.current_position),
                false => self.get_human_move()
            };

            match self.current_position.is_playable_move(&selected_move, &self.engine.move_tables) {
                true => {
                    self.current_position.make_legal_move(&selected_move);
                    turn_count += 1
                },
                false => continue
            }
        }
        match self.current_position.is_over(&self.engine.move_tables).unwrap() {
            GameOver::Draw => println!("Game ended in a draw!"),
            GameOver::Checkmate => println!("{} wins by checkmate!", self.current_position.active_player.opponent())
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
        
        let selected_white = white_pieces.get_piece_at(*tile_index); // TODO: These should all be references in functions!
        let selected_black = black_pieces.get_piece_at(*tile_index);
        let selected_piece = selected_white.or(selected_black);
        
        let (selected_color, allied_occupied, enemy_occupied, pawn_tables) = match black_pieces.get_piece_at(*tile_index) {
            Some(_t) => (Color::Black, black_pieces.occupied, white_pieces.occupied, &self.engine.move_tables.black_pawn_tables),
            _ => (Color::White, white_pieces.occupied, black_pieces.occupied, &self.engine.move_tables.white_pawn_tables)
        };

        let mut pseudo_moves =  match selected_piece {
            Some(Piece::Pawn) => {
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
            let mut promotion: Option<Piece> = None;
            if pawn_tables.promotion_board.get_bit_at_tile(destination_tile) && selected_piece == Some(Piece::Pawn) {
                promotion = Some(Piece::Queen);
            }
            // TODO: Redesign and use BitBoardMoves
            let chess_move = Move::new(*tile_index, destination_tile, promotion, None);
            if !self.current_position.is_playable_move(&chess_move, &self.engine.move_tables) {
                pseudo_moves.flip_bit_at_tile_index(destination_tile);
            }
        }

        return pseudo_moves
    }

    pub fn attempt_move_input(&mut self, source_tile: TileIndex, destination_tile: TileIndex) -> Result<(), TileParseError> {
        let chess_move = self.parse_move_input(source_tile, destination_tile)?;
        match self.current_position.is_playable_move(&chess_move, &self.engine.move_tables) {
            true => {
                self.current_position.make_legal_move(&chess_move);
                return Ok(())
            },
            false => return Err(TileParseError::InvalidMoveError)
        }
    }

    // TODO: Rename equivalent things to source_tile and destination_tile
    fn parse_move_input(&self, source_tile: TileIndex, destination_tile: TileIndex) -> Result<Move, TileParseError> {
        // Assumes destination is valid due to limiting the selectable tiles
        let active_pieces = &self.current_position.pieces[self.current_position.active_player.as_idx()];

        let en_passant_data = match active_pieces.get_piece_at(source_tile) {
            Some(Piece::Pawn) => {
                self.engine.move_tables.white_pawn_tables.en_passant_table[source_tile.index()].clone().or(
                    self.engine.move_tables.black_pawn_tables.en_passant_table[source_tile.index()].clone()
                )
            },
            None => return Err(TileParseError::InvalidMoveError), // Source could be enemy pieces
            _ => None
        };

        let en_passant_data = match en_passant_data {
            Some(epd) => {
                if epd.occupied_tile != destination_tile {
                    None
                } else {
                    Some(epd)
                }
            },
            None => None
        };

        let mut promotion = None;
        if active_pieces.get_piece_at(source_tile) == Some(Piece::Pawn) {
            let promotion_board = match self.current_position.active_player.as_idx() {
                0 => self.engine.move_tables.white_pawn_tables.promotion_board,
                _ => self.engine.move_tables.black_pawn_tables.promotion_board
            };
            if promotion_board.get_bit_at_tile(destination_tile) {
                promotion = Some(Piece::Queen)
            }
        }
        
        return Ok(Move::from_input(
            source_tile,
            destination_tile,
            promotion,
            en_passant_data
        ))
    }

    fn get_human_move(&mut self) -> Move {
        let mut selected_tile = None;
        let mut to_tile = None;
        let mut promotion = None;
        while to_tile == None {
            clearscreen::clear().expect("failed to clear screen");
            println!("{}", self.board.display(&self.current_position, selected_tile, &self.engine.move_tables, true));
            println!("{} to move", self.current_position.active_player);
            if self.current_position.is_in_check(&self.engine.move_tables, &self.current_position.active_player) {
                println!("You ({}) are in check!", self.current_position.active_player)
            }

            let mut player_input = String::new();
            io::stdin().read_line(&mut player_input)
                .expect("Failed to read line");

            let player_input: Vec<&str> = player_input.trim().split(", ").collect();

            if player_input.len() == 0 {
                continue
            }
            // Arg1 = tile index for selected piece, or from_tile
            if player_input.len() >= 1 {
                match self.parse_tile(player_input[0]) {
                    Ok(t) => selected_tile = Some(t),
                    Err(_) => continue
                }
            }
            // Arg2 = tile index for to_tile
            if player_input.len() >= 2 {
                match self.parse_tile(player_input[1]) {
                    Ok(t) => to_tile = Some(t),
                    Err(_) => continue
                }
            }
            // Arg3 = Optional promotion value
            if player_input.len() >= 3 {
                match parse_promotion(player_input[2]) {
                    Ok(p) => promotion = Some(p),
                    Err(_) => continue
                }
            }
        }

        let selected_idx = selected_tile.unwrap().index();

        let en_passant_data = match self.current_position.pieces[self.current_position.active_player.as_idx()].get_piece_at(selected_tile.unwrap()) {
            Some(Piece::Pawn) => {
                self.engine.move_tables.white_pawn_tables.en_passant_table[selected_idx].clone().or(
                    self.engine.move_tables.black_pawn_tables.en_passant_table[selected_idx].clone()
                )
            },
            _ => None
        };

        let en_passant_data = match en_passant_data {
            Some(epd) => {
                if epd.occupied_tile != to_tile.unwrap() {
                    None
                } else {
                    Some(epd)
                }
            },
            None => None
        };

        Move::from_input(
            selected_tile.unwrap(),
            to_tile.unwrap(),
            promotion,
            en_passant_data
        )
    }

    fn parse_tile(&self, arg: &str) -> Result<TileIndex, TileParseError> {
        let num = arg.parse();
        match num {
            Ok(n) => {
                if n > self.board.0.node_count() {
                    Err(TileParseError::TileOutOfRangeError)
                } else {
                    Ok(TileIndex::new(n))
                }
            },
            Err(_) => Err(TileParseError::ParseIntError)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TileParseError {
    ParseIntError,
    TileOutOfRangeError,
    InvalidMoveError
}

#[derive(Debug, Clone, PartialEq)]
pub enum PieceParseError {
    InvalidPieceChar,
}

fn parse_promotion(arg: &str) -> Result<Piece, PieceParseError> {
    match arg.to_lowercase().chars().nth(0) {
        Some('q') => Ok(Piece::Queen),
        Some('r') => Ok(Piece::Rook),
        Some('b') => Ok(Piece::Bishop),
        Some('n') => Ok(Piece::Knight),
        _ => Err(PieceParseError::InvalidPieceChar) // Pawns and Kings are not valid promotion targets
    }
}
