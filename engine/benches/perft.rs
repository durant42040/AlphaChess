use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use engine::Perft;
use std::hint::black_box;

fn perft_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft");
    group.sample_size(10);

    // Starting position
    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let expected_nodes = vec![20, 400, 8902, 197281, 4865609];

    for depth in 1..=5 {
        group.bench_with_input(
            BenchmarkId::new("starting_position", depth),
            &depth,
            |b, &depth| {
                let mut perft = Perft::new(starting_fen);
                b.iter(|| {
                    let nodes = perft.search(black_box(depth));
                    assert_eq!(nodes, expected_nodes[(depth - 1) as usize]);
                    nodes
                })
            },
        );
    }

    // Kiwipete position
    let kiwipete_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
    let kiwipete_nodes = vec![48, 2039, 97862, 4085603];

    for depth in 1..=4 {
        group.bench_with_input(BenchmarkId::new("kiwipete", depth), &depth, |b, &depth| {
            let mut perft = Perft::new(kiwipete_fen);
            b.iter(|| {
                let nodes = perft.search(black_box(depth));
                assert_eq!(nodes, kiwipete_nodes[(depth - 1) as usize]);
                nodes
            })
        });
    }

    // Position 3
    let pos3_fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    let pos3_nodes = vec![14, 191, 2812, 43238, 674624];

    for depth in 1..=5 {
        group.bench_with_input(
            BenchmarkId::new("position_3", depth),
            &depth,
            |b, &depth| {
                let mut perft = Perft::new(pos3_fen);
                b.iter(|| {
                    let nodes = perft.search(black_box(depth));
                    assert_eq!(nodes, pos3_nodes[(depth - 1) as usize]);
                    nodes
                })
            },
        );
    }

    group.finish();
}

fn perft_nodes_per_second(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft_nps");
    group.sample_size(10);

    let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    for depth in 3..=5 {
        group.bench_with_input(BenchmarkId::new("nps", depth), &depth, |b, depth| {
            let mut perft = Perft::new(starting_fen);
            b.iter(|| {
                let nodes = perft.search(black_box(*depth));
                let _nps = nodes;
                nodes
            })
        });
    }

    group.finish();
}

criterion_group!(benches, perft_benchmark, perft_nodes_per_second);
criterion_main!(benches);
