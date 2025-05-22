use petgraph::graph::NodeIndex;

use crate::position::PieceType;


#[derive(Debug, PartialEq, Clone)]
pub struct EnPassantData {
    pub capturable_tile: NodeIndex,
    pub piece_tile: NodeIndex
}

impl EnPassantData {
    pub fn new(capturable_tile: NodeIndex, piece_tile: NodeIndex) -> Self {
        Self { capturable_tile, piece_tile }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct Move {
    pub from_node: NodeIndex,
    pub to_node: NodeIndex,
    pub promotion: Option<PieceType>,
    pub en_passant_data: Option<EnPassantData>
}

impl Move {
    pub fn new(from_node: NodeIndex, to_node: NodeIndex, promotion: Option<PieceType>, en_passant_tile: Option<NodeIndex>) -> Self {
        let en_passant_data = match en_passant_tile {
            Some(node) => Some(EnPassantData::new(node, to_node)),
            None => None
        };
        return Self { from_node, to_node, promotion, en_passant_data }
    }
}
