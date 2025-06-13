use std::io;

use crate::{chess_move::Move, engine::Engine, graph_board::{TileIndex, TraditionalBoardGraph}, piece_set::Piece, position::Position};



pub struct Game {
    pub engine: Engine,
    pub are_players_cpu: Vec<bool>,
    pub current_position: Position,
    pub board: TraditionalBoardGraph
}

impl Game {
    pub fn play_game(&mut self) {
        let mut turn_count = 0;
        let active_player = self.current_position.active_player.as_idx();
        while self.current_position.is_winner() == None {
            clearscreen::clear().expect("failed to clear screen");
            println!("{}", self.board.display(&self.current_position, None, &self.engine.move_tables, true));
            println!("Turn {}, {} to move", turn_count, self.current_position.active_player);

            let selected_move = match self.are_players_cpu[active_player] {
                true => Move::new(TileIndex::new(0), TileIndex::new(0), None, None), // TODO: Add Engine call here
                false => self.get_human_move() // Request move info here
            };

            println!("{:?}", selected_move);

            match self.current_position.is_playable_move(&selected_move, &self.engine.move_tables) {
                true => {
                    self.current_position.make_legal_move(&selected_move);
                    turn_count += 1
                },
                false => continue
            }
        }
    }

    fn get_human_move(&mut self) -> Move {
        let mut selected_tile = None;
        let mut to_tile = None;
        let mut promotion = None;
        while to_tile == None {
            // clearscreen::clear().expect("failed to clear screen");
            println!("{}", self.board.display(&self.current_position, selected_tile, &self.engine.move_tables, true));
            println!("{} to move", self.current_position.active_player);

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

        let mut en_passant_data = match self.current_position.pieces[self.current_position.active_player.as_idx()].get_piece_at(selected_tile.unwrap()) {
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
    TileOutOfRangeError
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
