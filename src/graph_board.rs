use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashSet, HashMap};
use std::ops::{Deref, DerefMut, Index};

use crate::bit_board::{BitBoard, CarryRippler};
use crate::create_limited_int;
use crate::limited_int::LimitedIntTrait;


#[derive(Debug, PartialEq)]
enum Color {
    White,
    Black
}

#[derive(Debug)]
pub struct Tile<N: LimitedIntTrait> {
    orientation: N,
    pawn_start: Option<Color>
}

#[derive(Debug)]
pub struct JumpTable(Vec<BitBoard>);

impl JumpTable {
    pub fn new(val: Vec<BitBoard>) -> Self {
        Self(val)
    }
}

impl Index<NodeIndex> for JumpTable {
    type Output = BitBoard;
   
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.0[index.index()]
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
   
    fn get_next_node_in_direction(&self, source_node: NodeIndex, direction: &E) -> Option<NodeIndex> {
        self.edges_directed(source_node, petgraph::Direction::Outgoing)
            .find(|edge| &edge.weight() == &direction)
            .map(|edge| edge.target())
    }
   
    pub fn knight_jumps_from(&self, source_node: NodeIndex) -> HashSet<NodeIndex> {
        let mut result: HashSet<NodeIndex> = HashSet::new();
        for direction in E::all_values() {
            if let Some(next_node) = self.get_next_node_in_direction(source_node, &direction) {
                for next_direction in E::adjacent_values(&direction) {
                    if let Some(final_node) = self.get_next_node_in_direction(next_node, &next_direction) {
                        result.insert(final_node);
                    }
                }
            }
        }
        return result
    }

    pub fn slides_from_in_direction(&self, source_node: NodeIndex, direction: &E, limit: u32, obstructions: BitBoard) -> HashSet<NodeIndex> {
        let mut result: HashSet<NodeIndex> = HashSet::new();
        let mut current_node = source_node;
        let mut distance_traveled = 0;

        while let Some(n) = self.get_next_node_in_direction(current_node, direction) {
            if BitBoard::new(1 << n.index()) & obstructions != BitBoard::empty() {
                break
            }
            result.insert(n);
            distance_traveled += 1;
            if distance_traveled == limit {
                break
            }
            current_node = n;
        }
        return result
    }

    pub fn cast_slides_from(
        &self,
        source_node: NodeIndex,
        obstructions: BitBoard,
        limit: u32,
        diagonals: bool,
        orthogonals: bool
    ) -> HashSet<NodeIndex> {
       
        let initital_direction = match orthogonals {
            true => 0,
            false => 1
        };
        let direction_step = match orthogonals & diagonals {
            true => 1,
            false => 2
        };

        let mut result: HashSet<NodeIndex> = HashSet::new();
        for even_direction in E::all_values()
                                    .into_iter()
                                    .skip(initital_direction)
                                    .step_by(direction_step) { // TODO: Better iterator usage
            result.extend(self.slides_from_in_direction(
                source_node,
                &even_direction,
                limit,
                obstructions
            ))
        }
        return result
    }

    pub fn knight_jumps_table(&self) -> JumpTable {
        let mut result: Vec<BitBoard> = vec![];
        for source_node in self.0.node_indices() {
            result.push(BitBoard::from_node_indices(self.knight_jumps_from(source_node)))
        }
        return JumpTable::new(result)
    }

    pub fn slide_table_for_direction(&self, direction: &E) -> Vec<HashMap<BitBoard, BitBoard>> {
        let mut attack_table: Vec<HashMap<BitBoard, BitBoard>> = vec![];
        for source_node in self.0.node_indices() {
            let unobstructed_attacks = BitBoard::from_node_indices(
                self.slides_from_in_direction(
                    source_node,
                    direction,
                    0,
                    BitBoard::empty()
                )
            );
            let mut attack_map = HashMap::new();
            attack_map.insert(BitBoard::new(0), unobstructed_attacks);
            for subset in CarryRippler::new(unobstructed_attacks) {
                attack_map.insert(
                    subset,
                    BitBoard::from_node_indices(
                        self.slides_from_in_direction(
                            source_node,
                            direction,
                            0,
                            subset
                        )
                    )
                );
            }
            attack_table.push(attack_map);
        }
        return attack_table
    }

    pub fn all_slide_tables(&self) -> Vec<Vec<HashMap<BitBoard, BitBoard>>> {
        let mut output = vec![];
        for direction in E::all_values() {
            output.push(self.slide_table_for_direction(&direction))
        }
        return output
    }

