use std::{char::ParseCharError, fmt::Error, io, num::ParseIntError};

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
        let active_player = 0;
        while self.current_position.is_winner() == None {
            clearscreen::clear().expect("failed to clear screen");
            println!("{}", self.board.display(&self.current_position, None, &self.engine.move_tables, true));

            let selected_move = match self.are_players_cpu[active_player] {
                true => Move::new(TileIndex::new(0), TileIndex::new(0), None, None), // TODO: Add Engine call here
                false => self.get_human_move() // Request move info here
            };

            self.current_position.make_legal_move(&selected_move)
            // TODO: Need position.is_playable_move()
            // If move is playable, increment turn counter, carry on
            // Else, retry
        }
    }

    fn get_human_move(&mut self) -> Move {
        let mut selected_tile = None;
        let mut to_tile = None;
        let mut promotion = None;
        while to_tile == None {
            clearscreen::clear().expect("failed to clear screen");
            println!("{}", self.board.display(&self.current_position, selected_tile, &self.engine.move_tables, true));

            let mut player_input = String::new();
            io::stdin().read_line(&mut player_input)
                .expect("Failed to read line");

            let player_input: Vec<&str> = player_input.trim().split(", ").collect();

            // Arg0 = tile index for selected piece, or from_tile
            if player_input.len() == 0 {
                continue
            }
            if player_input.len() >= 1 {
                match parse_tile(player_input[0]) {
                    Ok(t) => selected_tile = Some(t),
                    Err(_) => continue
                }
            }
            if player_input.len() >= 2 {
                match parse_tile(player_input[1]) {
                    Ok(t) => to_tile = Some(t),
                    Err(_) => continue
                }
            }
            if player_input.len() >= 3 {
                match parse_symbol(player_input[2]) {
                    Ok(p) => promotion = Some(p),
                    Err(_) => continue
                }
            }

            // Arg1 = tile index for to_tile
            // Arg2 = Option for promotion?
            // Ignore further arguments, and also ignore if no arguments provided

            // Make sure those all parse into the right stuff, then do position.make_playable_move() -> Result<>
            // TODO: Handle receiving non-numbers, extra punctuation, numbers too large
            // TODO: Handle asking/checking for promotion
        }
        Move::new(
            selected_tile.unwrap(),
            to_tile.unwrap(),
            promotion,
            None
        )
    }
}

fn parse_tile(arg: &str) -> Result<TileIndex, ParseIntError> {
    Ok(TileIndex::new(arg.parse()?))
}

#[derive(Debug, Clone, PartialEq)] // Derive Debug for {:?} printing, Clone if you need to copy it, PartialEq for testing
pub enum PieceParseError {
    /// Error when the first character does not map to a valid piece.
    InvalidPieceChar(), // Store the invalid character for better diagnostics
}

fn parse_symbol(arg: &str) -> Result<Piece, PieceParseError> {
    match arg.to_lowercase().chars().nth(0) {
        Some('k') => Ok(Piece::King),
        Some('q') => Ok(Piece::Queen),
        Some('r') => Ok(Piece::Rook),
        Some('b') => Ok(Piece::Bishop),
        Some('n') => Ok(Piece::Knight),
        Some('p') => Ok(Piece::Pawn),
        _ => Err(PieceParseError::InvalidPieceChar())
    }
}
