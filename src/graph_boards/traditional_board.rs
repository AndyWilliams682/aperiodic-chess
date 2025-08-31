use std::collections::HashSet;

use crate::graph_boards::graph_board::{GraphBoard, UniformTileOrientation, TileIndex, Tile};
use crate::piece_set::{Color, PieceType};
use crate::bit_board::{BitBoard, BitBoardTiles};
use crate::position::Position;
use crate::move_generator::MoveTables;
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

    fn new_tile(source: TileIndex) -> Tile<1> {
        if source.index() / 8 == 1 {
            return Tile { id: source, occupant: None, orientation: UniformTileOrientation::new(0), pawn_start: Some(Color::White) }
        } else if source.index() / 8 == 6 {
            return Tile { id: source, occupant: None, orientation: UniformTileOrientation::new(0), pawn_start: Some(Color::Black) }
        } else {
            return Tile { id: source, occupant: None, orientation: UniformTileOrientation::new(0), pawn_start: None }
        }
    }
   
    // This function is used for making the empty traditional board
    fn get_valid_directions(source: TileIndex) -> Vec<TraditionalDirection> {
        let mut result = TraditionalDirection::all_values();
        let mut invalid = HashSet::new();
        if source.index() % 8 == 0 {
            invalid.insert(1);
            invalid.insert(2);
            invalid.insert(3);
        } else if source.index() % 8 == 7 {
            invalid.insert(5);
            invalid.insert(6);
            invalid.insert(7);
        }
        if source.index() <= 7 {
            invalid.insert(3);
            invalid.insert(4);
            invalid.insert(5);
        } else if source.index() >= 56 {
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
    
    pub fn display(&self, position: &Position, selected_tile: Option<TileIndex>, move_tables: &MoveTables, showing_indices: bool) -> String {
        let mut output: Vec<char> = " ____ ____ ____ ____ ____ ____ ____ ____\n|    |    |    |    |    |    |    |    |\n|____|____|____|____|____|____|____|____|\n|    |    |    |    |    |    |    |    |\n|____|____|____|____|____|____|____|____|\n|    |    |    |    |    |    |    |    |\n|____|____|____|____|____|____|____|____|\n|    |    |    |    |    |    |    |    |\n|____|____|____|____|____|____|____|____|\n|    |    |    |    |    |    |    |    |\n|____|____|____|____|____|____|____|____|\n|    |    |    |    |    |    |    |    |\n|____|____|____|____|____|____|____|____|\n|    |    |    |    |    |    |    |    |\n|____|____|____|____|____|____|____|____|\n|    |    |    |    |    |    |    |    |\n|____|____|____|____|____|____|____|____|"
            .chars().collect();
        let mut display_piece = | piece_board, piece_char: char | {
            for tile_idx in BitBoardTiles::new(piece_board) {
                let display_idx = 631 - 84 * (tile_idx.index() / 8) + 5 * (tile_idx.index() % 8);
                output[display_idx] = piece_char;
            }
        };

        let white_pieces = &position.pieces[0];
        let black_pieces = &position.pieces[1];
        let occupied = white_pieces.occupied | black_pieces.occupied;

        display_piece(white_pieces.king, 'K');
        display_piece(white_pieces.queen, 'Q');
        display_piece(white_pieces.rook, 'R');
        display_piece(white_pieces.bishop, 'B');
        display_piece(white_pieces.knight, 'N');
        display_piece(white_pieces.pawn, 'P');

        display_piece(black_pieces.king, 'k');
        display_piece(black_pieces.queen, 'q');
        display_piece(black_pieces.rook, 'r');
        display_piece(black_pieces.bishop, 'b');
        display_piece(black_pieces.knight, 'n');
        display_piece(black_pieces.pawn, 'p');

        let mut move_options = BitBoard::empty();
        if let Some(tile) = selected_tile {
            let selected_white = white_pieces.get_piece_at(tile);
            let selected_black = black_pieces.get_piece_at(tile);
            let selected_piece = selected_white.or(selected_black);
            let allied_occupied = match black_pieces.get_piece_at(tile) {
                Some(_t) => black_pieces.occupied,
                _ => white_pieces.occupied
            };
            move_options = match selected_piece {
                Some(PieceType::Pawn) => BitBoard::empty(), // TODO: Add pawn movement stuff here, depends on color
                None => BitBoard::empty(),
                _ => { // All non-Pawn PieceTypes
                    let display_idx = 631 - 84 * (tile.index() / 8) + 5 * (tile.index() % 8);
                    output[display_idx + 1] = '?';
                    move_tables.query_piece(&selected_piece.unwrap(), tile, occupied) & !allied_occupied
                }
            };
        }

        for tile_idx in BitBoardTiles::new(move_options) {
            let display_idx = 631 - 84 * (tile_idx.index() / 8) + 5 * (tile_idx.index() % 8);
            match white_pieces.get_piece_at(tile_idx).or(black_pieces.get_piece_at(tile_idx)) {
                Some(_t) => output[display_idx + 1] = '!',
                None => output[display_idx + 1] = '.'
            }
        }

        if showing_indices {
            for tile_idx in 0..=63 {
                let ones_digit = ((tile_idx % 10) as u8 + b'0') as char;
                let tens_digit = (((tile_idx / 10) % 10) as u8 + b'0') as char;
                let display_idx = 631 - 84 * (tile_idx / 8) + 5 * (tile_idx % 8);
                output[display_idx + 42] = tens_digit;
                output[display_idx + 43] = ones_digit;
            }
        }

        output.iter().collect()
    }
}
