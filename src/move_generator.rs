
use crate::{
    bit_board::{BitBoard, BitBoardMoves},
    chess_move::{EnPassantData, Move},
    graph_board::TileIndex,
    position::Position,
    piece_set::{Color, Piece},
    movement_tables::{JumpTable, SlideTables, PawnTables},
};

pub struct MoveTables {
    pub king_table: JumpTable, // king_table is it's own reverse
    pub slide_tables: SlideTables,
    pub knight_table: JumpTable,
    pub white_pawn_tables: PawnTables,
    pub black_pawn_tables: PawnTables,
    pub reverse_slide_tables: Vec<JumpTable>,
    pub reverse_knight_table: JumpTable,
    pub reverse_white_pawn_table: JumpTable,
    pub reverse_black_pawn_table: JumpTable
}

impl MoveTables {
    pub fn query_piece(&self, piece_type: &Piece, source_tile: TileIndex, occupied: BitBoard) -> BitBoard {
        return match piece_type {
            Piece::King => self.king_table[source_tile],
            Piece::Queen => self.slide_tables.query(&source_tile, &occupied, true, true),
            Piece::Rook => self.slide_tables.query(&source_tile, &occupied, true, false),
            Piece::Bishop => self.slide_tables.query(&source_tile, &occupied, false, true),
            Piece::Knight => self.knight_table[source_tile],
            _ => BitBoard::empty() // Pawns are handled in a different function
        }
    }

    pub fn query_pawn(&self, color: &Color, source_tile: TileIndex, enemies: &BitBoard, occupied: BitBoard, current_ep_data: &Option<EnPassantData>) -> BitBoard {
        let pawn_tables = match color {
            Color::White => &self.white_pawn_tables,
            Color::Black => &self.black_pawn_tables
        };
        let mut all_moves = BitBoard::empty();
        let single_moves = pawn_tables.single_table[source_tile] & !occupied;
        all_moves |= pawn_tables.single_table[source_tile] & !occupied;
        if !single_moves.is_zero() { // Only check double moves if the single_move is unblocked
            all_moves |= *pawn_tables.double_table[source_tile].get(&BitBoard::empty()).unwrap() & !occupied;
        }
        all_moves |= pawn_tables.attack_table[source_tile] & *enemies;
        if let Some(data) = current_ep_data { // Can capture via EP even if no enemy is present
            all_moves |= pawn_tables.attack_table[source_tile] & BitBoard::from_ints(vec![data.passed_tile.index() as u128])
        }
        all_moves
    }

    fn get_pseudo_moves(&self, position: &Position) -> impl Iterator<Item=Move> {
        let active_player = &position.active_player;
        let active_pieces = &position.pieces[active_player.as_idx()];

        let enemy_occupants = position.pieces[position.active_player.opponent().as_idx()].occupied;
        let all_occupants = enemy_occupants | active_pieces.occupied;
        let current_ep = &position.record.en_passant_data;

        let mut piece_iters: Vec<BitBoardMoves> = vec![];

        let mut get_piece_iter = | mut piece_board: BitBoard, piece_type: &Piece | {
            let mut is_pawn = false;
            while !piece_board.is_zero() {
                let source_tile = piece_board.lowest_one().unwrap();

                let mut next_ep_data = None;
                let mut promotable_tiles = BitBoard::empty();
                let mut raw_attacks = if piece_type == &Piece::Pawn {
                    is_pawn = true;
                    let pawn_tables = match active_player {
                        Color::White => &self.white_pawn_tables,
                        Color::Black => &self.black_pawn_tables
                    };
                    next_ep_data = pawn_tables.en_passant_table[source_tile.index()].clone();
                    promotable_tiles = pawn_tables.promotion_board;
                    self.query_pawn(active_player, source_tile, &enemy_occupants, all_occupants, current_ep)
                } else {
                    self.query_piece(piece_type, source_tile, all_occupants)
                };

                raw_attacks &= !active_pieces.occupied;

                piece_iters.push(
                    BitBoardMoves::new(
                        source_tile,
                        is_pawn,
                        raw_attacks,
                        next_ep_data,
                        promotable_tiles
                    )
                );
                piece_board.flip_bit_at_tile_index(source_tile);
            }
        };

        get_piece_iter(active_pieces.king, &Piece::King);
        get_piece_iter(active_pieces.queen, &Piece::Queen);
        get_piece_iter(active_pieces.rook, &Piece::Rook);
        get_piece_iter(active_pieces.bishop, &Piece::Bishop);
        get_piece_iter(active_pieces.knight, &Piece::Knight);
        get_piece_iter(active_pieces.pawn, &Piece::Pawn);

        piece_iters.into_iter().flatten()
    }

