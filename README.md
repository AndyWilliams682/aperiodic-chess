A chess engine written in Rust that can support different board topologies (currently focusing on traditional square board and glinski's hexagonal chess). This is still a work-in-progress. My ultimate goal is to use this to explore different, custom chess variants, specifically using aperiodic tiling.

## Notable changes in design
- Because a hexagonal board has 91 tiles, we cannot represent each tile in a u64. BitBoards are represented using u128 to accomodate the additional tiles.
- To keep things general, I represent the tiles as nodes in a graph. If you can move from one tile to another (orthogonally or diagonally), they are connected with an edge. These edges are weighted to represent specific directions (north, south, north-west). All of the bitboards that store movement logic are derived from this graph, and then the rest of the code is completely independent of the graph itself. (The edges are also directed, which is not relevant for these board types, but may become relevant in the future).

## Current goal
I have recently finished implementing perft(N), which is a performance testing function for move generation that counts the number of possible moves from a given position after N turns. I am getting accurate results, but suboptimal performance beyond perft(6). perft(5) finishes in under a second, perft(6) takes about 20.
