//! Tests to assess evaluation accuracy: symmetry, material, and sanity checks.

use engine::Engine;
use engine::search::Evaluation;

/// Eval should flip sign when the same position is evaluated with the opposite side to move.
/// (Your eval is from the side-to-move perspective: positive = good for the side to move.)
#[test]
fn eval_symmetry_side_to_move() {
    let fen_board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R";
    let fen_tail = "0 1";

    let engine_white = Engine::from_fen(&format!("{} w KQkq - {}", fen_board, fen_tail));
    let engine_black = Engine::from_fen(&format!("{} b KQkq - {}", fen_board, fen_tail));

    let eval_white = engine_white.eval();
    let eval_black = engine_black.eval();

    assert_eq!(
        eval_white, -eval_black,
        "Same position with opposite side to move: eval should flip sign (white={}, black={})",
        eval_white, eval_black
    );
}

/// Symmetry on the starting position.
#[test]
fn eval_symmetry_starting_position() {
    let engine_white = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let engine_black = Engine::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");

    let eval_white = engine_white.eval();
    let eval_black = engine_black.eval();

    assert_eq!(
        eval_white, -eval_black,
        "Starting position: evals should be opposite"
    );
}

/// Clear material advantage (White has queen, Black only king) should yield a large positive eval for White.
#[test]
fn eval_material_advantage_white() {
    // White: K+Q vs Black: K. White to move.
    let engine = Engine::from_fen("4k3/8/8/8/8/8/8/4Q1K1 w - - 0 1");
    let e = engine.eval();
    assert!(
        e > 400,
        "White up a queen should have eval > 400 cp, got {}",
        e
    );
}

/// Same as above but Black to move: eval should be large positive for Black (side to move), i.e. negative in "white advantage" terms.
/// Our eval is for side-to-move, so Black to move with Black up a queen would be positive; here White is up a queen and Black to move, so eval (good for Black) should be negative.
#[test]
fn eval_material_advantage_white_black_to_move() {
    let engine = Engine::from_fen("4k3/8/8/8/8/8/8/4Q1K1 b - - 0 1");
    let e = engine.eval();
    assert!(
        e < -400,
        "Black to move, White up a queen: eval (for Black) should be < -400 cp, got {}",
        e
    );
}
