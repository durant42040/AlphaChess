use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use engine::{Engine, Search};
use std::hint::black_box;

fn best_move_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("best_move");
    group.sample_size(10);

    let positions = [
        (
            "starting_position",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ),
        (
            "kiwipete",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        ),
    ];

    for (name, fen) in positions {
        group.bench_with_input(BenchmarkId::from_parameter(name), fen, |b, fen| {
            b.iter(|| {
                let mut engine = Engine::from_fen(fen);
                let mv = engine.best_move();
                black_box(mv)
            })
        });
    }

    group.finish();
}

criterion_group!(benches, best_move_benchmark);
criterion_main!(benches);
