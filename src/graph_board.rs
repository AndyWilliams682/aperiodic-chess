use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashSet, HashMap};
use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::bit_board::{BitBoard, CarryRippler, BitBoardTiles};
use crate::create_limited_int;
use crate::limited_int::LimitedIntTrait;
use crate::move_generator::MoveTables;

pub type TileIndex = NodeIndex;


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    White,
    Black
}

impl Color {
    pub fn as_idx(&self) -> usize {
        return match self {
            Color::White => 0,
            Color::Black => 1
        }
    }

    pub fn opponent(&self) -> Self {
        return match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
}

#[derive(Debug)]
pub struct Tile<N: LimitedIntTrait> {
    orientation: N,
    pawn_start: Option<Color>
}

#[derive(Debug, PartialEq, Clone)]
pub struct JumpTable(pub Vec<BitBoard>);

impl JumpTable {
    pub fn new(val: Vec<BitBoard>) -> Self {
        Self(val)
    }

    pub fn empty(num_tiles: usize) -> Self {
        Self::new(vec![BitBoard::empty(); num_tiles])
    }

    pub fn num_tiles(&self) -> usize {
        return self.0.len()
    }

    pub fn reverse(&self) -> Self {
        let num_tiles = self.num_tiles();
        let mut output = Self::empty(num_tiles);

        let mut source_tile = 0;
        for source_tile_moves in &self.0 {
            for to_tile in BitBoardTiles::new(*source_tile_moves) {
                output[to_tile].flip_bit_at_tile_index(TileIndex::new(source_tile));
            }
            source_tile += 1;
        }
        output
    }
}

impl Index<TileIndex> for JumpTable {
    type Output = BitBoard;
   
    fn index(&self, index: TileIndex) -> &Self::Output {
        &self.0[index.index()]
    }
}

impl IndexMut<TileIndex> for JumpTable {
    fn index_mut(&mut self, index: TileIndex) -> &mut Self::Output {
        &mut self.0[index.index()]
    }
}

#[derive(Debug, Clone)]
pub struct DirectionalSlideTable(Vec<HashMap<BitBoard, BitBoard>>);

impl DirectionalSlideTable {
    pub fn new(val: Vec<HashMap<BitBoard, BitBoard>>) -> Self {
        return Self(val)
    }

    pub fn reverse(&self) -> JumpTable {
        // Returning a JumpTable because this does not care about blockers (will be handled later)
        let num_tiles = self.0.len();
        let mut output = JumpTable::empty(num_tiles);
       
        let mut source_tile = 0;
        for source_tile_moves in &self.0 {
            let unblocked_moves = source_tile_moves.get(&BitBoard::empty()).unwrap();
            for to_tile in BitBoardTiles::new(*unblocked_moves) {
                output[to_tile].flip_bit_at_tile_index(TileIndex::new(source_tile));
            }
            source_tile += 1;
        }
        output
    }
}

impl Index<TileIndex> for DirectionalSlideTable {
    type Output = HashMap<BitBoard, BitBoard>;
   
    fn index(&self, index: TileIndex) -> &Self::Output {
        &self.0[index.index()]
    }
}

#[derive(Debug, Clone)]
pub struct SlideTables(Vec<DirectionalSlideTable>);

impl SlideTables {
    fn new(val: Vec<DirectionalSlideTable>) -> Self {
        return Self(val)
    }
   
    pub fn query(&self, source_tile: &TileIndex, occupied: &BitBoard, orthogonals: bool, diagonals: bool) -> BitBoard {
        let mut result = BitBoard::empty();
        let initial_direction = match orthogonals {
            true => 0,
            false => 1
        };
        let direction_step = match orthogonals & diagonals {
            true => 1,
            false => 2
        };
        for direction in (initial_direction..self.0.len()).step_by(direction_step) {
            let unblocked_attacks = *self[direction][*source_tile].get(&BitBoard::empty()).unwrap();
            let blocked_attacks = *self[direction][*source_tile].get(&(*occupied & unblocked_attacks)).unwrap(); 
            result = result | blocked_attacks;
        }
        result
    }

