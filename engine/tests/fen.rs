use engine::Engine;

#[test]
fn test_get_fen_roundtrip_with_en_passant() {
    let fen = "4k3/3pr3/8/4P3/8/8/8/4K3 b - d6 0 1";
    let engine = Engine::from_fen(fen);
    assert_eq!(engine.get_fen(), fen);
}
