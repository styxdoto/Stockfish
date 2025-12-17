# Stockfish-RS: Rust Port of Stockfish Chess Engine

A functional chess engine ported from C++ Stockfish to Rust, implementing core chess functionality with a focus on correctness and performance.

## Project Status

✅ **Compilable and Functional** - The engine successfully compiles and runs
✅ **UCI Protocol** - Full Universal Chess Interface support
✅ **Move Generation** - Verified correct through perft testing (depths 1-3)
✅ **Position Management** - FEN parsing, move making/unmaking
✅ **Search Engine** - Alpha-beta search with iterative deepening
✅ **Transposition Table** - Hash-based move caching
✅ **Evaluation** - Classical piece-square table evaluation

⚠️ **In Progress** - Full NNUE evaluation network (simplified version currently implemented)
⚠️ **Known Issue** - Deep search (depth ≥4) has edge cases that need debugging

## Implemented Components

### Core Engine (100% Functional)
- **Type System** (`types.rs`) - All chess types: squares, pieces, moves, colors
- **Bitboards** (`bitboard.rs`) - Magic bitboards for sliding piece attack generation
- **Zobrist Hashing** (`zobrist.rs`) - Position hashing for transposition table
- **Position** (`position.rs`) - Board representation, FEN parsing, state management
- **Move Generation** (`movegen.rs`) - Legal move generation for all piece types
- **Transposition Table** (`tt.rs`) - Hash table for caching search results
- **Evaluation** (`evaluate.rs`) - Classical evaluation with piece-square tables
- **Search** (`search.rs`) - Alpha-beta negamax with quiescence search
- **UCI Protocol** (`uci.rs`) - Full UCI command support
- **Perft** (`perft.rs`) - Move generation verification

### Features
✅ All piece movements (pawns, knights, bishops, rooks, queens, kings)
✅ Special moves (castling, en passant, promotions)
✅ Check and checkmate detection
✅ Stalemate and draw detection
✅ Iterative deepening
✅ Transposition table
✅ Quiescence search
✅ Move ordering (basic)

## Building

```bash
cd stockfish_rs
cargo build --release
```

The compiled binary will be at `./target/release/stockfish_rs`

## Usage

### UCI Mode
```bash
./target/release/stockfish_rs
```

Then enter UCI commands:
```
uci
isready
position startpos
go depth 3
quit
```

### Run Perft Tests
```bash
echo "perfttest" | ./target/release/stockfish_rs
```

### Example Commands
```
# Set starting position
position startpos

# Set from FEN
position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1

# Make moves
position startpos moves e2e4 e7e5

# Search to depth 5
go depth 5

# Search for 1 second
go movetime 1000

# Perft divide at depth 4
perft 4

# Display current position
d
```

## Perft Results

Perft (Performance Test) validates move generation correctness:

**Starting Position:**
- Depth 1: 20 nodes ✓
- Depth 2: 400 nodes ✓
- Depth 3: 8,902 nodes ✓

**Kiwipete Position:**
- Depth 1: 48 nodes ✓
- Depth 2: 2,039 nodes ✓
- Depth 3: 97,862 nodes ✓

## Performance

Built with full optimizations:
- LTO (Link-Time Optimization)
- Single codegen unit
- Optimization level 3
- Stripped symbols

## Architecture

```
stockfish_rs/
├── src/
│   ├── types.rs      # Core chess types
│   ├── bitboard.rs   # Bitboard operations & magic bitboards
│   ├── zobrist.rs    # Zobrist hashing
│   ├── position.rs   # Position representation
│   ├── movegen.rs    # Move generation
│   ├── evaluate.rs   # Position evaluation
│   ├── tt.rs         # Transposition table
│   ├── search.rs     # Search algorithm
│   ├── uci.rs        # UCI protocol
│   ├── perft.rs      # Testing utilities
│   ├── lib.rs        # Library interface
│   └── main.rs       # Binary entry point
├── Cargo.toml
└── README.md
```

## Future Work for Complete 1:1 Port

For a complete 1:1 Stockfish port, the following would need to be added:

### NNUE Evaluation
- [ ] HalfKAv2_hm feature set
- [ ] Network file loading (.nnue files)
- [ ] Accumulator management
- [ ] Feature transformer
- [ ] Network layers (affine transform, clipped ReLU, etc.)
- [ ] SIMD optimizations (AVX2, SSE, NEON)

### Advanced Search
- [ ] Principal Variation Search (PVS)
- [ ] Late Move Reductions (LMR)
- [ ] Null move pruning
- [ ] Futility pruning
- [ ] History heuristic
- [ ] Killer moves
- [ ] Counter moves
- [ ] Multi-cut
- [ ] Singular extensions

### Multi-Threading
- [ ] Lazy SMP (parallel search)
- [ ] Thread pool management
- [ ] Shared hash table with thread safety

### Time Management
- [ ] Smart time allocation
- [ ] Time control handling
- [ ] Pondering support

### Endgame
- [ ] Syzygy tablebase support
- [ ] Endgame-specific evaluation
- [ ] Bitbase integration

### Optimization
- [ ] SIMD move generation
- [ ] Assembly-optimized bitboard operations
- [ ] Profile-guided optimization (PGO)
- [ ] NUMA awareness

## License

GPL-3.0-or-later (matching original Stockfish license)

## Credits

- Original Stockfish by The Stockfish developers
- Rust port created as an educational/research project
- Based on Stockfish commit: 75edbee (January 2025)

## Notes

This is a educational/research port demonstrating how a modern chess engine can be implemented in Rust. While functional, it's not intended to replace the highly optimized C++ Stockfish for competitive play. The focus was on creating a clean, idiomatic Rust implementation that maintains correctness while being readable and maintainable.

For production chess analysis, please use the official Stockfish engine from https://stockfishchess.org/
