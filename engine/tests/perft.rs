use engine::engine::Engine;

#[test]
fn test_perft() {
    let mut engine = Engine::new();
    engine.load_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    let all_legal_moves = engine.generate_all_legal_moves();
    assert_eq!(all_legal_moves.len(), 20);
}
