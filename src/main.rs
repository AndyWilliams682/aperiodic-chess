use chess_move::Move;
use graph_board::{HexagonalBoardGraph, TraditionalBoardGraph, TileIndex};
use position::Position;

mod graph_board;
mod bit_board;
mod limited_int;
mod position;
mod chess_move;
mod move_generator;
mod piece_set;
mod movement_tables;
mod evaluator;

fn main() {
    let board = TraditionalBoardGraph::new();
    let move_tables = board.0.move_tables();
    let mut position = Position::new_traditional();
    position.make_legal_move(&Move::new(TileIndex::new(8), TileIndex::new(17), None, None));
    // println!("Square: {:?}", move_tables.perft(&mut position, 5));
    println!("{}", board.display(position, TileIndex::new(0), move_tables, true));

    let board = HexagonalBoardGraph::new();
    let move_tables = board.0.move_tables();
    let mut position = Position::new_hexagonal();
    // println!("Hex: {:?}", move_tables.perft(&mut position, 5));
}
