use crate::piece_set::PieceType;
use crate::graph_boards::graph_board::TileIndex;


#[derive(Debug, PartialEq, Clone)]
pub struct EnPassantData {
    pub source_tile: TileIndex,
    pub passed_tile: TileIndex,
    pub occupied_tile: TileIndex
}

impl EnPassantData {
    pub fn new(source_tile: TileIndex, passed_tile: TileIndex, occupied_tile: TileIndex) -> Self {
        Self { source_tile, passed_tile, occupied_tile }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct Move {
    pub source_tile: TileIndex,
    pub destination_tile: TileIndex,
    pub promotion: Option<PieceType>,
    pub en_passant_data: Option<EnPassantData>
}

impl Move {
    pub fn new(source_tile: TileIndex, destination_tile: TileIndex, promotion: Option<PieceType>, en_passant_tile: Option<TileIndex>) -> Self {
        let en_passant_data = match en_passant_tile {
            Some(tile) => Some(EnPassantData::new(source_tile, tile, destination_tile)),
            None => None
        };
        return Self { source_tile, destination_tile, promotion, en_passant_data }
    }

    pub fn from_input(source_tile: TileIndex, destination_tile: TileIndex, promotion: Option<PieceType>, en_passant_data: Option<EnPassantData>) -> Self {
        return Self { source_tile, destination_tile, promotion, en_passant_data }
    }
}
