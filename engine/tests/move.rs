use engine::r#move::Move;
use engine::square::Square;

#[test]
fn test_move() {
    let from = Square::new(1, 4);
    let to = Square::new(3, 4);
    let r#move = Move::new(from, to, 'q');
    let move_string = format!("{}", r#move);

    assert_eq!(r#move.from, from);
    assert_eq!(r#move.to, to);
    assert_eq!(r#move.promotion, Some('q'));
    assert_eq!(move_string, "e2e4q");
}

#[test]
fn test_move_without_promotion() {
    let r#move = "e2e4".parse::<Move>().unwrap();
    let move_string = format!("{}", r#move);

    assert_eq!(r#move.from, "e2".parse::<Square>().unwrap());
    assert_eq!(r#move.to, "e4".parse::<Square>().unwrap());
    assert_eq!(r#move.promotion, None);
    assert_eq!(move_string, "e2e4");
}

#[test]
fn test_move_with_promotion() {
    let r#move = "e7e8q".parse::<Move>().unwrap();
    let move_string = format!("{}", r#move);

    assert_eq!(r#move.from, "e7".parse::<Square>().unwrap());
    assert_eq!(r#move.to, "e8".parse::<Square>().unwrap());
    assert_eq!(r#move.promotion, Some('q'));
    assert_eq!(move_string, "e7e8q");
}
