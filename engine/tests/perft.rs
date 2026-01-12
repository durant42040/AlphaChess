use engine::engine::Engine;
use engine::engine::Perft;

#[test]
fn test_starting_perft() {
    let results = vec![20, 400, 8902, 197281, 4865609];

    let mut engine = Engine::new();
    engine.load_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    for depth in 1..6 {
        let nodes = engine.perft(depth);
        assert_eq!(nodes, results[(depth - 1) as usize]);
    }
}

#[test]
fn test_kiwipete_perft() {
    let results = vec![48, 2039, 97862, 4085603];

    let mut engine = Engine::new();
    engine.load_from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -".to_string(),
    );
    for depth in 1..5 {
        let nodes = engine.perft(depth);
        assert_eq!(nodes, results[(depth - 1) as usize]);
    }
}
