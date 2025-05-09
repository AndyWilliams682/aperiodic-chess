use petgraph::graph::{Node, NodeIndex};

use crate::graph_board::{SlideTables, JumpTable, Color};
use crate::bit_board::BitBoard;
use crate::chess_move::Move;
use crate::piece;


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

    fn update_occupied(&mut self) {
        let mut occupied = BitBoard::empty();
        occupied = occupied | self.king;
        occupied = occupied | self.queen;
        occupied = occupied | self.rook;
        occupied = occupied | self.bishop;
        occupied = occupied | self.knight;
        occupied = occupied | self.pawn;
        self.occupied = occupied
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

    fn get_bitboard_for_piece(&mut self, piece_type: PieceType) -> &mut BitBoard {
        return match piece_type {
            PieceType::King => &mut self.king,
            PieceType::Queen => &mut self.queen,
            PieceType::Rook => &mut self.rook,
            PieceType::Bishop => &mut self.bishop,
            PieceType::Knight => &mut self.knight,
            PieceType::Pawn => &mut self.pawn,
        };
    }

    fn move_piece(&mut self, from_node: NodeIndex, to_node: NodeIndex) {
        let piece_type = self.get_piece_at(from_node).unwrap();
        let bitboard = self.get_bitboard_for_piece(piece_type);
        bitboard.flip_bit_at_node(from_node);
        bitboard.flip_bit_at_node(to_node);
    }

    fn capture_piece(&mut self, capture_node: NodeIndex) {
        let piece_type = self.get_piece_at(capture_node).unwrap();
        let bitboard = self.get_bitboard_for_piece(piece_type);
        bitboard.flip_bit_at_node(capture_node);
    }

    fn promote_piece(&mut self, promotion_node: NodeIndex, promotion_target: PieceType) {
        // This assumes the move has been registered before applying the promotion
        self.pawn.flip_bit_at_node(promotion_node);
        let bitboard = self.get_bitboard_for_piece(promotion_target);
        bitboard.flip_bit_at_node(promotion_node);
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
            Color::White => {
                self.active_player = Color::Black;
                0
            },
            Color::Black => {
                self.active_player = Color::White;
                1
            }
        };

        let from_node = legal_move.from_node;
        let to_node = legal_move.to_node;

        self.pieces[player_idx].move_piece(from_node, to_node);

        let target_piece = self.pieces[(player_idx + 1) % 2].get_piece_at(to_node);
        match target_piece {
            Some(_t) => self.pieces[(player_idx + 1) % 2].capture_piece(to_node),
            None => {}
        }

        match legal_move.promotion {
            Some(promotion_target) => self.pieces[player_idx].promote_piece(to_node, promotion_target),
            None => {}
        }

        self.pieces[player_idx].update_occupied();
        self.pieces[(player_idx + 1) % 2].update_occupied();
    }
}


mod tests {
    use crate::piece::Piece;

    use super::*;

    fn test_traditional_position() -> Position {
        return Position::new_traditional()
    }

    fn test_traditional_piece_set() -> PieceSet {
        return PieceSet::new_traditional(Color::White);
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
        let piece_set = test_traditional_piece_set();
        assert_eq!(
            piece_set.get_piece_at(NodeIndex::new(0)).unwrap(),
            PieceType::Rook
        )
    }

    #[test]
    fn test_move_piece() {
        let mut piece_set = test_traditional_piece_set();
        let from_node = NodeIndex::new(1);
        let to_node = NodeIndex::new(18);
        println!("{:?}", piece_set.knight);
        piece_set.move_piece(from_node, to_node);
        assert_eq!(
            piece_set.knight.get_bit_at_node(from_node),
            false
        );
        assert_eq!(
            piece_set.knight.get_bit_at_node(to_node),
            true
        )
    }

    #[test]
    fn test_capture_piece() {
        let mut piece_set = test_traditional_piece_set();
        println!("{:?}", piece_set.rook);
        let capture_node = NodeIndex::new(0);
        piece_set.capture_piece(capture_node);
        println!("{:?}", piece_set.rook);
        assert_eq!(
            piece_set.rook.get_bit_at_node(capture_node),
            false
        )
    }

    #[test]
    fn test_promote_piece() {
        let mut piece_set = test_traditional_piece_set();
        let promotion_node = NodeIndex::new(8);
        piece_set.promote_piece(promotion_node, PieceType::Queen);
        assert_eq!(
            piece_set.pawn.get_bit_at_node(promotion_node),
            false
        );
        assert_eq!(
            piece_set.queen.get_bit_at_node(promotion_node),
            true
        )
    }

    #[test]
    fn test_update_occupied() {
        let mut piece_set = test_traditional_piece_set();
        piece_set.capture_piece(NodeIndex::new(0));
        piece_set.update_occupied();
        assert_eq!(
            piece_set.occupied,
            BitBoard::new(65534) // 2 ** 16 - 2
        )
    }

    #[test]
    fn test_make_legal_move() {
        let mut position = test_traditional_position();
        let from_node = NodeIndex::new(1);
        let to_node = NodeIndex::new(18);
        let legal_move = Move::new(from_node, to_node, None);
        position.make_legal_move(legal_move);
        assert_eq!(
            position.pieces[0].knight.get_bit_at_node(from_node),
            false
        );
        assert_eq!(
            position.pieces[0].knight.get_bit_at_node(to_node),
            true
        )
    }
}
