use std::io;

use chess_move::Move;
use graph_board::{HexagonalBoardGraph, TraditionalBoardGraph, TileIndex};
use position::Position;

use crate::{engine::Engine, game::Game};

mod graph_board;
mod bit_board;
mod limited_int;
mod position;
mod chess_move;
mod move_generator;
mod piece_set;
mod movement_tables;
mod evaluator;
mod game;
mod engine;

fn main() {
    let board = TraditionalBoardGraph::new();
    let move_tables = board.0.move_tables();
    let mut position = Position::new_traditional();

    let mut game = Game {
        engine: Engine {
            move_tables
        },
        are_players_cpu: vec![false, false],
        current_position: position,
        board
    };

    game.play_game();
    // position.make_legal_move(&Move::new(TileIndex::new(8), TileIndex::new(17), None, None));
    // println!("Square: {:?}", move_tables.perft(&mut position, 5));
    // for _move_count in 0..5 {
    //     clearscreen::clear().expect("failed to clear screen");
    //     println!("{}", board.display(&position, None, &move_tables, true));
        
    //     let mut move_input = String::new();
    //     io::stdin().read_line(&mut move_input)
    //         .expect("Failed to read line");

    //     let move_input: Vec<&str> = move_input.trim().split(", ").collect();
    //     let from_tile = TileIndex::new(move_input[0].parse().unwrap());
    //     let to_tile = TileIndex::new(move_input[1].parse().unwrap());
    //     let new_move = Move::new(from_tile, to_tile, None, None);
    //     position.make_legal_move(&new_move);
    // }

    // let board = HexagonalBoardGraph::new();
    // let move_tables = board.0.move_tables();
    // let mut position = Position::new_hexagonal();
    // println!("Hex: {:?}", move_tables.perft(&mut position, 5));
}
