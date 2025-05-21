use std::sync::Arc;
use itertools::WithPosition;
use petgraph::graph::NodeIndex;

use crate::graph_board::Color;
use crate::bit_board::BitBoard;
use crate::chess_move::{EnPassantData, Move};


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
    pub player: Color,
    pub king: BitBoard,
    pub queen: BitBoard,
    pub rook: BitBoard,
    pub bishop: BitBoard,
    pub knight: BitBoard,
    pub pawn: BitBoard,
    pub occupied: BitBoard
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
                king: BitBoard::from_ints(vec![60]),
                queen: BitBoard::from_ints(vec![59]),
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

    pub fn get_bitboard_for_piece(&mut self, piece_type: PieceType) -> &mut BitBoard {
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

    fn return_piece(&mut self, captured_node: NodeIndex, captured_piece: PieceType) {
        let bitboard = self.get_bitboard_for_piece(captured_piece);
        bitboard.flip_bit_at_node(captured_node);
    } // Inverse of capture_piece
    
    fn demote_piece(&mut self, demotion_node: NodeIndex) {
        let piece_type = self.get_piece_at(demotion_node).unwrap();
        let bitboard = self.get_bitboard_for_piece(piece_type);
        bitboard.flip_bit_at_node(demotion_node);
        self.pawn.flip_bit_at_node(demotion_node);
    } // inverse of promote_piece
}


#[derive(Debug, Clone)]
pub struct PositionRecord {
    pub en_passant_data: Option<EnPassantData>,
    pub captured_piece: Option<PieceType>,
    pub previous_record: Option<Arc<PositionRecord>>,
    // previous_zobrist_key??
}

impl PositionRecord {
    pub fn default() -> PositionRecord {
        PositionRecord {
            en_passant_data: None,
            captured_piece: None,
            previous_record: None
        }
    }
   
    pub fn get_previous_record(&self) -> Option<Arc<PositionRecord>> {
        self.previous_record.as_ref().cloned()
    }
}


pub struct Position {
    pub active_player: Color,
    pub pieces: [PieceSet; 2],
    pub record: Arc<PositionRecord>
    // pub board_type
    // pub properties
}

impl Position {
    pub fn new_traditional() -> Self {
        return Self {
            active_player: Color::White,
            pieces: [
                PieceSet::new_traditional(Color::White),
                PieceSet::new_traditional(Color::Black)
            ],
            record: PositionRecord::default().into()
        }
    }

    pub fn new_hexagonal() -> Self {
        return Self {
            active_player: Color::White,
            pieces: [
                PieceSet::new_hexagonal(Color::White),
                PieceSet::new_hexagonal(Color::Black)
            ],
            record: PositionRecord::default().into()
        }
    }

    pub fn make_legal_move(&mut self, legal_move: Move) {
        // Assumes the move is legal?
        let player_idx = self.active_player.as_idx();
        let opponent_idx = self.active_player.opponent().as_idx();

        let from_node = legal_move.from_node;
        let to_node = legal_move.to_node;

        let moving_piece = self.pieces[player_idx].get_piece_at(from_node).unwrap();
        self.pieces[player_idx].move_piece(from_node, to_node);

        let target_piece = self.pieces[opponent_idx].get_piece_at(to_node);
        match target_piece {
            Some(_t) => self.pieces[opponent_idx].capture_piece(to_node),
            None => {}
        }

        match legal_move.promotion {
            Some(promotion_target) => self.pieces[player_idx].promote_piece(to_node, promotion_target),
            None => {}
        }

        if moving_piece == PieceType::Pawn {
            match &self.record.en_passant_data {
                Some(en_passant_data) if to_node == en_passant_data.capturable_tile => {
                    self.pieces[opponent_idx].capture_piece(en_passant_data.piece_tile)
                },
                _ => {}
            }
        }

        self.record = PositionRecord {
            en_passant_data: legal_move.en_passant_data,
            captured_piece: target_piece,
            previous_record: Some(self.record.clone())
        }.into();

        self.pieces[player_idx].update_occupied();
        self.pieces[opponent_idx].update_occupied();
        self.active_player = self.active_player.opponent();
    }

    pub fn unmake_legal_move(&mut self, legal_move: Move) {
        // Assumes the move was legal
        self.active_player = self.active_player.opponent();
        let player_idx = self.active_player.as_idx();
        let opponent_idx = self.active_player.opponent().as_idx();
       
        let from_node = legal_move.from_node;
        let to_node = legal_move.to_node;
       
        self.pieces[player_idx].move_piece(to_node, from_node);
       
        let captured_piece = self.record.captured_piece;
        match captured_piece {
            Some(piece_type) => self.pieces[opponent_idx].return_piece(to_node, piece_type),
            None => {}
        }
       
        match legal_move.promotion {
            Some(_t) => self.pieces[player_idx].demote_piece(from_node),
            None => {} // TODO: Use better syntax for cases like this, if Some(_t) = legal_move.promotion {}
        }
       
        self.pieces[player_idx].update_occupied();
        self.pieces[opponent_idx].update_occupied();
        if let Some(prev_record) = self.record.get_previous_record() {
            self.record = prev_record
        } else {
            self.record = PositionRecord::default().into();
        }
    }
}


