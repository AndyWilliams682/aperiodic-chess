use std::collections::{HashSet, HashMap};

use crate::graph_boards::graph_board::{GraphBoard, UniformTileOrientation, TileIndex, Tile};
use crate::piece_set::Color;
use crate::limited_int::LimitedIntTrait;
use crate::create_limited_int;

// Convention:
//    0 is the forward direction for White
//    1 is the forward-left direction, continuing counter-clockwise until 7, which is forward-right
//    Even directions are orthogonal, odd directions are diagonal
create_limited_int!(AperiodicDirection, 10);
create_limited_int!(AperiodicOrientation, 6);

#[derive(Debug)]
pub struct AperiodicBoardGraph(pub GraphBoard<AperiodicOrientation, AperiodicDirection>);

impl AperiodicBoardGraph {
    pub fn new() -> Self {
        let mut board_graph = GraphBoard::new();
        for tile in 0..122 {
            board_graph.add_node(Self::new_tile(tile));
        }
        for tile_idx in board_graph.node_indices() {
            for direction in Self::get_valid_directions(tile_idx) {
                let other_idx = TileIndex::from((tile_idx.index() as i32 + Self::get_tile_index_shift(tile_idx, &direction)) as u32);
                board_graph.add_edge(tile_idx, other_idx, direction);
            }
        }
        return AperiodicBoardGraph(board_graph)
    }

    fn new_tile(source: i32) -> Tile<AperiodicOrientation> {
        let pawn_start = match source {
            6  | 16 | 26 | 35 | 57  | 80  | 93  | 103 | 104 => Some(Color::White),
            70 | 71 | 72 | 85 | 95  | 106 | 107 | 110 | 121 => Some(Color::Black),
            _ => None
        };
        let orientation_list = [
            0, 4, 5, 0, 5, 0, 5, 0, 5, 1, 0, 5, 0, 2, 0, 2, 0, 4, 1, 0, // 20
            2, 0, 4, 1, 1, 1, 3, 1, 1, 1, 3, 2, 3, 0, 5, 2, 3, 1, 3, 0, // 40
            5, 2, 1, 2, 1, 0, 4, 0, 0, 2, 1, 0, 2, 0, 4, 5, 1, 4, 5, 4, // 60
            2, 0, 4, 5, 1, 1, 1, 3, 3, 3, 1, 1, 3, 3, 0, 5, 4, 5, 3, 2, // 80
            3, 2, 1, 5, 3, 2, 3, 2, 1, 0, 5, 2, 1, 4, 0, 4, 2, 1, 4, 1, // 100
            0, 2, 3, 2, 1, 0, 3, 5, 1, 5, 3, 2, 3, 0, 5, 1, 4, 5, 2, 1, // 120
            4, 2 // 122
        ];
        let orientation = AperiodicOrientation(orientation_list[source as usize]);
        return Tile { orientation, pawn_start }
    }

    fn get_valid_directions(source: TileIndex) -> Vec<AperiodicDirection> {
        let mut result = AperiodicDirection::all_values();
        let mut invalid = HashSet::new();
       
        if [9, 17, 18, 27, 36, 46, 47, 48, 69, 71, 83, 95, 107, 114, 117].contains(&source.index()) {
            invalid.insert(0);
        }

        if [0, 8, 9, 10, 18, 19, 27, 35, 36, 47, 48, 59, 61, 69, 71, 81, 82, 95, 106, 107, 109, 116, 117].contains(&source.index()) {
            invalid.insert(1);
        }

        if [0, 5, 7, 18, 19, 59, 61, 82, 109, 116].contains(&source.index()) {
            invalid.insert(2);
        }

        if [0, 2, 3, 4, 5, 6, 7, 14, 16, 18, 59, 82, 84, 94, 102, 103, 104, 118].contains(&source.index()) {
            invalid.insert(3);
        }

        if [0, 2, 4, 6, 7, 37, 38, 59, 84, 94, 96, 104, 115, 118, 119].contains(&source.index()) {
            invalid.insert(4);
        }

        if [1, 2, 4, 6, 7, 16, 17, 37, 38, 60, 62, 72, 84, 91, 92, 94, 96, 104, 105, 106, 110, 115, 118, 119, 121].contains(&source.index()) {
            invalid.insert(5);
        }

        if [1, 4, 6, 7, 38, 60, 83, 84, 94, 96, 104, 115, 118, 119].contains(&source.index()) {
            invalid.insert(6);
        }

        if [1, 7, 17, 27, 38, 46, 48, 49, 58, 60, 71, 83, 84, 94, 96, 97, 100, 104, 107, 113, 114, 115, 119].contains(&source.index()) {
            invalid.insert(7);
        }

        if [9, 17, 27, 36, 46, 47, 48, 69, 71, 83, 95, 107, 114, 117].contains(&source.index()) {
            invalid.insert(8);
        }

        if [9, 13, 15, 17, 18, 20, 27, 28, 36, 46, 47, 48, 69, 71, 80, 83, 90, 95, 107, 114, 117].contains(&source.index()) {
            invalid.insert(9);
        }

        for direction in invalid {
            result.retain(|element| element.0 != direction);
        }
        return result
    }
}
