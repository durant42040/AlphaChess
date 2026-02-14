# AlphaChess

A chess engine who is better than dhgf.

## Getting Started

### **Client**

```bash
cd client
pnpm dev
```

### **Server**

```bash
cd server
cargo run
```

### **Engine**

To test the engine, run

``` sh
cargo test
```

Play against the engine with, for example

``` sh
cargo play -- --side w --ponder 1000 --smp --threads 72
```

The engine will ponder for 1000ms per move and perform parallel search with 72 threads.

Run self-play with

``` sh
cargo selfplay
```

Play against Stockfish with

``` sh
cargo stockfish
```

To analyze a position

``` sh
cargo analyze -- -f <fen>
```
