use crate::piece_set::PieceType;
use crate::graph_board::TileIndex;


#[derive(Debug, PartialEq, Clone)]
pub struct EnPassantData {
    pub capturable_tile: TileIndex,
    pub piece_tile: TileIndex
}

impl EnPassantData {
    pub fn new(capturable_tile: TileIndex, piece_tile: TileIndex) -> Self {
        Self { capturable_tile, piece_tile }
    }
}


#[derive(Debug, PartialEq)]
pub struct Move {
    pub from_tile: TileIndex,
    pub to_tile: TileIndex,
    pub promotion: Option<PieceType>,
    pub en_passant_data: Option<EnPassantData>
}

impl Move {
    pub fn new(from_tile: TileIndex, to_tile: TileIndex, promotion: Option<PieceType>, en_passant_tile: Option<TileIndex>) -> Self {
        let en_passant_data = match en_passant_tile {
            Some(tile) => Some(EnPassantData::new(tile, to_tile)),
            None => None
        };
        return Self { from_tile, to_tile, promotion, en_passant_data }
    }
}
