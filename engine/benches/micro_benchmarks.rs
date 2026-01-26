use criterion::{Criterion, criterion_group, criterion_main};
use engine::chess::bitboard::Bitboard;
use engine::chess::chessboard::ChessBoard;
use engine::chess::move_generator::MoveGenerator;
use engine::chess::square::Square;
use std::hint::black_box;

fn move_generator_functions_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_generator");

    let move_gen = MoveGenerator::new();
    let all_pieces = Bitboard::from(0xFFFF00000000FFFF); // Starting position pieces

    // Pawn moves
    group.bench_function("generate_white_pawn_moves", |b| {
        let from = Square::from(12); // e2
        let capture_pieces = Bitboard::from(0xFF00000000FF00);

        b.iter(|| {
            let moves = move_gen.generate_white_pawn_moves(
                black_box(from),
                black_box(all_pieces),
                black_box(capture_pieces),
            );
            black_box(moves)
        })
    });

    group.bench_function("generate_black_pawn_moves", |b| {
        let from = Square::from(52); // e7
        let capture_pieces = Bitboard::from(0xFF00000000FF00);

        b.iter(|| {
            let moves = move_gen.generate_black_pawn_moves(
                black_box(from),
                black_box(all_pieces),
                black_box(capture_pieces),
            );
            black_box(moves)
        })
    });

    // Knight moves
    group.bench_function("generate_knight_moves", |b| {
        let from = Square::from(1); // b1

        b.iter(|| {
            let moves = move_gen.generate_knight_moves(black_box(from));
            black_box(moves)
        })
    });

    // Bishop moves
    group.bench_function("generate_bishop_moves", |b| {
        let from = Square::from(2); // c1

        b.iter(|| {
            let moves = move_gen.generate_bishop_moves(black_box(from), black_box(all_pieces));
            black_box(moves)
        })
    });

    // Rook moves
    group.bench_function("generate_rook_moves", |b| {
        let from = Square::from(0); // a1

        b.iter(|| {
            let moves = move_gen.generate_rook_moves(black_box(from), black_box(all_pieces));
            black_box(moves)
        })
    });

    // Queen moves
    group.bench_function("generate_queen_moves", |b| {
        let from = Square::from(3); // d1

        b.iter(|| {
            let moves = move_gen.generate_queen_moves(black_box(from), black_box(all_pieces));
            black_box(moves)
        })
    });

    // King moves
    group.bench_function("generate_king_moves", |b| {
        let from = Square::from(4); // e1

        b.iter(|| {
            let moves = move_gen.generate_king_moves(black_box(from));
            black_box(moves)
        })
    });

    group.finish();
}

fn bitboard_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitboard");

    let bb1 = Bitboard::from(0xFFFF00000000FFFF);
    let bb2 = Bitboard::from(0xFF00FF00FF00FF00);

    group.bench_function("bitboard_and", |b| {
        b.iter(|| {
            let result = black_box(bb1) & black_box(bb2);
            black_box(result)
        })
    });

    group.bench_function("bitboard_or", |b| {
        b.iter(|| {
            let result = black_box(bb1) | black_box(bb2);
            black_box(result)
        })
    });

    group.bench_function("bitboard_not", |b| {
        b.iter(|| {
            let result = !black_box(bb1);
            black_box(result)
        })
    });

    group.bench_function("bitboard_count", |b| {
        b.iter(|| {
            let count = black_box(bb1).count();
            black_box(count)
        })
    });

    group.bench_function("bitboard_iter", |b| {
        b.iter(|| {
            let count: u32 = black_box(bb1).iter().map(|_| 1).sum();
            black_box(count)
        })
    });

    group.bench_function("bitboard_get_lsb", |b| {
        b.iter(|| {
            let lsb = black_box(bb1).get_lsb();
            black_box(lsb)
        })
    });

    group.finish();
}

fn board_cloning_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("board_operations");

    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = ChessBoard::load_from_fen(starting_fen);

    group.bench_function("board_clone", |b| {
        b.iter(|| {
            let cloned = black_box(&board);
            black_box(cloned)
        })
    });

    group.bench_function("get_pieces", |b| {
        b.iter(|| {
            let pieces = black_box(&board).pieces();
            black_box(pieces)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    move_generator_functions_benchmark,
    bitboard_operations_benchmark,
    board_cloning_benchmark
);
criterion_main!(benches);
