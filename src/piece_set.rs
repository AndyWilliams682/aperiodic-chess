use std::fmt;

use crate::bit_board::BitBoard;
use crate::constants::NUM_PIECE_TYPES;
use crate::graph_boards::graph_board::TileIndex;


#[derive(Debug, PartialEq, Clone, Copy)]
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

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let color_str = match self {
            Color::White => "White",
            Color::Black => "Black"
        };
        write!(f, "{}", color_str)
    }
}


#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn
}

impl PieceType {
    pub fn from_idx(idx: usize) -> Self {
        return match idx {
            0 => PieceType::King,
            1 => PieceType::Queen,
            2 => PieceType::Rook,
            3 => PieceType::Bishop,
            4 => PieceType::Knight,
            _ => PieceType::Pawn
        }
    }

    pub fn from_char(character: char) -> Self {
        return match character.to_ascii_lowercase() {
            'k' => PieceType::King,
            'q' => PieceType::Queen,
            'r' => PieceType::Rook,
            'b' => PieceType::Bishop,
            'n' => PieceType::Knight,
            _ => PieceType::Pawn
        }
    }

    pub fn as_idx(&self) -> usize {
        return match self {
            PieceType::King => 0,
            PieceType::Queen => 1,
            PieceType::Rook => 2,
            PieceType::Bishop => 3,
            PieceType::Knight => 4,
            PieceType::Pawn => 5
        }
    }

    pub fn as_char(&self) -> char {
        return match self {
            PieceType::King => '♔',
            PieceType::Queen => '♕',
            PieceType::Rook => '♖',
            PieceType::Bishop => '♗',
            PieceType::Knight => '♘',
            PieceType::Pawn => '♙'
        }
    }

    pub fn as_colored_char(&self, color: Color) -> char {
        return match color {
            Color::White => self.as_char(),
            Color::Black => self.as_char().to_lowercase().next().unwrap()
        }
    }
}


#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Piece {
    pub piece: PieceType,
    pub color: Color
}

impl Piece {
    pub fn display(&self) -> char {
        let mut symbol = match self.piece {
            PieceType::King => 'K',
            PieceType::Queen => 'Q',
            PieceType::Rook => 'R',
            PieceType::Bishop => 'B',
            PieceType::Knight => 'N',
            PieceType::Pawn => 'P',
        };
        if self.color == Color::Black {
            symbol = symbol.to_ascii_lowercase();
        }
        return symbol
    }
}


#[derive(Debug)]
pub struct PieceSet {
    // pub king: BitBoard,
    // pub queen: BitBoard,
    // pub rook: BitBoard,
    // pub bishop: BitBoard,
    // pub knight: BitBoard,
    // pub pawn: BitBoard,
    pub piece_boards: [BitBoard; NUM_PIECE_TYPES],
    pub occupied: BitBoard
}

impl PieceSet {
    pub fn empty() -> Self {
        Self {
            piece_boards: [BitBoard::empty(); NUM_PIECE_TYPES],
            occupied: BitBoard::empty()
        }
    }

    pub fn update_occupied(&mut self) {
        let mut occupied = BitBoard::empty();
        for piece_board in self.piece_boards {
            occupied |= piece_board
        }
        self.occupied = occupied
    }

    pub fn get_piece_at(&self, tile_index: &TileIndex) -> Option<PieceType> {
        for piece_idx in 0..NUM_PIECE_TYPES {
            if self.piece_boards[piece_idx].get_bit_at_tile(tile_index) == true {
                return Some(PieceType::from_idx(piece_idx))
            }
        }
        return None
    }

    pub fn get_bitboard_for_piece(&mut self, piece_type: &PieceType) -> &mut BitBoard {
        return &mut self.piece_boards[piece_type.as_idx()]
    }

    pub fn move_piece(&mut self, source_tile: TileIndex, destination_tile: TileIndex) {
        let piece_type = self.get_piece_at(&source_tile).unwrap();
        let bitboard = self.get_bitboard_for_piece(&piece_type);
        bitboard.flip_bit_at_tile_index(source_tile);
        bitboard.flip_bit_at_tile_index(destination_tile);
    }

    pub fn capture_piece(&mut self, capture_tile: TileIndex) {
        let piece_type = self.get_piece_at(&capture_tile).unwrap();
        let bitboard = self.get_bitboard_for_piece(&piece_type);
        bitboard.flip_bit_at_tile_index(capture_tile);
    }

    pub fn promote_piece(&mut self, promotion_tile: TileIndex, promotion_target: &PieceType) {
        // This assumes the move has been registered before applying the promotion
        self.piece_boards[PieceType::Pawn.as_idx()].flip_bit_at_tile_index(promotion_tile);
        let bitboard = self.get_bitboard_for_piece(promotion_target);
        bitboard.flip_bit_at_tile_index(promotion_tile);
    }

    pub fn return_piece(&mut self, captured_tile: TileIndex, captured_piece: &PieceType) {
        let bitboard = self.get_bitboard_for_piece(captured_piece);
        bitboard.flip_bit_at_tile_index(captured_tile);
    } // Inverse of capture_piece
    
    pub fn demote_piece(&mut self, demotion_tile: TileIndex) {
        let piece_type = self.get_piece_at(&demotion_tile).unwrap();
        let bitboard = self.get_bitboard_for_piece(&piece_type);
        bitboard.flip_bit_at_tile_index(demotion_tile);
        self.piece_boards[PieceType::Pawn.as_idx()].flip_bit_at_tile_index(demotion_tile);
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
            piece_set.get_piece_at(&TileIndex::new(0)).unwrap(),
            PieceType::Rook
        );
        assert_eq!(
            piece_set.get_piece_at(&TileIndex::new(17)),
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
        let source_tile = TileIndex::new(1);
        let destination_tile = TileIndex::new(18);
        piece_set.move_piece(source_tile, destination_tile);
        assert_eq!(
            piece_set.piece_boards[PieceType::Knight.as_idx()],
            BitBoard::from_ints(vec![6, 18])
        );
    }

    #[test]
    fn test_capture_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let capture_tile = TileIndex::new(0);
        piece_set.capture_piece(capture_tile);
        assert_eq!(
            piece_set.piece_boards[PieceType::Rook.as_idx()],
            BitBoard::from_ints(vec![7])
        )
    }

    #[test]
    fn test_promote_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let promotion_tile = TileIndex::new(8);
        piece_set.promote_piece(promotion_tile, &PieceType::Queen);
        assert_eq!(
            piece_set.piece_boards[PieceType::Pawn.as_idx()],
            BitBoard::from_ints(vec![9, 10, 11, 12, 13, 14, 15])
        );
        assert_eq!(
            piece_set.piece_boards[PieceType::Queen.as_idx()],
            BitBoard::from_ints(vec![3, 8])
        )
    }

    #[test]
    fn test_return_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let captured_tile = TileIndex::new(16);
        piece_set.return_piece(captured_tile, &PieceType::Rook);
        assert_eq!(
            piece_set.piece_boards[PieceType::Rook.as_idx()],
            BitBoard::from_ints(vec![0, 7, 16])
        )
    }

    #[test]
    fn test_demote_piece() {
        let piece_set = &mut Position::new_traditional().pieces[0];
        let demotion_tile = TileIndex::new(0);
        piece_set.demote_piece(demotion_tile);
        assert_eq!(
            piece_set.piece_boards[PieceType::Rook.as_idx()],
            BitBoard::from_ints(vec![7])
        );
        assert_eq!(
            piece_set.piece_boards[PieceType::Pawn.as_idx()],
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