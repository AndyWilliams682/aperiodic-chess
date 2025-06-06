use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashSet, HashMap};
use std::ops::{Deref, DerefMut};

use crate::bit_board::{BitBoard, CarryRippler, BitBoardTiles};
use crate::{create_limited_int, piece_set};
use crate::limited_int::LimitedIntTrait;
use crate::move_generator::MoveTables;
use crate::piece_set::{Color, Piece};
use crate::movement_tables::{JumpTable, DirectionalSlideTable, SlideTables, PawnTables};
use crate::position::Position;

pub type TileIndex = NodeIndex;

#[derive(Debug)]
pub struct Tile<N: LimitedIntTrait> {
    orientation: N,
    pawn_start: Option<Color>
}

// Generic graph that uses LimitedIntTrait for the edges
#[derive(Debug)]
pub struct BoardGraph<N: LimitedIntTrait, E: LimitedIntTrait>(Graph<Tile<N>, E>);

impl<
    N: LimitedIntTrait + std::cmp::Eq + std::hash::Hash + std::fmt::Debug,
    E: LimitedIntTrait + std::cmp::PartialEq + std::fmt::Debug + std::cmp::PartialOrd
> BoardGraph<N, E> {
    pub fn new() -> Self {
        BoardGraph(Graph::new())
    }
   
    fn get_next_tile_in_direction(&self, source_tile: TileIndex, direction: &E) -> Option<TileIndex> {
        self.edges_directed(source_tile, petgraph::Direction::Outgoing)
            .find(|edge| &edge.weight() == &direction)
            .map(|edge| edge.target())
    }
   
    pub fn knight_jumps_from(&self, source_tile: TileIndex) -> HashSet<TileIndex> {
        let mut result: HashSet<TileIndex> = HashSet::new();
        for direction in E::all_values() {
            if let Some(next_tile) = self.get_next_tile_in_direction(source_tile, &direction) {
                for next_direction in E::adjacent_values(&direction) {
                    if let Some(final_tile) = self.get_next_tile_in_direction(next_tile, &next_direction) {
                        result.insert(final_tile);
                    }
                }
            }
        }
        return result
    }

    pub fn slides_from_in_direction(&self, source_tile: TileIndex, direction: &E, limit: u32, obstructions: BitBoard) -> HashSet<TileIndex> {
        let mut result: HashSet<TileIndex> = HashSet::new();
        let mut current_tile = source_tile;
        let mut distance_traveled = 0;
        let mut hit_obstruction = false;

        while let Some(n) = self.get_next_tile_in_direction(current_tile, direction) {
            if BitBoard::new(1 << n.index()) & obstructions != BitBoard::empty() {
                hit_obstruction = true;
            } // Assuming the first obstruction is an enemy, include it in result
            result.insert(n);
            distance_traveled += 1;
            if (distance_traveled == limit) | hit_obstruction {
                break
            }
            current_tile = n;
        }
        return result
    }

    pub fn cast_slides_from(
        &self,
        source_tile: TileIndex,
        obstructions: BitBoard,
        limit: u32,
        diagonals: bool,
        orthogonals: bool
    ) -> HashSet<TileIndex> {
       
        let initital_direction = match orthogonals {
            true => 0,
            false => 1
        };
        let direction_step = match orthogonals & diagonals {
            true => 1,
            false => 2
        };

        let mut result: HashSet<TileIndex> = HashSet::new();
        for even_direction in E::all_values()
                                    .into_iter()
                                    .skip(initital_direction)
                                    .step_by(direction_step) { // TODO: Better iterator usage
            result.extend(self.slides_from_in_direction(
                source_tile,
                &even_direction,
                limit,
                obstructions
            ))
        }
        return result
    }

    pub fn knight_jumps_table(&self) -> JumpTable {
        let mut result: Vec<BitBoard> = vec![];
        for source_tile in self.0.node_indices() {
            result.push(BitBoard::from_tile_indices(self.knight_jumps_from(source_tile)))
        }
        return JumpTable::new(result)
    }

    pub fn slide_table_for_direction(&self, direction: &E) -> DirectionalSlideTable {
        let mut attack_table: Vec<HashMap<BitBoard, BitBoard>> = vec![];
        for source_tile in self.0.node_indices() {
            let unobstructed_attacks = BitBoard::from_tile_indices(
                self.slides_from_in_direction(
                    source_tile,
                    direction,
                    0,
                    BitBoard::empty()
                )
            );
            let mut attack_map = HashMap::new();
            attack_map.insert(BitBoard::empty(), unobstructed_attacks);
            for subset in CarryRippler::new(unobstructed_attacks) {
                attack_map.insert(
                    subset,
                    BitBoard::from_tile_indices(
                        self.slides_from_in_direction(
                            source_tile,
                            direction,
                            0,
                            subset
                        )
                    )
                );
            }
            attack_table.push(attack_map);
        }
        return DirectionalSlideTable::new(attack_table)
    }

    pub fn all_slide_tables(&self) -> SlideTables {
        let mut output = vec![];
        for direction in E::all_values() {
            output.push(self.slide_table_for_direction(&direction))
        }
        return SlideTables::new(output)
    }

    pub fn king_move_table(&self) -> JumpTable {
        let mut result: Vec<BitBoard> = vec![];
        for source_tile in self.0.node_indices() {
            result.push(BitBoard::from_tile_indices(self.cast_slides_from(
                source_tile,
                BitBoard::empty(),
                1,
                true,
                true
            )))
        }
        return JumpTable::new(result)
    }

    pub fn pawn_single_table(&self, color: &Color) -> JumpTable {
        let mut result: Vec<BitBoard> = vec![];

        let forward_or_backward = match color {
            Color::White => 0,
            _ => E::max_value() / 2 // This assumes max_value is even
        };

        let map = N::map_to_other::<E>();

        for source_tile in self.0.node_indices() {
            let tile = &self.0[source_tile];

            let direction = map.get(&tile.orientation).unwrap().shift_by(forward_or_backward);

            result.push(BitBoard::from_tile_indices(self.slides_from_in_direction(
                source_tile,
                &direction,
                1,
                BitBoard::empty(),
            )));
        }
        return JumpTable::new(result)
    }

    pub fn pawn_attack_table(&self, color: &Color) -> JumpTable {
        let mut result: Vec<BitBoard> = vec![];

        let forward_or_backward = match color {
            Color::White => 0,
            _ => E::max_value() / 2 // This assumes max_value is even
        };

        let map = N::map_to_other::<E>();

        for source_tile in self.0.node_indices() {
            let tile = &self.0[source_tile];

            let move_direction = map.get(&tile.orientation).unwrap().shift_by(forward_or_backward);
            let attack_directions = E::adjacent_values(&move_direction);
            let mut attacks = BitBoard::empty();

            for direction in attack_directions {
                attacks = attacks | BitBoard::from_tile_indices(self.slides_from_in_direction(
                    source_tile,
                    &direction,
                    1, 
                    BitBoard::empty()
                ))
            }
            result.push(attacks);
        }
        return JumpTable::new(result)
    }

    pub fn pawn_double_table(&self, color: &Color) -> DirectionalSlideTable {
        let mut attack_table: Vec<HashMap<BitBoard, BitBoard>> = vec![];
        
        let single_table = self.pawn_single_table(color); // A double move is two single moves

        for source_tile in self.0.node_indices() {
            let tile = &self.0[source_tile];

            let unobstructed_attacks = match &tile.pawn_start {
                Some(pawn_start_color) if pawn_start_color == color => {
                    let intermediate_tile = single_table[source_tile].lowest_one().unwrap();
                        single_table[intermediate_tile]
                },
                _ => BitBoard::empty()
            };

            let mut attack_map = HashMap::new();
            attack_map.insert(BitBoard::empty(), unobstructed_attacks);

            let occupied = single_table[source_tile];
            attack_map.insert(occupied, BitBoard::empty());
        
            attack_table.push(attack_map);
        }
        return DirectionalSlideTable::new(attack_table)
    }

    pub fn pawn_tables(&self, color: &Color) -> PawnTables {
        PawnTables::new(
            self.pawn_single_table(color),
            self.pawn_double_table(color),
            self.pawn_attack_table(color)
        )
    }

    pub fn move_tables(&self) -> MoveTables {
        MoveTables {
            king_table: self.king_move_table(),
            slide_tables: self.all_slide_tables(),
            knight_table: self.knight_jumps_table(),
            white_pawn_tables: self.pawn_tables(&Color::White),
            black_pawn_tables: self.pawn_tables(&Color::Black),
            reverse_slide_tables: self.all_slide_tables().reverse(),
            reverse_knight_table: self.knight_jumps_table().reverse(),
            reverse_white_pawn_table: self.pawn_attack_table(&Color::White).reverse(),
            reverse_black_pawn_table: self.pawn_attack_table(&Color::Black).reverse()
        }
    }
}

