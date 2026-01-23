use criterion::{Criterion, criterion_group, criterion_main};
use engine::engine::Engine;
use engine::square::Square;
use std::hint::black_box;

fn generate_all_legal_moves_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_generation");

    // Starting position
    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    group.bench_function("generate_all_legal_moves_starting", |b| {
        b.iter(|| {
            let mut engine = Engine::from_fen(starting_fen.to_string());
            let moves = engine.generate_all_legal_moves();
            black_box(moves)
        })
    });

    // Kiwipete position (more complex)
    let kiwipete_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";

    group.bench_function("generate_all_legal_moves_kiwipete", |b| {
        b.iter(|| {
            let mut engine = Engine::from_fen(kiwipete_fen.to_string());
            let moves = engine.generate_all_legal_moves();
            black_box(moves)
        })
    });

    // Position in check
    let check_fen = "r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4";

    group.bench_function("generate_all_legal_moves_in_check", |b| {
        b.iter(|| {
            let mut engine = Engine::from_fen(check_fen.to_string());

            let moves = engine.generate_all_legal_moves();
            black_box(moves)
        })
    });

    group.finish();
}

fn generate_legal_moves_per_square_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate_legal_moves_square");

    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    // Test different piece types
    let test_squares = vec![
        ("pawn", Square::from(12)),  // e2 pawn
        ("knight", Square::from(1)), // b1 knight
        ("bishop", Square::from(2)), // c1 bishop
        ("rook", Square::from(0)),   // a1 rook
        ("queen", Square::from(3)),  // d1 queen
        ("king", Square::from(4)),   // e1 king
    ];

    for (piece_name, square) in test_squares {
        group.bench_function(format!("{}_square_{}", piece_name, square), |b| {
            let mut engine = Engine::from_fen(starting_fen.to_string());

            b.iter(|| {
                let moves = engine.generate_legal_moves(black_box(square));
                black_box(moves)
            })
        });
    }

    group.finish();
}

fn check_detection_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("check_detection");

    // Not in check
    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    group.bench_function("is_check_false", |b| {
        b.iter(|| {
            let mut engine = Engine::from_fen(starting_fen.to_string());
            let result = engine.is_check();
            black_box(result)
        })
    });

    // In check
    let check_fen = "r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4";

    group.bench_function("is_check_true", |b| {
        b.iter(|| {
            let mut engine = Engine::from_fen(check_fen.to_string());

            let result = engine.is_check();
            black_box(result)
        })
    });

    group.finish();
}

fn castling_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("castling");

    // Position where castling is possible
    let castling_fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";

    group.bench_function("generate_castling_moves", |b| {
        b.iter(|| {
            let mut engine = Engine::from_fen(castling_fen.to_string());
            // Generate moves for king (which includes castling)
            let moves = engine.generate_legal_moves(Square::from(4));
            black_box(moves)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    generate_all_legal_moves_benchmark,
    generate_legal_moves_per_square_benchmark,
    check_detection_benchmark,
    castling_benchmark
);
criterion_main!(benches);
