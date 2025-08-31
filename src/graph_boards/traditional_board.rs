use std::collections::HashSet;

use crate::graph_boards::graph_board::{GraphBoard, UniformTileOrientation, TileIndex, Tile};
use crate::piece_set::Color;
use crate::limited_int::LimitedInt;

// Convention:
//    0 is the forward direction for White
//    1 is the forward-left direction, continuing counter-clockwise until 7, which is forward-right
//    Even directions are orthogonal, odd directions are diagonal
// create_limited_int!(TraditionalDirection, 8);

pub type TraditionalDirection = LimitedInt::<8>;


#[derive(Debug)]
pub struct TraditionalBoardGraph(pub GraphBoard<1, 8>);

impl TraditionalBoardGraph {
    pub fn new() -> Self {
        let mut board_graph = GraphBoard::new();
        for tile in 0..64 {
            board_graph.add_node(Self::new_tile(TileIndex::new(tile)));
        }
        for tile_idx in board_graph.node_indices() {
            for direction in Self::get_valid_directions(tile_idx) {
                let other_idx = TileIndex::from((tile_idx.index() as i32 + Self::get_tile_index_shift(&direction)) as u32);
                board_graph.add_edge(tile_idx, other_idx, direction);
            }
        }
        return TraditionalBoardGraph(board_graph)
    }

    fn new_tile(source_tile: TileIndex) -> Tile<1> {
        if source_tile.index() / 8 == 1 {
            return Tile { id: source_tile, occupant: None, orientation: UniformTileOrientation::new(0), pawn_start: Some(Color::White) }
        } else if source_tile.index() / 8 == 6 {
            return Tile { id: source_tile, occupant: None, orientation: UniformTileOrientation::new(0), pawn_start: Some(Color::Black) }
        } else {
            return Tile { id: source_tile, occupant: None, orientation: UniformTileOrientation::new(0), pawn_start: None }
        }
    }
   
    // This function is used for making the empty traditional board
    fn get_valid_directions(source_tile: TileIndex) -> Vec<TraditionalDirection> {
        let mut result = TraditionalDirection::all_values();
        let mut invalid = HashSet::new();
        if source_tile.index() % 8 == 0 {
            invalid.insert(1);
            invalid.insert(2);
            invalid.insert(3);
        } else if source_tile.index() % 8 == 7 {
            invalid.insert(5);
            invalid.insert(6);
            invalid.insert(7);
        }
        if source_tile.index() <= 7 {
            invalid.insert(3);
            invalid.insert(4);
            invalid.insert(5);
        } else if source_tile.index() >= 56 {
            invalid.insert(1);
            invalid.insert(0);
            invalid.insert(7);
        }
        for direction in invalid {
            result.retain(|element| element.0 != direction);
        }
        return result
    }
   
    // This function is used for making the empty traditional board
    fn get_tile_index_shift(direction: &TraditionalDirection) -> i32 {
        let sign = match &direction.0 {
            2..=5 => -1,
            _ => 1,
        };
        let shift = match direction.0 % 4 {
            0 => 8,
            1 => 7,
            2 => 1,
            3 => 9,
            _ => 0
        };
        return shift * sign
    }   
}