    pub fn get_legal_moves(&self, position: &mut Position) -> Vec<Move> {
        let mut legal_moves = Vec::new();
        for chess_move in self.get_pseudo_moves(&position) {
            if !position.is_legal_move(&chess_move, &self) {
                continue;
            }
            legal_moves.push(chess_move);
        }
        legal_moves
    }

    // TODO: Rewrite to reduce code
    // I assume that get_legal_moves could be an iterator and .next() would handle this?
    pub fn has_legal_moves(&self, position: &mut Position) -> bool {
        for chess_move in self.get_pseudo_moves(&position) {
            if position.is_legal_move(&chess_move, &self) {
                return true;
            }
        }
        false
    }

    pub fn perft(&self, position: &mut Position, depth: u8) -> u64 {
        // TODO: May want to move to a separate Engine object?
        let mut output = 0;
       
        let legal_moves = self.get_legal_moves(position);
       
        if depth == 1 {
            return legal_moves.len() as u64;
        }
        for legal_move in legal_moves {
            position.make_legal_move(&legal_move);
            output += self.perft(position, depth - 1);
            position.unmake_legal_move(&legal_move);
        }
        output
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_board::TraditionalBoardGraph;

    fn test_move_tables() -> MoveTables {
        let board = TraditionalBoardGraph::new();
        board.0.move_tables()
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
        let color = &Color::White;
        let source_tile = TileIndex::new(9);
        let enemies = BitBoard::empty();
        let occupied = BitBoard::empty();
        assert_eq!( // Double and single
            move_tables.query_pawn(color, source_tile, &enemies, occupied, &None),
            BitBoard::from_ints(vec![17, 25])
        );
        assert_eq!( // Attacks, blocked double
            move_tables.query_pawn(color, source_tile, &BitBoard::from_ints(vec![16, 18]), BitBoard::from_ints(vec![25]), &None),
            BitBoard::from_ints(vec![16, 17, 18])
        );
        assert_eq!( // Blocked single, occupied attacks by allies
            move_tables.query_pawn(color, source_tile, &BitBoard::from_ints(vec![17]), BitBoard::from_ints(vec![16, 17, 18]), &None),
            BitBoard::empty()
        );
        assert_eq!( // Blocked single, occupied
            move_tables.query_pawn(color, source_tile, &BitBoard::empty(), BitBoard::from_ints(vec![17]), &None),
            BitBoard::empty()
        );
        assert_eq!( // Single move, no doubles
            move_tables.query_pawn(color, TileIndex::new(17), &enemies, occupied, &None),
            BitBoard::from_ints(vec![25])
        );
        assert_eq!( // En Passant Capture
            move_tables.query_pawn(
                color, source_tile, &enemies, occupied, 
                &Some(EnPassantData { passed_tile: TileIndex::new(16), occupied_tile: TileIndex::new(8) })
            ),
            BitBoard::from_ints(vec![16, 17, 25])
        );
        assert_eq!( // Irrelevant En Passant
            move_tables.query_pawn(
                color, source_tile, &enemies, occupied, 
                &Some(EnPassantData { passed_tile: TileIndex::new(19), occupied_tile: TileIndex::new(11) })
            ),
            BitBoard::from_ints(vec![17, 25])
        )
    }

    #[test]
    fn test_query_pawn_black() {
        let move_tables = test_move_tables();
        let color = &Color::Black;
        let source_tile = TileIndex::new(49);
        let enemies = BitBoard::empty();
        let occupied = BitBoard::empty();
        assert_eq!( // Double and single
            move_tables.query_pawn(color, source_tile, &enemies, occupied, &None),
            BitBoard::from_ints(vec![41, 33])
        );
        assert_eq!( // Attacks, blocked double
            move_tables.query_pawn(color, source_tile, &BitBoard::from_ints(vec![40, 42]), BitBoard::from_ints(vec![33]), &None),
            BitBoard::from_ints(vec![40, 41, 42])
        );
        assert_eq!( // Blocked single, occupied attacks by allies
            move_tables.query_pawn(color, source_tile, &BitBoard::from_ints(vec![41]), BitBoard::from_ints(vec![40, 41, 42]), &None),
            BitBoard::empty()
        );
        assert_eq!( // Blocked single, occupied
            move_tables.query_pawn(color, source_tile, &BitBoard::empty(), BitBoard::from_ints(vec![41]), &None),
            BitBoard::empty()
        );
        assert_eq!(
            move_tables.query_pawn(color, TileIndex::new(41), &enemies, occupied, &None),
            BitBoard::from_ints(vec![33])
        );
        assert_eq!( // En Passant Capture
            move_tables.query_pawn(
                color, source_tile, &enemies, occupied, 
                &Some(EnPassantData { passed_tile: TileIndex::new(40), occupied_tile: TileIndex::new(48) })
            ),
            BitBoard::from_ints(vec![40, 41, 33])
        );
        assert_eq!( // Irrelevant En Passant
            move_tables.query_pawn(
                color, source_tile, &enemies, occupied, 
                &Some(EnPassantData { passed_tile: TileIndex::new(43), occupied_tile: TileIndex::new(51) })
            ),
            BitBoard::from_ints(vec![41, 33])
        )
    }

    #[test]
    fn test_get_legal_moves() {
        let move_tables = test_move_tables();
        let mut position = Position::new_traditional();
       
        position.pieces[0].pawn.flip_bit_at_tile_index(TileIndex::new(12));
        position.pieces[1].queen.flip_bit_at_tile_index(TileIndex::new(28));
        position.pieces[0].pawn.flip_bit_at_tile_index(TileIndex::new(13));
        position.pieces[0].pawn.flip_bit_at_tile_index(TileIndex::new(21));
        position.pieces[0].update_occupied();
        position.pieces[1].update_occupied();
       
        let legal_moves = move_tables.get_legal_moves(&mut position);
       
        assert_eq!(
            legal_moves.get(0).unwrap(),
            &Move::new(TileIndex::new(4), TileIndex::new(13), None, None)
        ); // Evading with King
        assert_eq!(
            legal_moves.get(1).unwrap(),
            &Move::new(TileIndex::new(3), TileIndex::new(12), None, None)
        ); // Blocking with Queen
        assert_eq!(
            legal_moves.get(2).unwrap(),
            &Move::new(TileIndex::new(5), TileIndex::new(12), None, None)
        ); // Blocking with Bishop
        assert_eq!(
            legal_moves.get(3).unwrap(),
            &Move::new(TileIndex::new(6), TileIndex::new(12), None, None)
        ); // Blocking with Knight
        assert_eq!(
            legal_moves.get(4).unwrap(),
            &Move::new(TileIndex::new(21), TileIndex::new(28), None, None)
        ); // Capturing with Pawn
        assert_eq!(
            legal_moves.len(),
            5
        );
    }

    #[test]
    fn test_perft_to_6() {
        let move_tables = test_move_tables();
        let mut position = Position::new_traditional();
        assert_eq!(move_tables.perft(&mut position, 1), 20);
        assert_eq!(move_tables.perft(&mut position, 2), 400);
        assert_eq!(move_tables.perft(&mut position, 3), 8902);
        assert_eq!(move_tables.perft(&mut position, 4), 197281);
        assert_eq!(move_tables.perft(&mut position, 5), 4865609);
        // assert_eq!(move_tables.perft(&mut position, 6), 119060324);
    }
}
