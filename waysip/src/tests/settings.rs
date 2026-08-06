use clap::Parser;
use libwaysip::SelectionType;

use crate::cli::Cli;
use crate::settings::*;

fn cli(args: &[&str]) -> Cli {
    Cli::try_parse_from(std::iter::once("waysip").chain(args.iter().copied())).unwrap()
}

#[test]
fn dispatch_point() {
    let args = cli(&["-p"]);
    assert!(matches!(
        SelectionDispatch::from_cli(&args),
        Some(SelectionDispatch::Point)
    ));
}

#[test]
fn dispatch_dimensions_and_output() {
    let args = cli(&["-d", "-o"]);
    assert!(matches!(
        SelectionDispatch::from_cli(&args),
        Some(SelectionDispatch::DimensionsOrOutput)
    ));
}

#[test]
fn dispatch_dimensions_only() {
    let args = cli(&["-d"]);
    assert!(matches!(
        SelectionDispatch::from_cli(&args),
        Some(SelectionDispatch::Area)
    ));
}

#[test]
fn dispatch_output_only() {
    let args = cli(&["-o"]);
    assert!(matches!(
        SelectionDispatch::from_cli(&args),
        Some(SelectionDispatch::Screen)
    ));
}

#[test]
fn dispatch_screen_flag() {
    let args = cli(&["-i"]);
    assert!(matches!(
        SelectionDispatch::from_cli(&args),
        Some(SelectionDispatch::Screen)
    ));
}

#[test]
fn dispatch_none_by_default() {
    let args = cli(&[]);
    assert!(SelectionDispatch::from_cli(&args).is_none());
}

#[test]
fn dispatch_ignores_boxes_flag() {
    let args = cli(&["-r"]);
    assert!(SelectionDispatch::from_cli(&args).is_none());
}

#[test]
fn selection_type_mapping() {
    assert!(matches!(
        SelectionDispatch::Point.selection_type(),
        SelectionType::Point
    ));
    assert!(matches!(
        SelectionDispatch::DimensionsOrOutput.selection_type(),
        SelectionType::DimensionsOrOutput
    ));
    assert!(matches!(
        SelectionDispatch::Area.selection_type(),
        SelectionType::Area
    ));
    assert!(matches!(
        SelectionDispatch::Screen.selection_type(),
        SelectionType::Screen
    ));
}

#[test]
fn resolve_output_format_uses_screen_template() {
    let mut args = cli(&["-i", "-f", "custom"]);
    let fmt = resolve_output_format(&mut args);
    assert!(fmt.starts_with("Screen : %o %d"));
}

#[test]
fn resolve_output_format_takes_custom_format_and_clears_it() {
    let mut args = cli(&["-f", "custom"]);
    let fmt = resolve_output_format(&mut args);
    assert_eq!(fmt, "custom");
    assert_eq!(args.format, "");
}

#[test]
fn resolve_output_format_default_when_unset() {
    let mut args = cli(&[]);
    let fmt = resolve_output_format(&mut args);
    assert_eq!(fmt, "%x,%y %wx%h\n");
}

#[test]
fn parse_hex_color_valid() {
    let color = parse_hex_color("#000000ff".to_string());
    assert_eq!((color.r, color.g, color.b, color.a), (0.0, 0.0, 0.0, 1.0));
}

#[test]
fn parse_aspect_ratio_valid() {
    assert_eq!(parse_aspect_ratio("16:9".to_string()), (16.0, 9.0));
}
