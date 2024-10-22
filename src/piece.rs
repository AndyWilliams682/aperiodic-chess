

#[derive(PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub enum Piece {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King
}

impl Piece {
    #[inline]
    fn to_index(&self) -> usize {
        *self as usize
    }
}