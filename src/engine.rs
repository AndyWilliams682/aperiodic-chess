use crate::{chess_move::Move, evaluator::Evaluator, move_generator::MoveTables, position::Position};



pub struct Engine {
    pub move_tables: MoveTables,
    pub evaluator: Evaluator
}

impl Engine {
    pub fn new(move_tables: MoveTables) -> Self {
        let evaluator = Evaluator::new(&move_tables);
        return Self { move_tables, evaluator }
    }

    pub fn search_for_move(&self, position: &mut Position) -> Move {
        // TODO: Implement some actual method for doing this
        let moves = self.move_tables.get_legal_moves(position);
        let num_moves = moves.len();
        let move_idx = num_moves / 2;
        return moves[move_idx].clone()
    }
}