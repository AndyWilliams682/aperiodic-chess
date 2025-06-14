use graph_board::{HexagonalBoardGraph, TraditionalBoardGraph};
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
    let position = Position::new_traditional();

    let mut game = Game {
        engine: Engine::new(move_tables),
        are_players_cpu: vec![false, true],
        current_position: position,
        board
    };

    game.play_game();
}