    pub fn king_move_table(&self) -> JumpTable {
        let mut result: Vec<BitBoard> = vec![];
        for source_node in self.0.node_indices() {
            result.push(BitBoard::from_node_indices(self.cast_slides_from(
                source_node,
                BitBoard::empty(),
                1,
                true,
                true
            )))
        }
        return JumpTable::new(result)
    }

    pub fn pawn_move_table(&self, color: Color) -> JumpTable {
        let mut result: Vec<BitBoard> = vec![];

        let forward_or_backward = match color {
            Color::White => 0,
            _ => E::max_value() / 2 // This assumes max_value is even
        };

        let map = N::map_to_other::<E>();

        for source_node in self.0.node_indices() {
            let tile = &self.0[source_node];

            let move_limit = match &tile.pawn_start {
                Some(pawn_start_color) if pawn_start_color == &color => 2,
                _ => 1
            };

            let direction = map.get(&tile.orientation).unwrap().shift_by(forward_or_backward);

            result.push(BitBoard::from_node_indices(self.slides_from_in_direction(
                source_node,
                &direction,
                move_limit,
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

        for source_node in self.0.node_indices() {
            let tile = &self.0[source_node];

            let move_direction = map.get(&tile.orientation).unwrap().shift_by(forward_or_backward);
            let attack_directions = E::adjacent_values(&move_direction);
            let mut attacks = BitBoard::empty();

            for direction in attack_directions {
                attacks = attacks | BitBoard::from_node_indices(self.slides_from_in_direction(
                    source_node,
                    &direction,
                    1, 
                    BitBoard::empty()
                ))
            }
            result.push(attacks);
        }
        return JumpTable::new(result)
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
pub struct TraditionalBoardGraph(BoardGraph<UniformTileOrientation, TraditionalDirection>);

impl TraditionalBoardGraph {
    pub fn empty() -> Self {
        let mut board_graph = BoardGraph::new();
        for node in 0..64 {
            board_graph.add_node(Self::new_tile(node));
        }
        for node_idx in board_graph.node_indices() {
            for direction in Self::get_valid_directions(node_idx) {
                let other_idx = NodeIndex::from((node_idx.index() as i32 + Self::get_node_index_shift(&direction)) as u32);
                board_graph.add_edge(node_idx, other_idx, direction);
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
    fn get_valid_directions(source: NodeIndex) -> Vec<TraditionalDirection> {
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
    fn get_node_index_shift(direction: &TraditionalDirection) -> i32 {
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
    pub fn empty() -> Self {
        let mut board_graph = BoardGraph::new();
        for node in 0..91 {
            board_graph.add_node(Self::new_tile(node));
        }
        for node_idx in board_graph.node_indices() {
            for direction in Self::get_valid_directions(node_idx) {
                let other_idx = NodeIndex::from((node_idx.index() as i32 + Self::get_node_index_shift(node_idx, &direction)) as u32);
                board_graph.add_edge(node_idx, other_idx, direction);
            }
        }
        return HexagonalBoardGraph(board_graph)
    }

    fn row_length(n: NodeIndex) -> i32 {
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
   
    fn get_valid_directions(source: NodeIndex) -> Vec<HexagonalDirection> {
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
   
    fn get_node_index_shift(source: NodeIndex, direction: &HexagonalDirection) -> i32 {
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
        return TraditionalBoardGraph::empty();
    }

    fn traditional_slide_tables() -> Vec<Vec<HashMap<BitBoard, BitBoard>>> {
        return test_traditional_board().0.all_slide_tables()
    }

    fn test_hexagonal_board() -> HexagonalBoardGraph {
        return HexagonalBoardGraph::empty();
    }

    fn hexagonal_slide_tables() -> Vec<Vec<HashMap<BitBoard, BitBoard>>> {
        return test_hexagonal_board().0.all_slide_tables()
    }

    #[test]
    fn test_get_next_node_in_direction_returns_node() {
        let board = test_traditional_board();
        assert_eq!(
            board.0.get_next_node_in_direction(NodeIndex::new(0), &TraditionalDirection(0)).unwrap(),
            NodeIndex::new(8)
        );
    }

    #[test]
    fn test_get_next_node_in_direction_returns_none() {
        let board = test_traditional_board();
        assert_eq!(
            board.0.get_next_node_in_direction(NodeIndex::new(0), &TraditionalDirection(2)),
            None
        )
    }

    #[test]
    fn test_knight_move_from() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(27);
        assert_eq!(
            board.0.knight_jumps_from(source_node),
            HashSet::from_iter([
                NodeIndex::new(27 + 10),
                NodeIndex::new(27 - 10),
                NodeIndex::new(27 + 6),
                NodeIndex::new(27 - 6),
                NodeIndex::new(27 + 17),
                NodeIndex::new(27 - 17),
                NodeIndex::new(27 + 15),
                NodeIndex::new(27 - 15)
            ])
        )
    }

    #[test]
    fn test_slide_move_from_no_limit_no_obstructions() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(1);
        assert_eq!(
            board.0.slides_from_in_direction(source_node, &TraditionalDirection(6), 0, BitBoard::empty()),
            HashSet::from_iter([
                NodeIndex::new(2),
                NodeIndex::new(3),
                NodeIndex::new(4),
                NodeIndex::new(5),
                NodeIndex::new(6),
                NodeIndex::new(7),
            ])
        )
    }
    #[test]
    fn test_slide_move_with_limit() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(1);
        assert_eq!(
            board.0.slides_from_in_direction(source_node, &TraditionalDirection(6), 1, BitBoard::empty()),
            HashSet::from_iter([NodeIndex::new(2)])
        )
    }

    #[test]
    fn test_slide_move_with_obstructions() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(1);
        let obstructions = BitBoard::new(32);
        assert_eq!(
            board.0.slides_from_in_direction(source_node, &TraditionalDirection(6), 0, obstructions),
            HashSet::from_iter([
                NodeIndex::new(2),
                NodeIndex::new(3),
                NodeIndex::new(4)
            ])
        )
    }

    #[test]
    fn test_diagonal_slides_unobstructed() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(27);
        assert_eq!(
            board.0.cast_slides_from(source_node, BitBoard::empty(), 0, true, false),
            HashSet::from_iter([    
                NodeIndex::new(0),
                NodeIndex::new(9),
                NodeIndex::new(18),
                NodeIndex::new(36),
                NodeIndex::new(45),
                NodeIndex::new(54),
                NodeIndex::new(63),
                NodeIndex::new(34),
                NodeIndex::new(41),
                NodeIndex::new(48),
                NodeIndex::new(20),
                NodeIndex::new(13),
                NodeIndex::new(6)
            ])
        )
    }

    #[test]
    fn test_diagonal_slides_obstructed() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(27);
        let blockers = BitBoard::from_ints(vec![36, 34, 20]);
        assert_eq!(
            board.0.cast_slides_from(source_node, blockers, 0, true, false),
            HashSet::from_iter([    
                NodeIndex::new(0),
                NodeIndex::new(9),
                NodeIndex::new(18),
            ])
        )
    }

    #[test]
    fn test_orthogonal_slides_unobstructed() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(27);
        assert_eq!(
            board.0.cast_slides_from(source_node, BitBoard::empty(), 0, false, true),
            HashSet::from_iter([    
                NodeIndex::new(24),
                NodeIndex::new(25),
                NodeIndex::new(26),
                NodeIndex::new(28),
                NodeIndex::new(29),
                NodeIndex::new(30),
                NodeIndex::new(31),
                NodeIndex::new(3),
                NodeIndex::new(19),
                NodeIndex::new(11),
                NodeIndex::new(35),
                NodeIndex::new(43),
                NodeIndex::new(51),
                NodeIndex::new(59)
            ])
        )
    }

    #[test]
    fn test_both_slides_unobstructed() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(27);
        assert_eq!(
            board.0.cast_slides_from(source_node, BitBoard::empty(), 0, true, true),
            HashSet::from_iter([    
                NodeIndex::new(24),
                NodeIndex::new(25),
                NodeIndex::new(26),
                NodeIndex::new(28),
                NodeIndex::new(29),
                NodeIndex::new(30),
                NodeIndex::new(31),
                NodeIndex::new(3),
                NodeIndex::new(19),
                NodeIndex::new(11),
                NodeIndex::new(35),
                NodeIndex::new(43),
                NodeIndex::new(51),
                NodeIndex::new(59),
                NodeIndex::new(0),
                NodeIndex::new(9),
                NodeIndex::new(18),
                NodeIndex::new(36),
                NodeIndex::new(45),
                NodeIndex::new(54),
                NodeIndex::new(63),
                NodeIndex::new(34),
                NodeIndex::new(41),
                NodeIndex::new(48),
                NodeIndex::new(20),
                NodeIndex::new(13),
                NodeIndex::new(6)
            ])
        )
    }

    #[test]
    fn test_cast_slides_with_limit() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(27);
        assert_eq!(
            board.0.cast_slides_from(source_node, BitBoard::empty(), 1, true, true),
            HashSet::from_iter([
                NodeIndex::new(36),
                NodeIndex::new(35),
                NodeIndex::new(34),
                NodeIndex::new(28),
                NodeIndex::new(26),
                NodeIndex::new(20),
                NodeIndex::new(19),
                NodeIndex::new(18),
            ])
        )
    }

    #[test]
    fn test_knight_table() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(63);
        assert_eq!(
            board.0.knight_jumps_table()[source_node], // Only testing last node
            BitBoard::from_ints(vec![53, 46])
        )
    }

    #[test]
    fn test_slide_table_for_direction() {
        let board = test_traditional_board();
        assert_eq!(
            *board.0.slide_table_for_direction(&TraditionalDirection(0))[0].get(&BitBoard::new(65536)).unwrap(),
            BitBoard::from_ints(vec![8])
        )
    }

    #[test]
    fn test_diagonal_slide_table() {
        let diag_table_3 = &traditional_slide_tables()[3];
        assert_eq!(
            *diag_table_3[63].get(&BitBoard::empty()).unwrap(), // Only testing last node
            BitBoard::from_ints(vec![54, 45, 36, 27, 18, 9, 0])
        );
        let blockers = BitBoard::from_ints(vec![45]);
        assert_eq!(
            *diag_table_3[63].get(&blockers).unwrap(),
            BitBoard::from_ints(vec![54])
        );
    }

    #[test]
    fn test_orthogonal_table() {
        let slides_table = &traditional_slide_tables();
        let mut unblocked_attacks = BitBoard::empty();
        for direction in (0..TraditionalDirection::max_value()).step_by(2) {
            unblocked_attacks = unblocked_attacks | *slides_table[direction as usize][63].get(&BitBoard::empty()).unwrap();
        }
        assert_eq!(
            unblocked_attacks, // Only testing last node
            BitBoard::from_ints(vec![62, 61, 60, 59, 58, 57, 56, 55, 47, 39, 31, 23, 15, 7])
        );
        let blockers = BitBoard::from_ints(vec![62]);
        let mut blocked_attacks: BitBoard = BitBoard::empty();
        for direction in (0..TraditionalDirection::max_value()).step_by(2) {
            let unblocked = *slides_table[direction as usize][63].get(&BitBoard::empty()).unwrap();
            blocked_attacks = blocked_attacks | *slides_table[direction as usize][63].get(&(blockers & unblocked)).unwrap();
        }
        assert_eq!(
            blocked_attacks,
            BitBoard::from_ints(vec![55, 47, 39, 31, 23, 15, 7])
        )
    }

    #[test]
    fn test_king_table() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(63);
        assert_eq!(
            board.0.king_move_table()[source_node], // Only testing last node
            BitBoard::from_ints(vec![62, 55, 54])
        )
    }

    #[test]
    fn test_pawn_move_table_forward() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(8);
        assert_eq!(
            board.0.pawn_move_table(Color::White)[source_node],
            BitBoard::from_ints(vec![16, 24])
        )
    }

    #[test]
    fn test_pawn_move_table_backward() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(48);
        assert_eq!(
            board.0.pawn_move_table(Color::Black)[source_node],
            BitBoard::from_ints(vec![40, 32])
        )
    }

