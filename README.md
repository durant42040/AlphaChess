# AlphaChess

A chess AI who is better than dhgf.

## Getting Started

Install dependencies and start frontend:

``` bash
cd client
npm install
npm run start
```

Build and start server

``` bash
cd server
cargo run
```

## Architecture

This is a full-stack web application for a chess game. The player will play against Stockfish with 20-depth search. The application consists of three services:

* **Client**: Simple React App of a Chess game GUI, enables players to choose sides or let it be chosen randomly. The game supports drag and drop or clicking of pieces and sound effects for every move. The client-side connects to the backend via API calls with Rest.
* **Server**: Rust web server using Axum that
    1. Connects to Stockfish CLI to calculate the best move
    2. Validates moves through the engine
    3. Manages game state updates through the engine
* **Engine**: provides fast move generation and updates the chessboard according to each move as well as checking for draws and checkmates.

## Implementation

### Server

The web server is implemented using Axum. The server is set up with async handlers and routes:

``` rust
use axum::{Router, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/generate", get(generate_move))
        .route("/act", get(make_move))
        .route("/reset", get(reset))
        .route("/game", get(game));

    let listener = TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

Routes are defined as async functions that handle HTTP requests:

``` rust
async fn generate_move(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Value>), StockfishError> {
    let mut engine = state.engine.lock().await;
    let mut stockfish = state.stockfish.lock().await;
    
    let stockfish_output = stockfish.go()?;
    let move_string = stockfish_output.best_move();
    stockfish.play_move(move_string)?;
    engine.act(move_string.clone());
    
    Ok((StatusCode::OK, Json(json!({ "move": move_string }))))
}
```

Stockfish and the engine are both managed as shared, asynchronous state using `Arc<Mutex<T>>` to allow safe concurrent access by multiple HTTP requests.

``` rust
let engine = Arc::new(Mutex::new(Engine::new()));
let stockfish = Arc::new(Mutex::new(
    Stockfish::new("stockfish").expect("Failed to initialize Stockfish"),
));
```

#### Stockfish

The server uses the `stockfish` Rust crate to communicate with the Stockfish engine. The crate provides a convenient wrapper around the Stockfish binary:

``` rust
use stockfish::Stockfish;

let mut stockfish = Stockfish::new("stockfish")?;
stockfish.setup_for_new_game()?;
stockfish.set_depth(20);
```

To set up a position, you can use FEN notation or play moves:

``` rust
// Set position using FEN
let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
stockfish.set_fen_position(fen)?;

// Or play moves from the starting position
stockfish.play_moves(&["e2e4", "c7c5"])?;
```

To get the best move:

``` rust
let output = stockfish.go()?;
let best_move = output.best_move();
```

All moves are expressed in long algebraic notation (e.g., "e2e4").

### Chess Engine

#### Structure

* `engine.rs`:
  * `new()`: creates a new engine instance
  * `is_legal_move(Move)`: returns if a move is legal
  * `act(String)`: makes a move
  * `get_game_state()`: checks if the game is active, drawn, or won
  * `to_board_string()`: returns board as a string
* `chessboard.rs`: stores board information and updates the board for each move
* `move_generator.rs`: generates legal moves for each piece in each position
* `move.rs`: struct definition of `Move`, which stores start and target squares and promotion
* `bitboard.rs`: struct definition of `Bitboard`, provides bit operation methods and implements operator overloading
* `constants.rs`: precomputed magic bitboards and position keys

#### Bitboards

For fast move generation and board manipulation, an efficient data structure for storing and writing board information is needed. **Bitboards** are 64-bit integers (`u64` in Rust) used to represent an 8x8 chessboard. A bit of a bitboard is set if a chess piece is present on its square. Therefore, we can have a complete representation of a chessboard with 8 bitboards:

``` rust
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

We can check if a square is occupied with simple bit operations:

``` rust
bitboard & (1 << index) != 0
```

Similarly, we can set a square with:

``` rust
bitboard |= 1 << index;
```

Other operations such as bit count and getting the least significant bit can be implemented using Rust's built-in methods:

``` rust
pub fn count(&self) -> u32 {
    self.bitboard.count_ones()
}

pub fn get_lsb(&self) -> u8 {
    self.bitboard.trailing_zeros() as u8
}
```

Using bitboards as board representation thus allows for extremely fast board operations, even faster than basic arithmetic.

#### Move Generation

With the bitboards above, basic move generation involving kings and knights can be implemented with lookup tables with $O(1)$ time complexity. For pawns, bishops, and rooks, blockers become a problem, as generating the legal moves would require the location of blockers. For pawns, this problem is solved with some clever bit manipulations:

``` rust
let one_step_moves = (from_mask >> 8) & !all_pieces.bitboard;
let two_step_moves = ((one_step_moves & (0xFFu64 << 40)) >> 8) & !all_pieces.bitboard;
```

Capture moves can be generated with table lookups. For sliding pieces, a naive solution is to use loops:

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

After basic move generation, castling moves are added. Moves that put the king in check are filtered out:

``` rust
let temp_board = self.board.clone();
for to in legal_moves.iter() {
    self.board.act(Move::new(from, to.into(), None));
    if self.is_player_in_check(self.board.get_player().switch()) {
        legal_moves.clear(to);
    }
    self.board = temp_board.clone();
}
```

After that, promotions are added and the full move set is returned.
