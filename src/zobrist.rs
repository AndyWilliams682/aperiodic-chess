use rand::rngs::StdRng;
use rand::{SeedableRng, Rng};

use crate::constants::{MAX_NUM_TILES, NUM_PIECE_TYPES, NUM_PLAYERS};


#[derive(Debug)]
pub struct ZobristTable {
    pub pieces: [[[u64; MAX_NUM_TILES]; NUM_PIECE_TYPES]; NUM_PLAYERS],
    pub en_passant: [u64; MAX_NUM_TILES],
    pub black_to_move: u64
    // Ignoring castling rights for now
}

impl ZobristTable {
    pub fn generate() -> Self {
        let mut rng = StdRng::seed_from_u64(5435651169991665628);
        // TODO: Better syntax? Single for loop across all three things; it's doing permutations
        // Add a way to iterate over piece type variants, tiles, and players
        let mut pieces = [[[0; MAX_NUM_TILES]; NUM_PIECE_TYPES]; NUM_PLAYERS];
        let mut en_passant = [0; MAX_NUM_TILES]; // TODO: Can use less tiles, but would need to convert b/t them
        let black_to_move = rng.gen::<u64>();

        for tile_idx in 0..MAX_NUM_TILES {
            for player_idx in 0..NUM_PLAYERS {
                for piece_idx in 0..NUM_PIECE_TYPES {
                    pieces[player_idx][piece_idx][tile_idx] = rng.gen::<u64>();
                }
            }
            en_passant[tile_idx] = rng.gen::<u64>();
        }
        return Self { pieces, en_passant, black_to_move }
    }
}
