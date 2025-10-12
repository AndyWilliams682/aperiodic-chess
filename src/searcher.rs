use crate::{chess_move::Move, evaluator::{Evaluator, CHECKMATED_SCORE}, move_generator::MoveTables, position::Position, transposition_table::{TranspositionTable, Flag}};

#[derive(Debug)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub best_score: i32
}

pub struct Searcher {
    transposition_table: TranspositionTable,
    evaluator: Evaluator,
    pub movegen: MoveTables,
    nodes_searched: usize,
}

impl Searcher {
    pub fn new(movegen: MoveTables) -> Self {
        Searcher {
            transposition_table: TranspositionTable::new(),
            evaluator: Evaluator::new(&movegen),
            movegen,
            nodes_searched: 0
        }
    }

    pub fn alpha_beta(&mut self, position: &mut Position, mut alpha: i32, beta: i32, depth: u8) -> i32 {
        
        if depth == 0 {
            return self.evaluator.static_evaluate(position) as i32
        }

        // --- TRANSPOSITION TABLE PROBE (Optional but highly recommended) ---
        let key = position.get_zobrist();
        if let Some(tt_score) = self.transposition_table.retrieve(key, depth, alpha, beta) {
            return tt_score;
        }

        // --- BASE CASE 2: Check for Game Over (Mate/Stalemate) ---
        let legal_moves = self.movegen.get_legal_moves(position);
        if legal_moves.is_empty() {
            return if position.is_checkmate(&self.movegen) {
                // Return a mate score adjusted by depth (shallower mate is better)
                -CHECKMATED_SCORE as i32 + depth as i32
            } else {
                // Stalemate
                0 
            };
        }

        // --- ITERATION AND RECURSION ---
        let mut best_score = i32::MIN;
        let mut best_move: Option<Move> = None;
        let mut flag = Flag::UpperBound; // Default flag, assuming score will be < beta

        // 1. Move Ordering/Generation
        // (Move ordering is critical! Sort moves by importance: TT-move, captures, checks, etc.)
        // let ordered_moves = self.order_moves(position, legal_moves);

        for current_move in self.movegen.get_legal_moves(position) {
            position.make_legal_move(&current_move);
            let score = -self.alpha_beta(position, -beta, -alpha, depth - 1);
            position.unmake_legal_move(&current_move);

            if score > best_score {
                best_score = score;
                best_move = Some(current_move);
            }

            // Update Alpha
            alpha = alpha.max(best_score);

            // Beta Cut-off (Pruning)
            if alpha >= beta {
                flag = Flag::LowerBound; // We found a move that's too good; opponent avoids this line
                // Optional: Store a "Killer Move" or "History Heuristic" here
                break; // PRUNE!
            }
        }
        
        // --- TRANSPOSITION TABLE STORE ---
        if best_score >= beta {
            flag = Flag::LowerBound; // Alpha was already updated to be >= beta
        } else if best_score > alpha {
            flag = Flag::Exact; // The score fell strictly between the original alpha and beta
        } else {
            flag = Flag::UpperBound; // best_score <= alpha (the upper bound on the true score)
        }

        self.transposition_table.store(key, best_score, depth, flag, best_move);

        return best_score;
    }

    pub fn get_best_move(&mut self, position: &mut Position, max_depth: u8) -> SearchResult {
        let legal_moves = self.movegen.get_legal_moves(position);
        
        // Handle no moves case (mate or stalemate)
        if legal_moves.is_empty() {
            return SearchResult { best_move: None, best_score: 0 };
        }

        let mut best_score = i32::MIN;
        let mut best_move: Option<Move> = None;

        // Start with a large window for alpha and beta
        // These are the "fail-soft" bounds for the top level search.
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX; 

        // 2. Iterate through all root moves
        for current_move in legal_moves {
            // 3. Make the move on the board
            position.make_legal_move(&current_move);
            // 4. Call the Negamax Alpha-Beta function
            // We flip alpha and beta and negate the result as required by Negamax.
            println!("{:?}", max_depth);
            let score = -self.alpha_beta(position, -beta, -alpha, max_depth - 1);
            // 5. Unmake the move
            position.unmake_legal_move(&current_move);

            // 6. Update the Best Move and Score
            if score > best_score {
                best_score = score;
                best_move = Some(current_move);
                // 7. Update the root alpha bound
                alpha = alpha.max(best_score);
            }
        }
        
        // Return the final result
        SearchResult {
            best_move,
            best_score
        }
    }
}
