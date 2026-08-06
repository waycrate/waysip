use crate::error::ColorError;
use crate::utils::*;

#[test]
fn size_from_tuple() {
    let size: Size = (10, 20).into();
    assert_eq!(size.width, 10);
    assert_eq!(size.height, 20);
}

#[test]
fn color_default() {
    let c = Color::default();
    assert_eq!((c.r, c.g, c.b, c.a), (0.0, 0.0, 0.0, 0.5));
}

#[test]
fn style_default() {
    let s = Style::default();
    assert_eq!(s.font_size, 12);
    assert_eq!(s.font_name, "Sans");
    assert_eq!(s.border_weight, 1.0);
}

#[test]
fn hex_to_color_valid_with_hash() {
    let c = Color::hex_to_color("#66666680".to_string()).unwrap();
    assert!((c.r - 0.4).abs() < 0.01);
    assert!((c.g - 0.4).abs() < 0.01);
    assert!((c.b - 0.4).abs() < 0.01);
    assert!((c.a - 0.5).abs() < 0.01);
}

#[test]
fn hex_to_color_valid_without_hash() {
    let c = Color::hex_to_color("000000ff".to_string()).unwrap();
    assert_eq!((c.r, c.g, c.b, c.a), (0.0, 0.0, 0.0, 1.0));
}

#[test]
fn hex_to_color_white() {
    let c = Color::hex_to_color("#ffffffff".to_string()).unwrap();
    assert_eq!((c.r, c.g, c.b, c.a), (1.0, 1.0, 1.0, 1.0));
}

#[test]
fn hex_to_color_wrong_length() {
    let err = Color::hex_to_color("#fff".to_string()).unwrap_err();
    assert!(matches!(err, ColorError::InvalidColorFormat(_)));
}

#[test]
fn hex_to_color_invalid_chars() {
    let err = Color::hex_to_color("#zzzzzzzz".to_string()).unwrap_err();
    assert!(matches!(err, ColorError::InvalidColorFormat(_)));
}

#[test]
fn hex_to_color_error_message() {
    let err = Color::hex_to_color("#fff".to_string()).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Invalid color format `#fff`, expected `#rrggbbaa/rrggbbaa`"
    );
}
