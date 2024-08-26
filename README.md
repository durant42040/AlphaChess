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
mkdir build && cd build
cmake ..
make
./server
```
## Architecture
This is a full-stack web application for a chess game. The player will play against Stockfish with 20-depth search. The application consists of three services:
* **Client**: Simple React App of a Chess game GUI, enables players to choose sides or let it be chosen randomly. The game supports drag and drop or clicking of pieces and sound effects for every move. The client-side connects to the backend via api calls with Rest.
* **Server**: C++ web server that 
    1. Connects to Stockfish CLI to calculate the best move
    2. Validates moves through the engine
    3. Manages game state updates through the engine
* **Engine**: provides fast move generation and updates the chessboard according to each move as well as checking for draws and checkmates.
## Implementation
### Server
A web server in C++ can be implemented with basic socket programming. First, a socket is created with
``` C
int server_socket = socket(AF_INET, SOCK_STREAM, 0);
```
For communicating between processes on different hosts connected by IPV4, we use `AF_INET` as the domain. `SOCK_STREAM` is used for TCP connections. Next, the socket is bound to the `server_addr` with
``` C
bind(server_socket, (struct sockaddr*)&server_addr, sizeof(server_addr))
```
The socket begins to listen to incoming connections with    
``` C
listen(server_socket, 10)
```
Each incoming connection is put in the queue and is extracted and handled accordingly
``` C
int client_socket = accept(server_socket, nullptr, nullptr);

read(client_socket, buffer, 1024);
```
#### Stockfish
Communicating with the Stockfish engine requires running shell commands in the program. This is achieved with the `popen()` command in C, which additionally allows reading and writing from the process. 
``` C
std::unique_ptr<FILE, decltype(&pclose)> pipe(popen("stockfish", "r+"), pclose);
```
`fprintf()` sends the formatted input string to the process through the pipe. then `fflush()` flushes the buffer,  ensuring that any buffered data is sent to the process immediately.

to setup a position, the input command is
``` bash
position startpos moves <move_1> <move_2> ...
```
then we get the best move with
```
go depth 20
```
all moves are expressed with long algebraic notation.
### Chess Engine
#### Structure
* `engine.h`: 
    * `init_engine()`: precompute magic bitboards and position keys
    * `is_legal_move(Move move)`: return if a move is legal
    * `act(string move)`: make move
    * `get_game_state()`: check if the game is active, drawn, or won
    * `get_board()`: return board as a string
* `chessboard.h`: stores board information and updates the board for each move
* `move_generator.h`: generates legal moves for each piece in each position
* `move.h`: class definition of `Move`, which stores start and target squares and promotion.
* `bitboard.h`: class definition of `Bitboard`, provides bit operation methods and implements some operator overloading.
* `random.h`: random number generator for position hash.
#### Bitboards
For fast move generation and board manipulation, an efficient data structure for storing and writing board information is needed. **Bitboards** are 64-bit integers (`uint64_t` in C++) used to represent an 8x8 chessboard. A bit of a bitboard is set if a chess piece is present on its square. Therefore, we can have a complete representation of a chessboard with 8 bitboards:
``` C++
class ChessBoard {
    ...

    Bitboard white_pieces_;
    Bitboard black_pieces_;

    Bitboard pawns_;
    Bitboard knights_;
    Bitboard bishops_;
    Bitboard rooks_;
    Bitboard queens_;
    Bitboard kings_;
};
```
We can check if a square is occupied with simple bit operations
``` C++
bitboard & (1ULL << i);
```
similarly, we can set a square with
``` C++
bitboard |= (1ULL << i);
```
Other operations such as bit count and getting the least significant bit can be implemented with built-in methods.
``` C++
int count() const
{        
    return __builtin_popcountll(bitboard_);
}

int getLSB() 
{
    return __builtin_ctzll(bitboard_);
}
```
Using bitboards as board representation thus allows for extremely fast board operations, even faster than basic arithmetic.
#### Move Generation
With the bitboards above, basic move generation involving kings and knights can be implemented with lookup tables with $O(1)$ time complexity. For pawns, bishops, and rooks, blockers become a problem, as generating the legal moves would require the location of blockers. For pawns, this problem is solved with some clever bit manipulations:
``` C++
Bitboard one_step_moves = (from_mask >> 8) & ~all_pieces;
Bitboard two_step_moves = ((one_step_moves & (0xFFULL << 40)) >> 8) & ~all_pieces;
```
Capture moves can be generated with table lookups. For sliding pieces, a naive solution is to use for loops
``` C++
for (int i = rank + 1; i < 8; i++) {
    moves |= (1ULL << (8 * i + file));
    if (all_pieces.bitboard_ & 1ULL << (8 * i + file)) break;
}
```
However, this is extremely inefficient. For faster generation, a lookup table with indices that encode blocker positions are devised, called **Magic Bitboards**. 

Given the blocker positions, the legal moves for rooks and bishops in every square and every possible combination of blockers can be precomputed. The difficulty lies in the storage of this information. Using the square and the blocker bitboard as indices to a 2D array is simply too inefficient, since it would require up to $64 \times 2^{64}$ long long integers to be stored in memory. Instead, for each square, a magic number is computed to scale down the size of the array. The blocker is multiplied with this magic number, and then right-shifted to reduce the index value.
``` C++
uint64_t key = (blockers * bishop_magic_numbers[square]) >> (64 - bishop_shift_bits[square]);
```
With this key generation procedure, the full move set is precomputed as follows
``` C++
for (int square = 0; square < 64; square++) {
    for (int i = 0; i < (1 << bishop_shift_bits[square]); i++) {
        uint64_t blockers = get_blockers(i, bishop_masks[square]);
        uint64_t key = (blockers * bishop_magic_numbers[square]) >> (64 - bishop_shift_bits[square]);
        bishop_table[square][key] = generate_bishop_moves_slow(Square(square), Bitboard(blockers));
    }
}
```
then the move sets can be retrieved by recomputing the key and performing a lookup
``` C++
uint64_t blockers = all_pieces.bitboard_ & bishop_masks[from.square_].bitboard_;
uint64_t key = (blockers * bishop_magic_numbers[from.square_]) >> (64 - bishop_shift_bits[from.square_]);
return bishop_table[from.square_][key];
```
After basic move generation, castling moves are added. Moves that put the king in check are filtered out.
``` C++
for (auto to : moves) {
    ChessBoard temp_board = *this;
    temp_board.act(Move(from, to, '\0'), false);
    if (temp_board.is_player_in_check(player_)) {            
        moves.clear(to);
    }
}
``` 
After that, promotions are added and the full move set is returned.
