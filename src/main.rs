use graph_board::{TraditionalBoardGraph, Color};
use move_generator::MoveTables;
use position::Position;

mod graph_board;
mod bit_board;
mod limited_int;
mod position;
mod chess_move;
mod move_generator;
mod piece_set;

fn main() {
    let board = TraditionalBoardGraph::new();
    let move_tables = board.0.move_tables();
    let mut position = Position::new_traditional();
    println!("{:?}", move_tables.perft(&mut position, 5));
}
