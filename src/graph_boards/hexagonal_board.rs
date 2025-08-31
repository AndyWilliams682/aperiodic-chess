use std::collections::HashSet;

use crate::graph_boards::graph_board::{GraphBoard, UniformTileOrientation, TileIndex, Tile};
use crate::piece_set::Color;
use crate::limited_int::LimitedInt;

// Convention:
//    0 is the forward direction for White
//    1 is the forward-left direction, continuing counter-clockwise until 7, which is forward-right
//    Even directions are orthogonal, odd directions are diagonal
pub type HexagonalDirection = LimitedInt<12>;


#[derive(Debug)]
pub struct HexagonalBoardGraph(pub GraphBoard<1, 12>);

impl HexagonalBoardGraph {
    pub fn new() -> Self {
        let mut board_graph = GraphBoard::new();
        for tile in 0..91 {
            board_graph.add_node(Self::new_tile(TileIndex::new(tile)));
        }
        for tile_idx in board_graph.node_indices() {
            for direction in Self::get_valid_directions(tile_idx) {
                let other_idx = TileIndex::from((tile_idx.index() as i32 + Self::get_tile_index_shift(tile_idx, &direction)) as u32);
                board_graph.add_edge(tile_idx, other_idx, direction);
            }
        }
        return HexagonalBoardGraph(board_graph)
    }

    fn row_length(n: TileIndex) -> i32 {
        return match n.index() as i32 {
            0..=5 | 85..=90 => 6,
        6..=12 | 78..=84 => 7,
        13..=20 | 70..=77 => 8,
        21..=29 | 61..=69 => 9,
        30..=39 | 51..=60 => 10,
        40..=50 => 11,
        _ => 0
        }
    }

    fn new_tile(source: TileIndex) -> Tile<1> {
        let pawn_start = match source.index() {
            4 | 10 | 17 | 25 | 30..=34 => Some(Color::White),
            56..=60 | 65 | 73 | 80 | 86 => Some(Color::Black),
            _ => None
        };
        return Tile { id: source, occupant: None, orientation: UniformTileOrientation::new(0), pawn_start }
    }
   
    fn get_valid_directions(source: TileIndex) -> Vec<HexagonalDirection> {
        let mut result = HexagonalDirection::all_values();
        let mut invalid = HashSet::new();
       
        match source.index() {
            0..=5 => {
                invalid.insert(5);
                invalid.insert(6);
                invalid.insert(7);
                invalid.insert(8);
                invalid.insert(9);
            },
            50 | 60 | 69 | 77 | 84 | 90 => {
                invalid.insert(9);
                invalid.insert(10);
                invalid.insert(11);
                invalid.insert(0);
                invalid.insert(1);
            },
            40 | 51 | 61 | 70 | 78 | 85 => {
                invalid.insert(1);
                invalid.insert(2);
                invalid.insert(3);
                invalid.insert(4);
                invalid.insert(5);
            },
            7..=11 => {
                invalid.insert(7);
            },
            49 | 59 | 68 | 76 | 83 => {
                invalid.insert(11);
            },
            41 | 52 | 62 | 71 | 79 => {
                invalid.insert(3);
            },
            _ => {}
        };
       
        match source.index() {
            5 | 12 | 20 | 29 | 39 | 50 => {
                invalid.insert(7);
                invalid.insert(8);
                invalid.insert(9);
                invalid.insert(10);
                invalid.insert(11);
            },
            85..=90 => {
                invalid.insert(11);
                invalid.insert(0);
                invalid.insert(1);
                invalid.insert(2);
                invalid.insert(3);
            },
            0 | 6 | 13 | 21 | 30 | 40 => {
                invalid.insert(3);
                invalid.insert(4);
                invalid.insert(5);
                invalid.insert(6);
                invalid.insert(7);
            },
            79..=83 => {
                invalid.insert(1);
            },
            7 | 14 | 22 | 31 | 41 => {
                invalid.insert(5);
            },
            11 | 19 | 28 | 38 | 49 => {
                invalid.insert(9);
            },
            _ => {}
        };
       
        for direction in invalid {
            result.retain(|element| element.0 != direction);
        }
        return result
    }
   
    fn get_tile_index_shift(source: TileIndex, direction: &HexagonalDirection) -> i32 {
        let row = Self::row_length(source);
        return match direction.0 {
            0 => {
                if source.index() <= 40 { row + 1 }
                else { row }
            },
            1 => {
                if source.index() <= 30 { 2 * row + 2 }
                else if source.index() >= 41 { 2 * row - 2 }
                else { 2 * row + 1 }
            },
            2 => {
                if source.index() <= 40 { row }
                else { row - 1}
            },
            3 => {
                if source.index() <= 40 { row - 1 }
                else { row - 2 }
            },
            4 => -1,
            5 => {
                if source.index() <= 51 { -row - 1 }
                else { -row - 2 }
            },
            6 => {
                if source.index() <= 51 { -row }
                else { -row - 1}
            },
            7 => {
                if source.index() >= 62 { -2 * row - 2 }
                else if source.index() <= 41 { -2 * row + 2 }
                else { -2 * row - 1 }
            },
            8 => {
                if source.index() <= 51 { -row + 1 }
                else { -row }
            },
            9 => {
                if source.index() <= 51 { -row + 2 }
                else { -row + 1 }
            },
            10 => 1,
            11 => {
                if source.index() <= 40 { row + 2 }
                else { row + 1 }
            },
            _ => 0
        }
    }
}
