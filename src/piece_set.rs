
use crate::bit_board::BitBoard;
use crate::graph_board::TileIndex;


#[derive(Debug, PartialEq, Clone)]
pub enum Color {
    White,
    Black
}

impl Color {
    pub fn as_idx(&self) -> usize {
        return match self {
            Color::White => 0,
            Color::Black => 1
        }
    }

    pub fn opponent(&self) -> Self {
        return match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn
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
        occupied |= self.king;
        occupied |= self.queen;
        occupied |= self.rook;
        occupied |= self.bishop;
        occupied |= self.knight;
        occupied |= self.pawn;
        self.occupied = occupied
    }

    pub fn get_piece_at(&self, tile_index: TileIndex) -> Option<Piece> {
        if self.king.get_bit_at_tile(tile_index) == true {
            return Some(Piece::King)
        } else if self.queen.get_bit_at_tile(tile_index) == true {
            return Some(Piece::Queen)
        } else if self.rook.get_bit_at_tile(tile_index) == true {
            return Some(Piece::Rook)
        } else if self.bishop.get_bit_at_tile(tile_index) == true {
            return Some(Piece::Bishop)
        } else if self.knight.get_bit_at_tile(tile_index) == true {
            return Some(Piece::Knight)
        } else if self.pawn.get_bit_at_tile(tile_index) == true {
            return Some(Piece::Pawn)
        } else {
            return None
        }
    }

    pub fn get_bitboard_for_piece(&mut self, piece_type: &Piece) -> &mut BitBoard {
        return match piece_type {
            Piece::King => &mut self.king,
            Piece::Queen => &mut self.queen,
            Piece::Rook => &mut self.rook,
            Piece::Bishop => &mut self.bishop,
            Piece::Knight => &mut self.knight,
            Piece::Pawn => &mut self.pawn,
        };
    }

    pub fn move_piece(&mut self, from_tile: TileIndex, to_tile: TileIndex) {
        let piece_type = self.get_piece_at(from_tile).unwrap();
        let bitboard = self.get_bitboard_for_piece(&piece_type);
        bitboard.flip_bit_at_tile_index(from_tile);
        bitboard.flip_bit_at_tile_index(to_tile);
    }

    pub fn capture_piece(&mut self, capture_tile: TileIndex) {
        let piece_type = self.get_piece_at(capture_tile).unwrap();
        let bitboard = self.get_bitboard_for_piece(&piece_type);
        bitboard.flip_bit_at_tile_index(capture_tile);
    }

    pub fn promote_piece(&mut self, promotion_tile: TileIndex, promotion_target: &Piece) {
        // This assumes the move has been registered before applying the promotion
        self.pawn.flip_bit_at_tile_index(promotion_tile);
        let bitboard = self.get_bitboard_for_piece(promotion_target);
        bitboard.flip_bit_at_tile_index(promotion_tile);
    }

    pub fn return_piece(&mut self, captured_tile: TileIndex, captured_piece: &Piece) {
        let bitboard = self.get_bitboard_for_piece(captured_piece);
        bitboard.flip_bit_at_tile_index(captured_tile);
    } // Inverse of capture_piece
    
    pub fn demote_piece(&mut self, demotion_tile: TileIndex) {
        let piece_type = self.get_piece_at(demotion_tile).unwrap();
        let bitboard = self.get_bitboard_for_piece(&piece_type);
        bitboard.flip_bit_at_tile_index(demotion_tile);
        self.pawn.flip_bit_at_tile_index(demotion_tile);
    } // inverse of promote_piece
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    #[test]
    fn test_get_piece_at_tile() {
        let piece_set = &Position::new_traditional().pieces[0];
        assert_eq!(
            piece_set.get_piece_at(TileIndex::new(0)).unwrap(),
            Piece::Rook
        );
        assert_eq!(
            piece_set.get_piece_at(TileIndex::new(17)),
            None
        )
    }

    #[test]
    fn test_get_bitboard_for_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        assert_eq!(
            *piece_set.get_bitboard_for_piece(&Piece::King),
            BitBoard::new(16)
        )
    }

    #[test]
    fn test_move_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let from_tile = TileIndex::new(1);
        let to_tile = TileIndex::new(18);
        piece_set.move_piece(from_tile, to_tile);
        assert_eq!(
            piece_set.knight,
            BitBoard::from_ints(vec![6, 18])
        );
    }

    #[test]
    fn test_capture_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let capture_tile = TileIndex::new(0);
        piece_set.capture_piece(capture_tile);
        assert_eq!(
            piece_set.rook,
            BitBoard::from_ints(vec![7])
        )
    }

    #[test]
    fn test_promote_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let promotion_tile = TileIndex::new(8);
        piece_set.promote_piece(promotion_tile, &Piece::Queen);
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
        let captured_tile = TileIndex::new(16);
        piece_set.return_piece(captured_tile, &Piece::Rook);
        assert_eq!(
            piece_set.rook,
            BitBoard::from_ints(vec![0, 7, 16])
        )
    }

    #[test]
    fn test_demote_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let demotion_tile = TileIndex::new(0);
        piece_set.demote_piece(demotion_tile);
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
        piece_set.capture_piece(TileIndex::new(0));
        piece_set.update_occupied();
        assert_eq!(
            piece_set.occupied,
            BitBoard::new(65534) // 2 ** 16 - 2
        )
    }
}