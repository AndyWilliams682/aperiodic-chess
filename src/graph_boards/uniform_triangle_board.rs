use std::collections::HashSet;

use crate::graph_boards::graph_board::{GraphBoard, UniformTileOrientation, TileIndex, Tile};
use crate::piece_set::Color;
use crate::limited_int::LimitedInt;


// Convention:
//    0 is the forward direction for White
//    1 is the forward-left direction, continuing counter-clockwise until 7, which is forward-right
//    Even directions are orthogonal, odd directions are diagonal
pub type TriangularDirection = LimitedInt<6>;


#[derive(Debug)]
pub struct UniformTriangleBoardGraph(pub GraphBoard<1, 6>);

impl UniformTriangleBoardGraph {
    pub fn new() -> Self {
        let mut board_graph = GraphBoard::new();
        for tile in 0..55 {
            board_graph.add_node(Self::new_tile(TileIndex::new(tile)));
        }
        for tile_idx in board_graph.node_indices() {
            for direction in Self::get_valid_directions(tile_idx) {
                let other_idx = TileIndex::from((tile_idx.index() as i32 + Self::get_tile_index_shift(tile_idx, &direction)) as u32);
                board_graph.add_edge(tile_idx, other_idx, direction);
            }
        }
        return UniformTriangleBoardGraph(board_graph)
    }

    fn row_length(n: TileIndex) -> i32 {
        return match n.index() as i32 {
            00..=9  => 10,
            10..=18 => 9,
            19..=26 => 8,
            27..=33 => 7,
            34..=39 => 6,
            40..=44 => 5,
            45..=48 => 4,
            49..=51 => 3,
            52..=53 => 2,
            54      => 1,
            _       => 0
        }
    }

    fn new_tile(source: TileIndex) -> Tile<1> {
        let pawn_start = match source.index() {
            3 | 12 | 20 | 27 => Some(Color::White),
            6 | 16 | 25 | 33 => Some(Color::Black),
            _ => None
        };
        return Tile { orientation: UniformTileOrientation::new(0), pawn_start }
    }
   
    fn get_valid_directions(source: TileIndex) -> Vec<TriangularDirection> {
        let mut result = TriangularDirection::all_values();
        let mut invalid = HashSet::new();
       
        match source.index() { // TODO: Rewrite more elegantly
            0..=9 => {
                invalid.insert(4);
                invalid.insert(5);
            },
            _ => {}
        };
        match source.index() { // TODO: Rewrite more elegantly
            0 | 10 | 19 | 27 | 34 | 40 | 45 | 49 | 52 | 54 => {
                invalid.insert(2);
                invalid.insert(3);
            },
            _ => {}
        };
        match source.index() { // TODO: Rewrite more elegantly
            9 | 18 | 26 | 33 | 39 | 44 | 48 | 51 | 53 | 54 => {
                invalid.insert(0);
                invalid.insert(1);
            },
            _ => {}
        };
        for direction in invalid {
            result.retain(|element| element.0 != direction);
        }
        return result
    }
   
    fn get_tile_index_shift(source: TileIndex, direction: &TriangularDirection) -> i32 {
        let row = Self::row_length(source);
        return match direction.0 {
            0 => 1,
            1 => row,
            2 => row - 1,
            3 => -1,
            4 => -row - 1,
            5 => -row,
            _ => 0 // TODO: Remove impossible match arms? I've covered the whole space above
        }
    }
}
