use petgraph::graph::NodeIndex;

use crate::{bit_board::{BitBoard, BitBoardMoves}, chess_move::{EnPassantData, Move}, graph_board::{Color, JumpTable, SlideTables, PawnTables}, piece, position::{PieceSet, PieceType, Position}};

pub struct MoveTables {
    pub king_table: JumpTable, // king_table is it's own reverse
    pub slide_tables: SlideTables,
    pub knight_table: JumpTable,
    pub white_pawn_tables: PawnTables,
    pub black_pawn_tables: PawnTables,
    pub reverse_slide_tables: Vec<JumpTable>,
    pub reverse_knight_tables: JumpTable,
    pub reverse_white_pawn_table: JumpTable,
    pub reverse_black_pawn_table: JumpTable
}

impl MoveTables {
    fn new(king: JumpTable, slide: SlideTables, knight: JumpTable, white_pawn: PawnTables, black_pawn: PawnTables) -> Self {
        Self {
            king_table: king,
            slide_tables: slide.clone(),
            knight_table: knight.clone(),
            white_pawn_tables: white_pawn.clone(),
            black_pawn_tables: black_pawn.clone(),
            reverse_slide_tables: slide.reverse(),
            reverse_knight_tables: knight.reverse(),
            reverse_white_pawn_table: white_pawn.attack_table.reverse(),
            reverse_black_pawn_table: black_pawn.attack_table.reverse()
        }
    }

    fn query_piece(&self, piece_type: PieceType, source_node: NodeIndex, occupied: BitBoard) -> BitBoard {
        return match piece_type {
            PieceType::King => self.king_table[source_node],
            PieceType::Queen => self.slide_tables.query(source_node, occupied, true, true),
            PieceType::Rook => self.slide_tables.query(source_node, occupied, true, false),
            PieceType::Bishop => self.slide_tables.query(source_node, occupied, false, true),
            PieceType::Knight => self.knight_table[source_node],
            _ => BitBoard::empty() // Pawns are handled in a different function
        }
    }

    fn query_pawn(&self, color: Color, source_node: NodeIndex, enemies: BitBoard, occupied: BitBoard, current_ep_data: &Option<EnPassantData>) -> BitBoard {
        let pawn_tables = match color {
            Color::White => &self.white_pawn_tables,
            Color::Black => &self.black_pawn_tables
        };
        let mut all_moves = BitBoard::empty();
        let single_moves = pawn_tables.single_table[source_node] & !occupied;
        all_moves = all_moves | (pawn_tables.single_table[source_node] & !occupied);
        if !single_moves.is_zero() { // Only check double moves if the single_move is unblocked
            all_moves = all_moves | (*pawn_tables.double_table[source_node].get(&BitBoard::empty()).unwrap() & !occupied);
        }
        all_moves = all_moves | (pawn_tables.attack_table[source_node] & enemies);
        match current_ep_data { // Can capture via EP even if no enemy is present
            Some(data) => all_moves = all_moves | (pawn_tables.attack_table[source_node] & BitBoard::from_ints(vec![data.capturable_tile.index() as u128])),
            None => {}
        }
        all_moves
    }

    fn check_en_passantable(&self, color: Color, source_node: NodeIndex) -> Option<EnPassantData> {
        let pawn_tables = match color {
            Color::White => &self.white_pawn_tables,
            Color::Black => &self.black_pawn_tables
        };
        match pawn_tables.double_table[source_node].get(&BitBoard::empty()).unwrap().lowest_one() {
            Some(piece_tile) => {
                let capturable_tile = pawn_tables.single_table[source_node].lowest_one().unwrap();
                Some(EnPassantData { capturable_tile, piece_tile })
            },
            _ => None
        }
    }

    fn check_promotable(&self, color: Color, source_node: NodeIndex) -> Option<Vec<NodeIndex>> {
        let pawn_tables = match color {
            Color::White => &self.white_pawn_tables,
            Color::Black => &self.black_pawn_tables
        };
        let total_moves = pawn_tables.single_table[source_node] & pawn_tables.attack_table[source_node];
        let mut output = vec![];
        while !total_moves.is_zero() {
            let to_node = total_moves.lowest_one().unwrap();
            if pawn_tables.single_table[to_node].is_zero() {
                output.push(to_node)
            }
        }
        if output.len() > 0 {
            Some(output)
        } else {
            None
        }
    }

