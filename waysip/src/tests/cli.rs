use crate::cli::Cli;
#[cfg(feature = "completions")]
use crate::cli::Shell;
use clap::Parser;
#[cfg(feature = "logger")]
use tracing::Level;

fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("waysip").chain(args.iter().copied()))
}

#[test]
fn defaults_with_no_args() {
    let cli = parse(&[]).unwrap();
    assert!(!cli.point);
    assert!(!cli.dimensions);
    assert!(!cli.screen);
    assert!(!cli.output);
    assert!(!cli.boxes);
    assert!(!cli.edit_selection);
    assert_eq!(cli.format, "%x,%y %wx%h\n");
    assert!(cli.background.is_none());
    assert!(cli.aspect_ratio.is_none());
}

#[test]
fn point_and_dimensions_conflict() {
    assert!(parse(&["-p", "-d"]).is_err());
}

#[test]
fn screen_and_output_conflict() {
    assert!(parse(&["-i", "-o"]).is_err());
}

#[test]
fn point_and_boxes_conflict() {
    assert!(parse(&["-p", "-r"]).is_err());
}

#[test]
fn dimensions_and_output_are_compatible() {
    // dimensions-or-output combined mode is intentionally allowed
    let cli = parse(&["-d", "-o"]).unwrap();
    assert!(cli.dimensions);
    assert!(cli.output);
}

#[test]
fn custom_format_overrides_default() {
    let cli = parse(&["-f", "%x"]).unwrap();
    assert_eq!(cli.format, "%x");
}

#[test]
fn aspect_ratio_value_is_captured() {
    let cli = parse(&["-a", "16:9"]).unwrap();
    assert_eq!(cli.aspect_ratio.as_deref(), Some("16:9"));
}

#[test]
fn edit_selection_key_requires_edit_selection() {
    assert!(parse(&["--edit-selection-key", "15"]).is_err());
}

#[test]
fn edit_selection_with_key_parses() {
    let cli = parse(&["-e", "--edit-selection-key", "15"]).unwrap();
    assert!(cli.edit_selection);
    assert_eq!(cli.edit_selection_key, Some(15));
}

#[test]
fn color_flags_are_captured() {
    let cli = parse(&["-b", "#000000ff", "-c", "#ffffffff"]).unwrap();
    assert_eq!(cli.background.as_deref(), Some("#000000ff"));
    assert_eq!(cli.border_color.as_deref(), Some("#ffffffff"));
}

#[cfg(feature = "logger")]
#[test]
fn log_level_parses() {
    let cli = parse(&["--log-level", "debug"]).unwrap();
    assert_eq!(cli.log_level, Some(Level::DEBUG));
}

#[cfg(feature = "completions")]
#[test]
fn completions_flag_parses_shell() {
    let cli = parse(&["--completions", "bash"]).unwrap();
    assert!(matches!(cli.completions, Some(Shell::Bash)));
}

#[cfg(feature = "completions")]
#[test]
fn completions_is_exclusive() {
    assert!(parse(&["--completions", "bash", "-p"]).is_err());
}
