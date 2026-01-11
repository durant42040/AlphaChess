use engine::engine::Engine;

/// Helper function to make a move and assert it's valid
fn make_move(engine: &mut Engine, move_str: &str) {
    assert!(
        engine.act(move_str.to_string()),
        "Move {} should be valid",
        move_str
    );
    println!("{}", engine);
}

/// Helper function to make a move and assert it's invalid
fn assert_invalid_move(engine: &mut Engine, move_str: &str) {
    assert!(
        !engine.act(move_str.to_string()),
        "Move {} should be invalid",
        move_str
    );
}

/// Helper function to assert game state
fn assert_game_state(engine: &Engine, expected: &str) {
    assert_eq!(
        engine.get_game_state(),
        expected,
        "Expected game state '{}', got '{}'",
        expected,
        engine.get_game_state()
    );
}

// ============================================================================
// Game Scenarios
// ============================================================================

#[test]
fn test_foolsmate() {
    // Fool's mate: fastest possible checkmate
    // 1. f3 e5
    // 2. g4 Qh4#
    // Note: This requires the queen to have a clear diagonal path
    // If the path is blocked, we'll test with a working checkmate pattern instead
    let mut engine = Engine::new();

    make_move(&mut engine, "f2f3");
    make_move(&mut engine, "e7e5");
    make_move(&mut engine, "g2g4");
    make_move(&mut engine, "d8h4");

    assert_game_state(&engine, "checkmate");
}

#[test]
fn test_scholars_mate() {
    // Scholar's mate: 4-move checkmate
    // 1. e4 e5 d1h5 g7g6 h5f3 f8g7 f3f7#
    let mut engine = Engine::new();

    make_move(&mut engine, "e2e4");
    make_move(&mut engine, "e7e5");
    make_move(&mut engine, "f1c4");
    make_move(&mut engine, "b8c6");
    make_move(&mut engine, "d1h5");
    make_move(&mut engine, "g7g6");
    make_move(&mut engine, "h5f3");
    make_move(&mut engine, "f8g7");
    make_move(&mut engine, "f3f7");

    assert_game_state(&engine, "checkmate");
}

#[test]
fn test_castling() {
    let mut engine = Engine::new();

    make_move(&mut engine, "e2e4");
    make_move(&mut engine, "e7e5");
    make_move(&mut engine, "g1f3");
    make_move(&mut engine, "f8d6");
    make_move(&mut engine, "f1c4");
    make_move(&mut engine, "g8f6");

    make_move(&mut engine, "e1g1");

    make_move(&mut engine, "e8g8");

    assert_game_state(&engine, "playing");

    engine.reset();
    make_move(&mut engine, "d2d4");
    make_move(&mut engine, "d7d5");
    make_move(&mut engine, "c1f4");
    make_move(&mut engine, "c8f5");
    make_move(&mut engine, "b1c3");
    make_move(&mut engine, "b8c6");
    make_move(&mut engine, "d1d2");
    make_move(&mut engine, "d8d7");

    make_move(&mut engine, "e1c1");
    make_move(&mut engine, "e8c8");
    assert_game_state(&engine, "playing");
}

#[test]
fn test_en_passant() {
    let mut engine = Engine::new();

    make_move(&mut engine, "e2e4");
    make_move(&mut engine, "b8c6");
    make_move(&mut engine, "e4e5");
    make_move(&mut engine, "f7f5");
    make_move(&mut engine, "e5f6");
    assert_game_state(&engine, "playing");
}
