use petgraph::graph::{Node, NodeIndex};

use crate::graph_board::{SlideTables, JumpTable, Color};
use crate::bit_board::BitBoard;
use crate::chess_move::Move;


pub struct MoveTables {
    slide_tables: SlideTables,
    knight_table: JumpTable,
    king_table: JumpTable,
    white_pawn_move_table: JumpTable,
    black_pawn_move_table: JumpTable,
    white_pawn_attack_table: JumpTable,
    black_pawn_attack_table: JumpTable
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn
}

impl PieceType {
    fn all_variants() -> &'static [PieceType] {
        &[
            PieceType::King,
            PieceType::Queen,
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::Pawn
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PieceSet {
    player: Color,
    king: BitBoard,
    queen: BitBoard,
    rook: BitBoard,
    bishop: BitBoard,
    knight: BitBoard,
    pawn: BitBoard,
    occupied: BitBoard
}

impl PieceSet {
    fn new_traditional(color: Color) -> Self {
        return match color {
            Color::White => Self {
                player: color,
                king: BitBoard::from_ints(vec![4]),
                queen: BitBoard::from_ints(vec![3]),
                rook: BitBoard::from_ints(vec![0, 7]),
                bishop: BitBoard::from_ints(vec![2, 5]),
                knight: BitBoard::from_ints(vec![1, 6]),
                pawn: BitBoard::from_ints(vec![8, 9, 10, 11, 12, 13, 14, 15]), // 2 ** 16 - 1 (2 ** 8 - 1)
                occupied: BitBoard::from_ints(vec![
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
                ])
            },
            Color::Black => Self {
                player: color,
                king: BitBoard::from_ints(vec![59]),
                queen: BitBoard::from_ints(vec![60]),
                rook: BitBoard::from_ints(vec![56, 63]),
                bishop: BitBoard::from_ints(vec![58, 61]),
                knight: BitBoard::from_ints(vec![57, 62]),
                pawn: BitBoard::from_ints(vec![48, 49, 50, 51, 52, 53, 54, 55]),
                occupied: BitBoard::from_ints(vec![
                    63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48
                ])
            }
        }
    }

    fn new_hexagonal(color: Color) -> Self {
        return match color {
            Color::White => Self {
                player: color,
                king: BitBoard::from_ints(vec![1]),
                queen: BitBoard::from_ints(vec![6]),
                rook: BitBoard::from_ints(vec![3, 21]),
                bishop: BitBoard::from_ints(vec![0, 7, 15]),
                knight: BitBoard::from_ints(vec![2, 13]),
                pawn: BitBoard::from_ints(vec![30, 31, 32, 33, 34, 4, 10, 17, 25]), // 2 ** 16 - 1 (2 ** 8 - 1)
                occupied: BitBoard::from_ints(vec![
                    0, 1, 2, 3, 4, 6, 7, 10, 13, 15, 17, 21, 25, 30, 31, 32, 33, 34
                ])
            },
            Color::Black => Self {
                player: color,
                king: BitBoard::from_ints(vec![84]),
                queen: BitBoard::from_ints(vec![89]),
                rook: BitBoard::from_ints(vec![69, 87]),
                bishop: BitBoard::from_ints(vec![75, 83, 90]),
                knight: BitBoard::from_ints(vec![77, 88]),
                pawn: BitBoard::from_ints(vec![86, 80, 73, 65, 56, 57, 58, 59, 60]),
                occupied: BitBoard::from_ints(vec![
                    56, 57, 58, 59, 60, 65, 69, 73, 75, 77, 80, 83, 84, 86, 87, 88, 89, 90
                ])
            }
        }
    }

    fn get_piece_at(&self, node: NodeIndex) -> Option<PieceType> {
        if self.king.get_bit_at_node(node) == true {
            return Some(PieceType::King)
        } else if self.queen.get_bit_at_node(node) == true {
            return Some(PieceType::Queen)
        } else if self.rook.get_bit_at_node(node) == true {
            return Some(PieceType::Rook)
        } else if self.bishop.get_bit_at_node(node) == true {
            return Some(PieceType::Bishop)
        } else if self.knight.get_bit_at_node(node) == true {
            return Some(PieceType::Knight)
        } else if self.pawn.get_bit_at_node(node) == true {
            return Some(PieceType::Pawn)
        } else {
            return None
        }

    }
}

// impl PieceSet {
   
// }

pub struct Position {
    pub active_player: Color,
    pub pieces: [PieceSet; 2],
    pub en_passant_square: Option<NodeIndex>
    // pub board_type
    // pub properties
}

impl Position {
    fn new_traditional() -> Self {
        return Self {
            active_player: Color::White,
            pieces: [
                PieceSet::new_traditional(Color::White),
                PieceSet::new_traditional(Color::Black)
            ],
            en_passant_square: None
        }
    }

    fn new_hexagonal() -> Self {
        return Self {
            active_player: Color::White,
            pieces: [
                PieceSet::new_hexagonal(Color::White),
                PieceSet::new_hexagonal(Color::Black)
            ],
            en_passant_square: None
        }
    }

    fn make_legal_move(&mut self, legal_move: Move) {
        // Assumes the move is legal?
        let player_idx = match self.active_player {
            Color::White => 0,
            Color::Black => 1
        };

        let moving_piece = self.pieces[player_idx].get_piece_at(legal_move.from_node);
    }
}


mod tests {
    use super::*;

    fn test_traditional_position() -> Position {
        return Position::new_traditional()
    }

    #[test]
    fn test_new_traditional_occupied() {
        let position = test_traditional_position();
        let occupied = position.pieces[0].occupied | position.pieces[1].occupied;
        assert_eq!(
            occupied,
            BitBoard::from_ints(vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63
            ])
        )
    }

    #[test]
    fn test_new_hexagonal_occupied() {
        let position = Position::new_hexagonal();
        let occupied = position.pieces[0].occupied | position.pieces[1].occupied;
        assert_eq!(
            occupied,
            BitBoard::from_ints(vec![
                0, 1, 2, 3, 4, 6, 7, 10, 13, 15, 17, 21, 25, 30, 31, 32, 33, 34,
                56, 57, 58, 59, 60, 65, 69, 73, 75, 77, 80, 83, 84, 86, 87, 88, 89, 90
            ])
        )
    }

    #[test]
    fn test_get_piece_at_node() {
        let piece_set = PieceSet::new_traditional(Color::White);
        assert_eq!(
            piece_set.get_piece_at(NodeIndex::new(0)).unwrap(),
            PieceType::Rook
        )
    }
}
