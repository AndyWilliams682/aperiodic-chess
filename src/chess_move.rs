use crate::piece_set::PieceType;
use crate::graph_boards::graph_board::TileIndex;


#[derive(Debug, PartialEq, Clone)]
pub struct EnPassantData {
    pub passed_tile: TileIndex,
    pub occupied_tile: TileIndex
}

impl EnPassantData {
    pub fn new(passed_tile: TileIndex, occupied_tile: TileIndex) -> Self {
        Self { passed_tile, occupied_tile }
    }
}


#[derive(Debug, PartialEq, Clone)]
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

    pub fn from_input(from_tile: TileIndex, to_tile: TileIndex, promotion: Option<PieceType>, en_passant_data: Option<EnPassantData>) -> Self {
        return Self { from_tile, to_tile, promotion, en_passant_data }
    }
}
