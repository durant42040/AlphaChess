# AlphaChess

A chess engine who is better than dhgf.

## Getting Started

**Client**

```bash
cd client
npm install
npm run dev
```

**Server**

```bash
cd server
cargo run
```

## Architecture

This is a full-stack web application for a chess game. The player will play against the engine. The application consists of three services:

* **Client**: React App of a Chess game GUI
* **Server**: Rust web server using Axum
* **Engine**:
  * fast move generation using bitboards
  * checks for draws and checkmates
  * generates best move with alpha-beta search.

## Implementation

### Server

The server is implemented with Axum and shared engine state:

```rust
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let engine = Arc::new(Mutex::new(Engine::new()));

    let app = Router::new()
        .route("/ping", get(ping))
        .route("/generate", get(generate_move))
        .route("/act", get(make_move))
        .route("/reset", get(reset))
        .route("/game", get(game))
        .route("/undo", get(undo_move))
        .with_state(engine);

    let listener = TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

* **`/generate`** — Computes the best move via `engine.best_move()`, applies it, and returns the move and updated board.
* **`/act?move=e2e4`** — Validates and applies the given move (e.g. long algebraic notation), returns board and check status.
* **`/reset`** — Resets the game to the starting position.
* **`/game`** — Returns the current game state (e.g. playing, draw, checkmate).
* **`/undo`** — Undoes the last two half-moves.

The engine is stored in `Arc<Mutex<Engine>>` so all handlers share one game.

### Chess Engine

#### Structure

* **`engine.rs`** — Main API: `new()`, `from_fen()`, `reset()`, `make_move()`, `act()`, `undo()`, `game_state()`, `to_board_string()`, `is_check()`, and evaluation/search helpers.
* **`search/mod.rs`** — `Search` trait implemented for `Engine`: `max_search`, `minimax_search`, `alpha_beta_search`, `best_move()`.
* **`chess/chessboard.rs`** — Board representation, castling rights, move application and undo.
* **`chess/move_generator.rs`** — Legal move generation for all piece types.
* **`chess/move.rs`** — `Move` type (from/to squares, promotion).
* **`chess/bitboard.rs`** — `Bitboard` and bit operations.
* **`chess/constants.rs`** — Precomputed magic bitboards and related tables.
* **`chess/pieces.rs`** — Piece sets (white/black, by type) and board occupancy.
* **`chess/square.rs`** — Square indexing and notation.

#### Bitboards

Board state is represented with 64-bit bitboards: one bit per square. For example, piece sets and occupancy:

```rust
pub struct Pieces {
    pub pawns: Bitboard,
    pub knights: Bitboard,
    pub bishops: Bitboard,
    pub rooks: Bitboard,
    pub queens: Bitboard,
    pub kings: Bitboard,
    pub white_pieces: Bitboard,
    pub black_pieces: Bitboard,
    pub all_pieces: Bitboard,
    pub en_passant: Bitboard,
}
```

Square tests and updates use bit operations:

```rust
bitboard & (1 << index) != 0
bitboard |= 1 << index;
```

Count and LSB use standard library helpers:

```rust
pub fn count(&self) -> u32 {
    self.bitboard.count_ones()
}

pub fn get_lsb(&self) -> u8 {
    self.bitboard.trailing_zeros() as u8
}
```

#### Move Generation

Kings and knights use O(1) lookup tables. Pawns use bit masks for advances and captures. For example, white pawn one- and two-step moves:

```rust
let one_step_moves = (from_mask >> 8) & !all_pieces.bitboard;
let two_step_moves = ((one_step_moves & (0xFFu64 << 40)) >> 8) & !all_pieces.bitboard;
```

Sliding pieces (rooks, bishops) use **magic bitboards**: precomputed tables indexed by square and blocker pattern. A magic number hashes the blocker configuration into a compact index:

``` rust
for i in (rank + 1)..8 {
    moves.bitboard |= 1 << (8 * i + file);
    if all_pieces.bitboard & (1 << (8 * i + file)) != 0 {
        break;
    }
}
```

However, this is extremely inefficient. For faster generation, a lookup table with indices that encode blocker positions are devised, called **Magic Bitboards**.

Given the blocker positions, the legal moves for rooks and bishops in every square and every possible combination of blockers can be precomputed. The difficulty lies in the storage of this information. Using the square and the blocker bitboard as indices to a 2D array is simply too inefficient, since it would require up to $64 \times 2^{64}$ long long integers to be stored in memory. Instead, for each square, a magic number is computed to scale down the size of the array. The blocker is multiplied with this magic number, and then right-shifted to reduce the index value.

``` rust
let key = (blockers * BISHOP_MAGIC_NUMBERS[square as usize]) >> (64 - BISHOP_SHIFT_BITS[square as usize]);
```

With this key generation procedure, the full move set is precomputed as follows:

``` rust
for square in 0..64 {
    for i in 0..(1 << BISHOP_SHIFT_BITS[square]) {
        let blockers = get_blockers(i, BISHOP_MASKS[square]);
        let key = (blockers * BISHOP_MAGIC_NUMBERS[square]) >> (64 - BISHOP_SHIFT_BITS[square]);
        BISHOP_TABLE[square][key as usize] = generate_bishop_moves_slow(Square::from(square), Bitboard::from(blockers));
    }
}
```

Then the move sets can be retrieved by recomputing the key and performing a lookup:

``` rust
let blockers = all_pieces.bitboard & BISHOP_MASKS[from.square as usize].bitboard;
let key = (blockers * BISHOP_MAGIC_NUMBERS[from.square as usize]) >> (64 - BISHOP_SHIFT_BITS[from.square as usize]);
BISHOP_TABLE[from.square as usize][key as usize]
```

After generating candidate moves, castling is added, and moves that leave the king in check are removed. Promotions are then added to produce the final legal move list.