    pub fn reverse(&self) -> Vec<JumpTable> {
        let mut output = vec![];
        for directional_table in &self.0 {
            output.push(directional_table.reverse())
        }
        output
    }
}

impl Index<usize> for SlideTables {
    type Output = DirectionalSlideTable;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[derive(Debug, Clone)]
pub struct PawnTables {
    pub single_table: JumpTable,
    pub double_table: DirectionalSlideTable,
    pub attack_table: JumpTable,
}

impl PawnTables {
    pub fn new(single_table: JumpTable, double_table: DirectionalSlideTable, attack_table: JumpTable) -> Self {
        Self { single_table, double_table, attack_table }
    }
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

    pub fn pawn_single_table(&self, color: Color) -> JumpTable {
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

    pub fn pawn_attack_table(&self, color: Color) -> JumpTable {
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

    pub fn pawn_double_table(&self, color: Color) -> DirectionalSlideTable {
        let mut attack_table: Vec<HashMap<BitBoard, BitBoard>> = vec![];
        
        let single_table = self.pawn_single_table(color); // A double move is two single moves

        for source_tile in self.0.node_indices() {
            let tile = &self.0[source_tile];

            let unobstructed_attacks = match &tile.pawn_start {
                Some(pawn_start_color) if pawn_start_color == &color => {
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

    pub fn pawn_tables(&self, color: Color) -> PawnTables {
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
            white_pawn_tables: self.pawn_tables(Color::White),
            black_pawn_tables: self.pawn_tables(Color::Black),
            reverse_slide_tables: self.all_slide_tables().reverse(),
            reverse_knight_table: self.knight_jumps_table().reverse(),
            reverse_white_pawn_table: self.pawn_attack_table(Color::White).reverse(),
            reverse_black_pawn_table: self.pawn_attack_table(Color::Black).reverse()
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

    fn traditional_slide_tables() -> SlideTables {
        return test_traditional_board().0.all_slide_tables()
    }

    fn test_hexagonal_board() -> HexagonalBoardGraph {
        return HexagonalBoardGraph::new();
    }

    fn hexagonal_slide_tables() -> SlideTables {
        return test_hexagonal_board().0.all_slide_tables()
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

    #[test]
    fn test_knight_table() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(63);
        assert_eq!(
            board.0.knight_jumps_table()[source_tile],
            BitBoard::from_ints(vec![53, 46])
        )
    }

    #[test]
    fn test_slide_table_for_direction() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(0);
        assert_eq!(
            *board.0.slide_table_for_direction(&TraditionalDirection(0))[source_tile].get(&BitBoard::new(65536)).unwrap(),
            BitBoard::from_ints(vec![8, 16])
        )
    }

    #[test]
    fn test_diagonal_slide_table() {
        let source_tile = TileIndex::new(63);
        assert_eq!(
            traditional_slide_tables().query(&source_tile, &BitBoard::empty(), false, true),
            BitBoard::from_ints(vec![54, 45, 36, 27, 18, 9, 0])
        );
        let occupied = BitBoard::from_ints(vec![45]);
        assert_eq!(
            traditional_slide_tables().query(&source_tile, &occupied, false, true),
            BitBoard::from_ints(vec![54, 45])
        );
    }

    #[test]
    fn test_orthogonal_table() {
        let source_tile = TileIndex::new(63);
        assert_eq!(
            traditional_slide_tables().query(&source_tile, &BitBoard::empty(), true, false),
            BitBoard::from_ints(vec![62, 61, 60, 59, 58, 57, 56, 55, 47, 39, 31, 23, 15, 7])
        );
        let occupied = BitBoard::from_ints(vec![62]);
        assert_eq!(
            traditional_slide_tables().query(&source_tile, &occupied, true, false),
            BitBoard::from_ints(vec![62, 55, 47, 39, 31, 23, 15, 7])
        )
    }

    #[test]
    fn test_king_table() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(63);
        assert_eq!(
            board.0.king_move_table()[source_tile],
            BitBoard::from_ints(vec![62, 55, 54])
        )
    }

    #[test]
    fn test_pawn_double_table_forward() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(8);
        assert_eq!(
            *board.0.pawn_double_table(Color::White)[source_tile].get(&BitBoard::empty()).unwrap(),
            BitBoard::from_ints(vec![24])
        );
        assert_eq!(
            *board.0.pawn_double_table(Color::White)[source_tile].get(&BitBoard::from_ints(vec![16])).unwrap(),
            BitBoard::empty()
        );
    }

    #[test]
    fn test_pawn_double_table_backward() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(48);
        assert_eq!(
            *board.0.pawn_double_table(Color::Black)[source_tile].get(&BitBoard::empty()).unwrap(),
            BitBoard::from_ints(vec![32])
        );
        assert_eq!(
            *board.0.pawn_double_table(Color::Black)[source_tile].get(&BitBoard::from_ints(vec![40])).unwrap(),
            BitBoard::empty()
        );
    }

    #[test]
    fn test_pawn_single_table() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(48);
        assert_eq!(
            board.0.pawn_single_table(Color::White)[source_tile],
            BitBoard::from_ints(vec![56])
        );
        assert_eq!(
            board.0.pawn_single_table(Color::White)[TileIndex::new(56)],
            BitBoard::empty()
        );
    }

    #[test]
    fn test_pawn_attack_table() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(49);
        assert_eq!(
            board.0.pawn_attack_table(Color::White)[source_tile],
            BitBoard::from_ints(vec![56, 58])
        )
    }

    #[test]
    fn test_pawn_attack_table_at_edge() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(48);
        assert_eq!(
            board.0.pawn_attack_table(Color::Black)[source_tile],
            BitBoard::from_ints(vec![41])
        )
    }

