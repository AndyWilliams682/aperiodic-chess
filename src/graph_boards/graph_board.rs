use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashSet, HashMap};
use std::ops::{Deref, DerefMut};

use crate::bit_board::{BitBoard, CarryRippler};
use crate::limited_int::LimitedInt;
use crate::move_generator::MoveTables;
use crate::piece_set::Color;
use crate::movement_tables::{JumpTable, DirectionalSlideTable, SlideTables, PawnTables};


pub type TileIndex = NodeIndex;

#[derive(Debug, Clone, Copy)]
pub struct Tile<const N: u8> {
    pub orientation: LimitedInt<N>,
    pub pawn_start: Option<Color>
}

// Generic graph that uses LimitedIntTrait for the edges
#[derive(Debug)]
pub struct GraphBoard<const N: u8, const E: u8>(Graph<Tile<N>, LimitedInt<E>>);

impl <const N: u8, const E: u8> GraphBoard<N, E> {
    pub fn new() -> Self {
        GraphBoard(Graph::new())
    }
   
    fn get_next_tile_in_direction(&self, source_tile: TileIndex, direction: &LimitedInt<E>) -> Option<TileIndex> {
        self.edges_directed(source_tile, petgraph::Direction::Outgoing)
            .find(|edge| &edge.weight() == &direction)
            .map(|edge| edge.target())
    }
   
    pub fn knight_jumps_from(&self, source_tile: TileIndex) -> HashSet<TileIndex> {
        let mut result: HashSet<TileIndex> = HashSet::new();
        for direction in LimitedInt::<E>::all_values() {
            if let Some(next_tile) = self.get_next_tile_in_direction(source_tile, &direction) {
                for next_direction in LimitedInt::<E>::adjacent_values(&direction) {
                    if let Some(final_tile) = self.get_next_tile_in_direction(next_tile, &next_direction) {
                        result.insert(final_tile);
                    }
                }
            }
        }
        return result
    }

    pub fn slides_from_in_direction(&self, source_tile: TileIndex, direction: &LimitedInt<E>, limit: u32, obstructions: BitBoard) -> HashSet<TileIndex> {
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
        for even_direction in LimitedInt::<E>::all_values()
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

    pub fn slide_table_for_direction(&self, direction: &LimitedInt<E>) -> DirectionalSlideTable {
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
        for direction in LimitedInt::<E>::all_values() {
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
            _ => E / 2 // This assumes max_value is even
        };

        let map = LimitedInt::<N>::map_to_other::<E>();

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
            _ => E / 2 // This assumes max_value is even
        };

        let map = LimitedInt::<N>::map_to_other::<E>();

        for source_tile in self.0.node_indices() {
            let tile = &self.0[source_tile];

            let move_direction = map.get(&tile.orientation).unwrap().shift_by(forward_or_backward);
            let attack_directions = LimitedInt::<E>::adjacent_values(&move_direction);
            let mut attacks = BitBoard::empty();

            for direction in attack_directions {
                attacks |= BitBoard::from_tile_indices(self.slides_from_in_direction(
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

impl<const N: u8, const E: u8> Deref for GraphBoard<N, E> {
    type Target = Graph<Tile<N>, LimitedInt<E>>;
   
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: u8, const E: u8> DerefMut for GraphBoard<N, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


pub type UniformTileOrientation = LimitedInt<1>;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_boards::traditional_board::{TraditionalBoardGraph, TraditionalDirection};

    fn test_traditional_board() -> TraditionalBoardGraph {
        return TraditionalBoardGraph::new();
    }

    #[test]
    fn test_get_next_tile_in_direction_returns_tile() {
        let board = test_traditional_board();
        assert_eq!(
            board.0.get_next_tile_in_direction(TileIndex::new(0), &TraditionalDirection::new(0)).unwrap(),
            TileIndex::new(8)
        );
    }

    #[test]
    fn test_get_next_tile_in_direction_returns_none() {
        let board = test_traditional_board();
        assert_eq!(
            board.0.get_next_tile_in_direction(TileIndex::new(0), &TraditionalDirection::new(2)),
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
            board.0.slides_from_in_direction(source_tile, &TraditionalDirection::new(6), 0, BitBoard::empty()),
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
            board.0.slides_from_in_direction(source_tile, &TraditionalDirection::new(6), 1, BitBoard::empty()),
            HashSet::from_iter([TileIndex::new(2)])
        )
    }

    #[test]
    fn test_slide_move_with_obstructions() {
        let board = test_traditional_board();
        let source_tile = TileIndex::new(1);
        let obstructions = BitBoard::new(16);
        assert_eq!(
            board.0.slides_from_in_direction(source_tile, &TraditionalDirection::new(6), 0, obstructions),
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
