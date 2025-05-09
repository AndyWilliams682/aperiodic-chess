use petgraph::graph::NodeIndex;

use crate::position::PieceType;


pub struct Move {
    pub from_node: NodeIndex,
    pub to_node: NodeIndex,
    pub promotion: Option<PieceType>
}

impl Move {
    pub fn new(from_node: NodeIndex, to_node: NodeIndex, promotion: Option<PieceType>) -> Self {
        return Self { from_node, to_node, promotion }
    }
}
