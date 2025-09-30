use crate::piece_set::PieceType;
use rand::rngs::StdRng;
use rand::{SeedableRng, Rng};


#[derive(Debug)]
pub struct ZobristTable {
    pub pieces: [[[u64; 128]; 6]; 2], // TODO: Unhardcode instances of 128 in the code
    pub en_passant: [u64; 128],
    pub black_to_move: u64
    // Ignoring castling rights for now
}

impl ZobristTable {
    pub fn generate() -> Self {
        let mut rng = StdRng::seed_from_u64(5435651169991665628);
        // TODO: Better syntax? Single for loop across both things?
        // Add a way to iterate over piece type variants and their indices
        let mut pieces = [[[0; 128]; 6]; 2];
        let mut en_passant = [0; 128]; // TODO: Can use less tiles, but would need to convert b/t them
        let black_to_move = rng.gen::<u64>();

        for tile_idx in 0..128 {
            for player_idx in 0..2 {
                for piece_idx in 0..6 {
                    pieces[player_idx][piece_idx][tile_idx] = rng.gen::<u64>();
                }
            }
            en_passant[tile_idx] = rng.gen::<u64>();
        }
        return Self { pieces, en_passant, black_to_move }
    }
}