    fn get_pseudo_moves(&self, position: &mut Position) -> impl Iterator<Item=Move> {
        let active_player = position.active_player;
        let active_pieces = &position.pieces[active_player.as_idx()];

        let enemy_occupants = position.pieces[position.active_player.opponent().as_idx()].occupied;
        let all_occupants = enemy_occupants | active_pieces.occupied;
        let current_ep = &position.en_passant_data;

        let mut piece_iters: Vec<BitBoardMoves> = vec![];

        let mut get_piece_iter = | mut piece_board: BitBoard, piece_type: PieceType | {
            while !piece_board.is_zero() {
                let source_node = piece_board.lowest_one().unwrap();

                let mut next_ep_data = None;
                let mut promotable_tiles = None;
                let mut raw_attacks = if piece_type == PieceType::Pawn {
                    next_ep_data = self.check_en_passantable(active_player, source_node);
                    promotable_tiles = self.check_promotable(active_player, source_node);
                    self.query_pawn(active_player, source_node, enemy_occupants, all_occupants, current_ep)
                } else {
                    self.query_piece(piece_type, source_node, all_occupants)
                };

                raw_attacks = raw_attacks & !active_pieces.occupied;

                piece_iters.push(
                    BitBoardMoves::new(
                        source_node,
                        piece_type,
                        raw_attacks,
                        next_ep_data,
                        promotable_tiles
                    )
                );
                piece_board.flip_bit_at_node(source_node);
            }
        };

        get_piece_iter(active_pieces.king, PieceType::King);
        get_piece_iter(active_pieces.queen, PieceType::Queen);
        get_piece_iter(active_pieces.rook, PieceType::Rook);
        get_piece_iter(active_pieces.bishop, PieceType::Bishop);
        get_piece_iter(active_pieces.knight, PieceType::Knight);
        get_piece_iter(active_pieces.pawn, PieceType::Pawn);

        piece_iters.into_iter().flatten()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_board::TraditionalBoardGraph;

    fn test_move_tables() -> MoveTables {
        let board = TraditionalBoardGraph::new();
        MoveTables::new(
            board.0.king_move_table(),
            board.0.all_slide_tables(),
            board.0.knight_jumps_table(),
            board.0.pawn_tables(Color::White),
            board.0.pawn_tables(Color::Black)
        )
    }

    #[test]
    fn test_perft_one() {
        let move_tables = test_move_tables();
        let mut count = 0;
        for _chess_move in move_tables.get_pseudo_moves(&mut Position::new_traditional()) {
            count += 1;
        }
        assert_eq!(
            count,
            20
        )
    }

    #[test]
    fn test_query_pawn_white() {
        let move_tables = test_move_tables();
        let color = Color::White;
        let source_node = NodeIndex::new(9);
        let enemies = BitBoard::empty();
        let occupied = BitBoard::empty();
        assert_eq!( // Double and single
            move_tables.query_pawn(color, source_node, enemies, occupied, &None),
            BitBoard::from_ints(vec![17, 25])
        );
        assert_eq!( // Attacks, blocked double
            move_tables.query_pawn(color, source_node, BitBoard::from_ints(vec![16, 18]), BitBoard::from_ints(vec![25]), &None),
            BitBoard::from_ints(vec![16, 17, 18])
        );
        assert_eq!( // Blocked single, occupied attacks by allies
            move_tables.query_pawn(color, source_node, BitBoard::from_ints(vec![17]), BitBoard::from_ints(vec![16, 17, 18]), &None),
            BitBoard::empty()
        );
        assert_eq!( // Blocked single, occupied
            move_tables.query_pawn(color, source_node, BitBoard::empty(), BitBoard::from_ints(vec![17]), &None),
            BitBoard::empty()
        );
        assert_eq!( // Single move, no doubles
            move_tables.query_pawn(color, NodeIndex::new(17), enemies, occupied, &None),
            BitBoard::from_ints(vec![25])
        );
        assert_eq!( // En Passant Capture
            move_tables.query_pawn(
                color, source_node, enemies, occupied, 
                &Some(EnPassantData { capturable_tile: NodeIndex::new(16), piece_tile: NodeIndex::new(8) })
            ),
            BitBoard::from_ints(vec![16, 17, 25])
        );
        assert_eq!( // Irrelevant En Passant
            move_tables.query_pawn(
                color, source_node, enemies, occupied, 
                &Some(EnPassantData { capturable_tile: NodeIndex::new(19), piece_tile: NodeIndex::new(11) })
            ),
            BitBoard::from_ints(vec![17, 25])
        )
    }

    #[test]
    fn test_query_pawn_black() {
        let move_tables = test_move_tables();
        let color = Color::Black;
        let source_node = NodeIndex::new(49);
        let enemies = BitBoard::empty();
        let occupied = BitBoard::empty();
        assert_eq!( // Double and single
            move_tables.query_pawn(color, source_node, enemies, occupied, &None),
            BitBoard::from_ints(vec![41, 33])
        );
        assert_eq!( // Attacks, blocked double
            move_tables.query_pawn(color, source_node, BitBoard::from_ints(vec![40, 42]), BitBoard::from_ints(vec![33]), &None),
            BitBoard::from_ints(vec![40, 41, 42])
        );
        assert_eq!( // Blocked single, occupied attacks by allies
            move_tables.query_pawn(color, source_node, BitBoard::from_ints(vec![41]), BitBoard::from_ints(vec![40, 41, 42]), &None),
            BitBoard::empty()
        );
        assert_eq!( // Blocked single, occupied
            move_tables.query_pawn(color, source_node, BitBoard::empty(), BitBoard::from_ints(vec![41]), &None),
            BitBoard::empty()
        );
        assert_eq!(
            move_tables.query_pawn(color, NodeIndex::new(41), enemies, occupied, &None),
            BitBoard::from_ints(vec![33])
        );
        assert_eq!( // En Passant Capture
            move_tables.query_pawn(
                color, source_node, enemies, occupied, 
                &Some(EnPassantData { capturable_tile: NodeIndex::new(40), piece_tile: NodeIndex::new(48) })
            ),
            BitBoard::from_ints(vec![40, 41, 33])
        );
        assert_eq!( // Irrelevant En Passant
            move_tables.query_pawn(
                color, source_node, enemies, occupied, 
                &Some(EnPassantData { capturable_tile: NodeIndex::new(43), piece_tile: NodeIndex::new(51) })
            ),
            BitBoard::from_ints(vec![41, 33])
        )
    }
}
