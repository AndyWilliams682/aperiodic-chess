use itertools::Itertools;
use petgraph::graph::{Graph, Node, NodeIndex}; // 0.6.5
use petgraph::dot::{Dot, Config};
use petgraph::visit::EdgeRef;
use std::collections::HashSet;

mod graph_board;
mod piece;
mod bit_board;
mod limited_int;
// mod limited_int {
//     macro_rules! create_limited_int {
//         () => ()
//     }

//     pub(crate) use create_limited_int;
// }

const RANKS: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
const FILES: [i32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
const DIRECTIONS: [i32; 8] = [0, 1, 2, 3, 4, 5, 6, 7]; // Odds are diagonals


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Space {
    coords: String,
    piece_present: i32
}

impl Space {
    fn new(coords: &str) -> Space {
        return Space { coords: coords.to_string(), piece_present: 0 }
    }

    fn directions_with_nodes(&self) -> Vec<i32> {
        let mut result = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let mut invalid = HashSet::new();
        if self.coords.contains('1') {
            invalid.insert(3);
            invalid.insert(4);
            invalid.insert(5);
        } else if self.coords.contains('8') {
            invalid.insert(0);
            invalid.insert(1);
            invalid.insert(7);
        }
        if self.coords.contains('a') {
            invalid.insert(5);
            invalid.insert(6);
            invalid.insert(7);
        } else if self.coords.contains('h') {
            invalid.insert(1);
            invalid.insert(2);
            invalid.insert(3);
        }
        for direction in invalid {
            result.retain(|element| element != &direction);
        }
        return result
    }
   
}


fn main() {
    // let test = GraphBoard::empty_traditional();
    // println!("{:?}", Dot::new(&test.board_graph));
    // let mut board = Graph::<Space, i32>::new();
    
    // for cell in RANKS.iter().cartesian_product(FILES.iter()) {
    //     let coords = cell.0.to_string() + &cell.1.to_string();
    //     let space = Space::new(&coords);
    //     board.add_node(space.clone());
    // }

    // *board.node_weight_mut(0.into()).unwrap() = Space { coords: "a1".to_string(), piece_present: 1 };
    // *board.node_weight_mut(27.into()).unwrap() = Space { coords: "c3".to_string(), piece_present: -1 };

    // for node_idx in board.node_indices() {
    //     let space = &board[node_idx];
    //     let valid_directions = space.directions_with_nodes();
    //     for direction in valid_directions {
    //         let other_idx = NodeIndex::from((node_idx.index() as i32 + get_node_index_shift(direction)) as u32);
    //         board.add_edge(node_idx, other_idx, direction);
    //     }
    // }
    // // let test = get_valid_slides_in_direction(&board, 0.into(), 1, 0);
    // let test = get_valid_knight_moves(&board, 63.into());
    // println!("{:?}", test);

    // println!("{:?}", board);
    // println!("{:?}", Dot::with_config(&board, &[Config::NodeNoLabel]));
}