    #[test]
    fn test_hex_knight_table() {
        let board = test_hexagonal_board();
        let source_tile = TileIndex::new(0);
        assert_eq!(
            board.0.knight_jumps_table()[source_tile],
            BitBoard::from_ints(vec![9, 16, 22, 23])
        )
    }

    #[test]
    fn test_hex_queen_table() {
        let source_tile = TileIndex::new(0);
        assert_eq!(
            hexagonal_slide_tables().query(&source_tile, &BitBoard::empty(), true, true),
            BitBoard::from_ints(vec![
                1, 2, 3, 4, 5, // Direction 10
                6, 13, 21, 30, 40, // 2
                7, 15, 24, 34, 45, 56, 66, 75, 83, 90, // 0
                14, 32, 53, 71, 85, // 1
                8, 17, 27, 38, 50 // 11
            ])
        );
    }

    #[test]
    fn test_hex_king_table() {
        let board = test_hexagonal_board();
        let source_tile = TileIndex::new(0);
        assert_eq!(
            board.0.king_move_table()[source_tile],
            BitBoard::from_ints(vec![1, 6, 7, 8, 14])
        )
    }

    #[test]
    fn test_hex_pawn_double_table_backward() {
        let board = test_hexagonal_board();
        let source_tile = TileIndex::new(56);
        assert_eq!(
            *board.0.pawn_double_table(Color::Black)[source_tile].get(&BitBoard::empty()).unwrap(),
            BitBoard::from_ints(vec![34])
        )
    }

    #[test]
    fn test_jump_table_empty_and_len() {
        let test = JumpTable::empty(64);
        assert_eq!(
            test.num_tiles(),
            64
        )
    }

    #[test]
    fn test_reverse_knight_table() {
        let board = test_traditional_board();
        let knight_table = board.0.knight_jumps_table();
        let output = knight_table.reverse();
        // For traditional/hexagonal boards, these are equal
        assert_eq!(
            output,
            knight_table
        )
    }
   
    #[test]
    fn test_reverse_pawn_table() {
        let board = test_traditional_board();
        // For traditional/hexagonal boards, rev(White)=Black
        assert_eq!(
            board.0.pawn_attack_table(Color::White).reverse(),
            board.0.pawn_attack_table(Color::Black)
        )
    }

    #[test]
    fn test_reverse_directional_slide_table() {
        let board = test_traditional_board();
        let directional_slide_table = board.0.slide_table_for_direction(
            &TraditionalDirection(0)
        );
        assert_eq!(
            directional_slide_table.reverse()[TileIndex::new(56)],
            BitBoard::from_ints(vec![0, 8, 16, 24, 32, 40, 48])
        );
        assert_eq!(
            directional_slide_table.reverse()[TileIndex::new(0)],
            BitBoard::empty()
        )
    }
}