impl<N: LimitedIntTrait, E: LimitedIntTrait> Deref for BoardGraph<N, E> {
    type Target = Graph<Tile<N>, E>;
   
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<N: LimitedIntTrait, E: LimitedIntTrait> DerefMut for BoardGraph<N, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


create_limited_int!(TraditionalDirection, 8);
// Convention:
//    0 is the forward direction for White
//    1 is the forward-left direction, continuing counter-clockwise until 7, which is forward-right
//    Even directions are orthogonal, odd directions are diagonal
create_limited_int!(UniformTileOrientation, 1);

#[derive(Debug)]
pub struct TraditionalBoardGraph(pub BoardGraph<UniformTileOrientation, TraditionalDirection>);

impl TraditionalBoardGraph {
    pub fn new() -> Self {
        let mut board_graph = BoardGraph::new();
        for tile in 0..64 {
            board_graph.add_node(Self::new_tile(tile));
        }
        for tile_idx in board_graph.node_indices() {
            for direction in Self::get_valid_directions(tile_idx) {
                let other_idx = TileIndex::from((tile_idx.index() as i32 + Self::get_tile_index_shift(&direction)) as u32);
                board_graph.add_edge(tile_idx, other_idx, direction);
            }
        }
        return TraditionalBoardGraph(board_graph)
    }

