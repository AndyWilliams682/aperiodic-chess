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
    let move_tables = MoveTables::new(
        board.0.king_move_table(),
        board.0.all_slide_tables(),
        board.0.knight_jumps_table(),
        board.0.pawn_tables(Color::White),
        board.0.pawn_tables(Color::Black)
    );
    let mut position = Position::new_traditional();
    println!("{:?}", move_tables.perft(&mut position, 5));
}