mod tests {
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
        let position: Position = Position::new_hexagonal();
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
    fn test_get_bitboard_for_piece() {
        let mut piece_set = test_traditional_piece_set();
        let piece_type = PieceType::King;
        assert_eq!(
            *piece_set.get_bitboard_for_piece(piece_type),
            BitBoard::new(16)
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
    fn test_return_piece() {
        let mut piece_set = test_traditional_piece_set();
        let captured_node = NodeIndex::new(16);
        let captured_piece = PieceType::Rook;
        piece_set.return_piece(captured_node, captured_piece);
        assert_eq!(
            piece_set.rook.get_bit_at_node(captured_node),
            true
        )
    }

    #[test]
    fn test_demote_piece() {
        let mut piece_set = test_traditional_piece_set();
        let demotion_node = NodeIndex::new(0);
        piece_set.demote_piece(demotion_node);
        assert_eq!(
            piece_set.rook.get_bit_at_node(demotion_node),
            false
        );
        assert_eq!(
            piece_set.pawn.get_bit_at_node(demotion_node),
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
    fn test_en_passant_move() {
        let mut position = test_traditional_position();
        let to_node = NodeIndex::new(24);
        let legal_move = Move::new(
            NodeIndex::new(8),
            to_node,
            None,
            Some(NodeIndex::new(16))
        );
        position.make_legal_move(legal_move);
        assert_eq!(
            *position.record.en_passant_data.as_ref().unwrap(),
            EnPassantData::new(NodeIndex::new(16), to_node)
        )
    }

    #[test]
    fn test_en_passant_capture() {
        let mut position = test_traditional_position();
        let en_passant_node = NodeIndex::new(16);
        let captured_node = NodeIndex::new(24);
        let first_move = Move::new(
            NodeIndex::new(8),
            captured_node,
            None,
            Some(en_passant_node)
        );
        position.make_legal_move(first_move);
        let capturing_move = Move::new(
            NodeIndex::new(48),
            en_passant_node,
            None,
            None
        );
        position.make_legal_move(capturing_move);
        assert_eq!(
            position.pieces[0].pawn.get_bit_at_node(NodeIndex::new(24)),
            false
        );
        assert_eq!(
            position.pieces[1].pawn.get_bit_at_node(NodeIndex::new(16)),
            true
        )
    }

    #[test]
    fn test_make_legal_move() {
        let mut position = test_traditional_position();
        let from_node = NodeIndex::new(1);
        let to_node = NodeIndex::new(18);
        let legal_move = Move::new(from_node, to_node, None, None);
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

    #[test]
    fn test_sequential_moves() {
        let mut position = test_traditional_position();
        let first_move = Move::new(
            NodeIndex::new(12),
            NodeIndex::new(28),
            None,
            Some(NodeIndex::new(20))
        );
        let second_move = Move::new(
            NodeIndex::new(51),
            NodeIndex::new(35),
            None,
            Some(NodeIndex::new(43))
        );
        let third_move = Move::new(
            NodeIndex::new(28),
            NodeIndex::new(35),
            None,
            None
        );
        position.make_legal_move(first_move);
        position.make_legal_move(second_move);
        assert_eq!(
            *position.record.en_passant_data.as_ref().unwrap(),
            EnPassantData { capturable_tile: NodeIndex::new(43), piece_tile: NodeIndex::new(35) }
        );
        position.make_legal_move(third_move);
        assert_eq!(
            position.pieces[0].occupied,
            BitBoard::new(2_u128.pow(16) - 1 - 2_u128.pow(12) + 2_u128.pow(35))
        );
        assert_eq!(
            position.pieces[1].occupied,
            BitBoard::new(2_u128.pow(64) - 2_u128.pow(48) - 2_u128.pow(51))
        )
    }

    #[test]
    fn test_unmake_legal_move() {
        let mut position = test_traditional_position();
        position.make_legal_move(
            Move::new(NodeIndex::new(15), NodeIndex::new(31), None, Some(NodeIndex::new(23)))
        );
        let from_node = NodeIndex::new(1);
        let to_node = NodeIndex::new(18);
        let legal_move = Move::new(from_node, to_node, None, None);
        position.active_player = Color::White;
        position.make_legal_move(legal_move);
        let legal_move = Move::new(from_node, to_node, None, None);
        position.unmake_legal_move(legal_move);
        assert_eq!(
            position.pieces[0].knight.get_bit_at_node(from_node),
            true
        );
        assert_eq!(
            position.pieces[0].knight.get_bit_at_node(to_node),
            false
        );
        assert_eq!(
            position.record.en_passant_data,
            Some(EnPassantData { capturable_tile: NodeIndex::new(23), piece_tile: NodeIndex::new(31) })
        );
        let from_node = NodeIndex::new(8);
        let to_node = NodeIndex::new(16);
        let demotion_move = Move::new(from_node, to_node, Some(PieceType::Knight), None);
        position.make_legal_move(demotion_move);
        let demotion_move = Move::new(from_node, to_node, Some(PieceType::Knight), None);
        position.unmake_legal_move(demotion_move);
        assert_eq!(
            position.pieces[0].knight.get_bit_at_node(from_node) & position.pieces[0].knight.get_bit_at_node(to_node),
            false
        );
        assert_eq!(
            position.pieces[0].pawn.get_bit_at_node(from_node),
            true
        );
        let from_node = NodeIndex::new(0);
        let to_node = NodeIndex::new(56);
        let capture_move = Move::new(from_node, to_node, None, None);
        position.make_legal_move(capture_move);
        assert_eq!(
            position.record.captured_piece,
            Some(PieceType::Rook)
        );
        let capture_move = Move::new(from_node, to_node, None, None);
        position.unmake_legal_move(capture_move);
        assert_eq!(
            position.pieces[0].rook.get_bit_at_node(from_node),
            true
        );
        assert_eq!(
            position.pieces[0].rook.get_bit_at_node(to_node),
            false
        );
        assert_eq!(
            position.pieces[1].rook.get_bit_at_node(to_node),
            true
        );
    }
}