    fn new_tile(source: i32) -> Tile<UniformTileOrientation> {
        if source / 8 == 1 {
            return Tile { orientation: UniformTileOrientation(0), pawn_start: Some(Color::White) }
        } else if source / 8 == 6 {
            return Tile { orientation: UniformTileOrientation(0), pawn_start: Some(Color::Black) }
        } else {
            return Tile { orientation: UniformTileOrientation(0), pawn_start: None }
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
    
    pub fn display(&self, position: &Position, selected_tile: TileIndex, move_tables: &MoveTables, showing_indices: bool) -> String {
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
        let selected_white = white_pieces.get_piece_at(selected_tile);
        let selected_black = black_pieces.get_piece_at(selected_tile);
        let selected_piece = selected_white.or(selected_black);
        let allied_occupied = match black_pieces.get_piece_at(selected_tile) {
            Some(_t) => black_pieces.occupied,
            _ => white_pieces.occupied
        };
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

        let move_options = match selected_piece {
            Some(Piece::Pawn) => BitBoard::empty(),
            None => BitBoard::empty(),
            _ => { // All non-Pawn PieceTypes
                let display_idx = 631 - 84 * (selected_tile.index() / 8) + 5 * (selected_tile.index() % 8);
                output[display_idx + 1] = '?';
                move_tables.query_piece(&selected_piece.unwrap(), selected_tile, occupied) & !allied_occupied
            }
        };

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


create_limited_int!(HexagonalDirection, 12);

#[derive(Debug)]
pub struct HexagonalBoardGraph(pub BoardGraph<UniformTileOrientation, HexagonalDirection>);

impl HexagonalBoardGraph {
    pub fn new() -> Self {
        let mut board_graph = BoardGraph::new();
        for tile in 0..91 {
            board_graph.add_node(Self::new_tile(tile));
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

    fn new_tile(source: i32) -> Tile<UniformTileOrientation> {
        let pawn_start = match source {
            4 | 10 | 17 | 25 | 30..=34 => Some(Color::White),
            56..=60 | 65 | 73 | 80 | 86 => Some(Color::Black),
            _ => None
        };
        return Tile { orientation: UniformTileOrientation(0), pawn_start }
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


#[cfg(test)]
mod tests {
    use super::*;

    fn test_traditional_board() -> TraditionalBoardGraph {
        return TraditionalBoardGraph::new();
    }

    #[test]
    fn test_get_next_tile_in_direction_returns_tile() {
        let board = test_traditional_board();
        assert_eq!(
            board.0.get_next_tile_in_direction(TileIndex::new(0), &TraditionalDirection(0)).unwrap(),
            TileIndex::new(8)
        );
    }

    #[test]
    fn test_get_next_tile_in_direction_returns_none() {
        let board = test_traditional_board();
        assert_eq!(
            board.0.get_next_tile_in_direction(TileIndex::new(0), &TraditionalDirection(2)),
            None
        )
    }

    #[test]
    fn test_knight_move_from() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(27);
        assert_eq!(
            board.0.knight_jumps_from(source_tile),
            HashSet::from_iter([
                TileIndex::new(27 + 10),
                TileIndex::new(27 - 10),
                TileIndex::new(27 + 6),
                TileIndex::new(27 - 6),
                TileIndex::new(27 + 17),
                TileIndex::new(27 - 17),
                TileIndex::new(27 + 15),
                TileIndex::new(27 - 15)
            ])
        )
    }

    #[test]
    fn test_slide_move_from_no_limit_no_obstructions() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(1);
        assert_eq!(
            board.0.slides_from_in_direction(source_tile, &TraditionalDirection(6), 0, BitBoard::empty()),
            HashSet::from_iter([
                TileIndex::new(2),
                TileIndex::new(3),
                TileIndex::new(4),
                TileIndex::new(5),
                TileIndex::new(6),
                TileIndex::new(7),
            ])
        )
    }
    #[test]
    fn test_slide_move_with_limit() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(1);
        assert_eq!(
            board.0.slides_from_in_direction(source_tile, &TraditionalDirection(6), 1, BitBoard::empty()),
            HashSet::from_iter([TileIndex::new(2)])
        )
    }

    #[test]
    fn test_slide_move_with_obstructions() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(1);
        let obstructions = BitBoard::new(16);
        assert_eq!(
            board.0.slides_from_in_direction(source_tile, &TraditionalDirection(6), 0, obstructions),
            HashSet::from_iter([
                TileIndex::new(2),
                TileIndex::new(3),
                TileIndex::new(4),
            ])
        )
    }

    #[test]
    fn test_diagonal_slides_unobstructed() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(27);
        assert_eq!(
            board.0.cast_slides_from(source_tile, BitBoard::empty(), 0, true, false),
            HashSet::from_iter([    
                TileIndex::new(0),
                TileIndex::new(9),
                TileIndex::new(18),
                TileIndex::new(36),
                TileIndex::new(45),
                TileIndex::new(54),
                TileIndex::new(63),
                TileIndex::new(34),
                TileIndex::new(41),
                TileIndex::new(48),
                TileIndex::new(20),
                TileIndex::new(13),
                TileIndex::new(6)
            ])
        )
    }

    #[test]
    fn test_diagonal_slides_obstructed() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(27);
        let occupied = BitBoard::from_ints(vec![36, 34, 20]);
        assert_eq!(
            board.0.cast_slides_from(source_tile, occupied, 0, true, false),
            HashSet::from_iter([    
                TileIndex::new(0),
                TileIndex::new(9),
                TileIndex::new(18),
                TileIndex::new(36),
                TileIndex::new(34),
                TileIndex::new(20)
            ])
        )
    }

    #[test]
    fn test_orthogonal_slides_unobstructed() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(27);
        assert_eq!(
            board.0.cast_slides_from(source_tile, BitBoard::empty(), 0, false, true),
            HashSet::from_iter([    
                TileIndex::new(24),
                TileIndex::new(25),
                TileIndex::new(26),
                TileIndex::new(28),
                TileIndex::new(29),
                TileIndex::new(30),
                TileIndex::new(31),
                TileIndex::new(3),
                TileIndex::new(19),
                TileIndex::new(11),
                TileIndex::new(35),
                TileIndex::new(43),
                TileIndex::new(51),
                TileIndex::new(59)
            ])
        )
    }

    #[test]
    fn test_both_slides_unobstructed() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(27);
        assert_eq!(
            board.0.cast_slides_from(source_tile, BitBoard::empty(), 0, true, true),
            HashSet::from_iter([    
                TileIndex::new(24),
                TileIndex::new(25),
                TileIndex::new(26),
                TileIndex::new(28),
                TileIndex::new(29),
                TileIndex::new(30),
                TileIndex::new(31),
                TileIndex::new(3),
                TileIndex::new(19),
                TileIndex::new(11),
                TileIndex::new(35),
                TileIndex::new(43),
                TileIndex::new(51),
                TileIndex::new(59),
                TileIndex::new(0),
                TileIndex::new(9),
                TileIndex::new(18),
                TileIndex::new(36),
                TileIndex::new(45),
                TileIndex::new(54),
                TileIndex::new(63),
                TileIndex::new(34),
                TileIndex::new(41),
                TileIndex::new(48),
                TileIndex::new(20),
                TileIndex::new(13),
                TileIndex::new(6)
            ])
        )
    }

    #[test]
    fn test_cast_slides_with_limit() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(27);
        assert_eq!(
            board.0.cast_slides_from(source_tile, BitBoard::empty(), 1, true, true),
            HashSet::from_iter([
                TileIndex::new(36),
                TileIndex::new(35),
                TileIndex::new(34),
                TileIndex::new(28),
                TileIndex::new(26),
                TileIndex::new(20),
                TileIndex::new(19),
                TileIndex::new(18),
            ])
        )
    }
}
