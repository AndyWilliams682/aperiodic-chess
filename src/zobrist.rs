use crate::piece_set::PieceType;




// TODO: Replace with actual random function
fn rand() -> u64 {
    return 0;
}


#[derive(Debug)]
pub struct ZobristTable {
    pub pieces: [[u64; 12]; 128], // TODO: Unhardcode instances of 128 in the code
    pub en_passant: [u64; 128],
    pub black_to_move: u64
    // Ignoring castling rights for now
}

impl ZobristTable {
    pub fn generate() -> Self {
        // TODO: Better syntax? Single for loop across both things?
        // Add a way to iterate over piece type variants and their indices
        let mut pieces = [[0; 12]; 128];
        let mut en_passant = [0; 128]; // TODO: Can use less tiles, but would need to convert b/t them
        let black_to_move = rand();

        for tile_index in 0..128 {
            for piece_type in 0..12 {
                pieces[piece_type][tile_index] = rand();
            }
            en_passant[tile_index] = rand();
        }
        return Self { pieces, en_passant, black_to_move }
    }
}