    #[test]
    fn test_pawn_move_table_one_space() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(48);
        assert_eq!(
            board.0.pawn_move_table(Color::White)[source_node],
            BitBoard::from_ints(vec![56])
        )
    }

    #[test]
    fn test_pawn_attack_table() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(49);
        assert_eq!(
            board.0.pawn_attack_table(Color::White)[source_node],
            BitBoard::from_ints(vec![56, 58])
        )
    }

    #[test]
    fn test_pawn_attack_table_at_edge() {
        let board = test_traditional_board();
        let source_node = NodeIndex::new(48);
        assert_eq!(
            board.0.pawn_attack_table(Color::Black)[source_node],
            BitBoard::from_ints(vec![41])
        )
    }

    #[test]
    fn test_hex_knight_table() {
        let board = test_hexagonal_board();
        let source_node = NodeIndex::new(0);
        assert_eq!(
            board.0.knight_jumps_table()[source_node], // Only testing last node
            BitBoard::from_ints(vec![9, 16, 22, 23])
        )
    }

    #[test]
    fn test_hex_queen_table() {
        let slide_tables = hexagonal_slide_tables();
        let mut attacks = BitBoard::empty();
        for direction in 0..HexagonalDirection::max_value() {
            attacks = attacks | *slide_tables[direction as usize][0].get(&BitBoard::empty()).unwrap();
        }
        assert_eq!(
            attacks,
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
        let source_node = NodeIndex::new(0);
        assert_eq!(
            board.0.king_move_table()[source_node],
            BitBoard::from_ints(vec![1, 6, 7, 8, 14])
        )
    }

    #[test]
    fn test_hex_pawn_move_table_backward() {
        let board = test_hexagonal_board();
        let source_node = NodeIndex::new(56);
        assert_eq!(
            board.0.pawn_move_table(Color::Black)[source_node],
            BitBoard::from_ints(vec![34, 45])
        )
    }
}
