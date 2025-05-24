use petgraph::graph::NodeIndex;

use crate::bit_board::BitBoard;


#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug)]
pub struct PieceSet {
    pub king: BitBoard,
    pub queen: BitBoard,
    pub rook: BitBoard,
    pub bishop: BitBoard,
    pub knight: BitBoard,
    pub pawn: BitBoard,
    pub occupied: BitBoard
}

impl PieceSet {
    pub fn empty() -> Self {
        Self {
            king: BitBoard::empty(),
            queen: BitBoard::empty(),
            rook: BitBoard::empty(),
            bishop: BitBoard::empty(),
            knight: BitBoard::empty(),
            pawn: BitBoard::empty(),
            occupied: BitBoard::empty()
        }
    }

    pub fn update_occupied(&mut self) {
        let mut occupied = BitBoard::empty();
        occupied = occupied | self.king;
        occupied = occupied | self.queen;
        occupied = occupied | self.rook;
        occupied = occupied | self.bishop;
        occupied = occupied | self.knight;
        occupied = occupied | self.pawn;
        self.occupied = occupied
    }

    pub fn get_piece_at(&self, node: NodeIndex) -> Option<PieceType> {
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

    pub fn get_bitboard_for_piece(&mut self, piece_type: &PieceType) -> &mut BitBoard {
        return match piece_type {
            PieceType::King => &mut self.king,
            PieceType::Queen => &mut self.queen,
            PieceType::Rook => &mut self.rook,
            PieceType::Bishop => &mut self.bishop,
            PieceType::Knight => &mut self.knight,
            PieceType::Pawn => &mut self.pawn,
        };
    }

    pub fn move_piece(&mut self, from_node: NodeIndex, to_node: NodeIndex) {
        let piece_type = self.get_piece_at(from_node).unwrap();
        let bitboard = self.get_bitboard_for_piece(&piece_type);
        bitboard.flip_bit_at_node(from_node);
        bitboard.flip_bit_at_node(to_node);
    }

    pub fn capture_piece(&mut self, capture_node: NodeIndex) {
        let piece_type = self.get_piece_at(capture_node).unwrap();
        let bitboard = self.get_bitboard_for_piece(&piece_type);
        bitboard.flip_bit_at_node(capture_node);
    }

    pub fn promote_piece(&mut self, promotion_node: NodeIndex, promotion_target: &PieceType) {
        // This assumes the move has been registered before applying the promotion
        self.pawn.flip_bit_at_node(promotion_node);
        let bitboard = self.get_bitboard_for_piece(promotion_target);
        bitboard.flip_bit_at_node(promotion_node);
    }

    pub fn return_piece(&mut self, captured_node: NodeIndex, captured_piece: &PieceType) {
        let bitboard = self.get_bitboard_for_piece(captured_piece);
        bitboard.flip_bit_at_node(captured_node);
    } // Inverse of capture_piece
    
    pub fn demote_piece(&mut self, demotion_node: NodeIndex) {
        let piece_type = self.get_piece_at(demotion_node).unwrap();
        let bitboard = self.get_bitboard_for_piece(&piece_type);
        bitboard.flip_bit_at_node(demotion_node);
        self.pawn.flip_bit_at_node(demotion_node);
    } // inverse of promote_piece
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    #[test]
    fn test_get_piece_at_node() {
        let piece_set = &Position::new_traditional().pieces[0];
        assert_eq!(
            piece_set.get_piece_at(NodeIndex::new(0)).unwrap(),
            PieceType::Rook
        );
        assert_eq!(
            piece_set.get_piece_at(NodeIndex::new(17)),
            None
        )
    }

    #[test]
    fn test_get_bitboard_for_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        assert_eq!(
            *piece_set.get_bitboard_for_piece(&PieceType::King),
            BitBoard::new(16)
        )
    }

    #[test]
    fn test_move_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let from_node = NodeIndex::new(1);
        let to_node = NodeIndex::new(18);
        piece_set.move_piece(from_node, to_node);
        assert_eq!(
            piece_set.knight,
            BitBoard::from_ints(vec![6, 18])
        );
    }

    #[test]
    fn test_capture_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let capture_node = NodeIndex::new(0);
        piece_set.capture_piece(capture_node);
        assert_eq!(
            piece_set.rook,
            BitBoard::from_ints(vec![7])
        )
    }

    #[test]
    fn test_promote_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let promotion_node = NodeIndex::new(8);
        piece_set.promote_piece(promotion_node, &PieceType::Queen);
        assert_eq!(
            piece_set.pawn,
            BitBoard::from_ints(vec![9, 10, 11, 12, 13, 14, 15])
        );
        assert_eq!(
            piece_set.queen,
            BitBoard::from_ints(vec![3, 8])
        )
    }

    #[test]
    fn test_return_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let captured_node = NodeIndex::new(16);
        piece_set.return_piece(captured_node, &PieceType::Rook);
        assert_eq!(
            piece_set.rook,
            BitBoard::from_ints(vec![0, 7, 16])
        )
    }

    #[test]
    fn test_demote_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let demotion_node = NodeIndex::new(0);
        piece_set.demote_piece(demotion_node);
        assert_eq!(
            piece_set.rook,
            BitBoard::from_ints(vec![7])
        );
        assert_eq!(
            piece_set.pawn,
            BitBoard::from_ints(vec![0, 8, 9, 10, 11, 12, 13, 14, 15])
        )
    }

    #[test]
    fn test_update_occupied() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        piece_set.capture_piece(NodeIndex::new(0));
        piece_set.update_occupied();
        assert_eq!(
            piece_set.occupied,
            BitBoard::new(65534) // 2 ** 16 - 2
        )
    }
}