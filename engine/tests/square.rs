use engine::square::Square;

#[test]
fn test_square_display_new() {
    let square = Square::new(0, 0);
    let square_string = format!("{}", square);

    assert_eq!(square_string, "a1");
}

#[test]
fn test_square_display_from_string() {
    let square = "e4".parse::<Square>().unwrap();
    let square_string = format!("{}", square);

    assert_eq!(square_string, "e4");
}
