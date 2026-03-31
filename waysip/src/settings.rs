//! Runtime behaviour derived from CLI flags.

use std::io::{IsTerminal, Read};

use crate::cli::Cli;
use libwaysip::{AreaInfo, BoxInfo, Color, SelectionType, WaySip};

// ─── Selection dispatch ───────────────────────────────────────────────────────

/// Interactive selection mode from CLI flags (excluding `--boxes`, handled separately).
#[derive(Clone, Copy)]
pub(crate) enum SelectionDispatch {
    Point,
    /// Single click = output, drag = dimensions.
    DimensionsOrOutput,
    Area,
    Screen,
}

impl SelectionDispatch {
    pub(crate) fn from_cli(args: &Cli) -> Option<Self> {
        if args.point {
            Some(Self::Point)
        } else if args.dimensions && args.output {
            Some(Self::DimensionsOrOutput)
        } else if args.dimensions {
            Some(Self::Area)
        } else if args.output || args.screen {
            Some(Self::Screen)
        } else {
            None
        }
    }

    pub(crate) fn selection_type(self) -> SelectionType {
        match self {
            Self::Point => SelectionType::Point,
            Self::DimensionsOrOutput => SelectionType::DimensionsOrOutput,
            Self::Area => SelectionType::Area,
            Self::Screen => SelectionType::Screen,
        }
    }
}

// ─── CLI value parsing ──────────────────────────────────────────────────────

pub(crate) fn parse_hex_color(s: String) -> Color {
    Color::hex_to_color(s).unwrap_or_else(|e| {
        eprintln!("Err: {e}");
        std::process::exit(1);
    })
}

pub(crate) fn parse_aspect_ratio(s: String) -> (f64, f64) {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        eprintln!("Invalid aspect ratio format, use -a <width:height>");
        std::process::exit(1);
    }
    let width = parts[0].parse::<f64>().unwrap_or_else(|_| {
        eprintln!("Invalid width in aspect ratio");
        std::process::exit(1);
    });
    let height = parts[1].parse::<f64>().unwrap_or_else(|_| {
        eprintln!("Invalid height in aspect ratio");
        std::process::exit(1);
    });
    (width, height)
}

// ─── WaySip from CLI ──────────────────────────────────────────────────────────

pub(crate) fn run_selection(
    args: &mut Cli,
    sel: SelectionType,
    boxes: Option<Vec<BoxInfo>>,
) -> AreaInfo {
    let mut builder = WaySip::new().with_selection_type(sel);

    if let Some(color) = args.background.take() {
        builder = builder.with_background_color(parse_hex_color(color));
    }
    if let Some(color) = args.border_color.take() {
        builder = builder.with_foreground_color(parse_hex_color(color));
    }
    if let Some(color) = args.selection_color.take() {
        builder = builder.with_border_text_color(parse_hex_color(color));
    }
    if let Some(color) = args.box_color.take() {
        builder = builder.with_box_color(parse_hex_color(color));
    }
    if let Some(border_weight) = args.border_weight.take() {
        let bw = border_weight.parse::<f64>().unwrap_or_else(|_| {
            eprintln!("Invalid border weight, use -w <n> to set it");
            std::process::exit(1);
        });
        builder = builder.with_border_weight(bw);
    }
    if let Some(font_size) = args.font_size.take() {
        builder = builder.with_font_size(font_size);
    }
    if let Some(font_name) = args.font_name.take() {
        builder = builder.with_font_name(font_name);
    }
    if let Some(boxes) = boxes {
        builder = builder.with_predefined_boxes(boxes);
    }
    if let Some(aspect_ratio) = args.aspect_ratio.take() {
        let (width, height) = parse_aspect_ratio(aspect_ratio);
        builder = builder.with_aspect_ratio(width, height);
    }

    match builder.get() {
        Ok(Some(info)) => info,
        Ok(None) => {
            eprintln!("Selection canceled");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

// ─── Output format string ───────────────────────────────────────────────────

pub(crate) fn resolve_output_format(args: &mut Cli) -> String {
    match args.screen {
        true => {
            "Screen : %o %d\nlogic_width: %w, logic_height: %h\nwidth: %L, height: %T".to_string()
        }
        false => std::mem::take(&mut args.format),
    }
}

// ─── Predefined boxes (stdin) ─────────────────────────────────────────────────

pub(crate) fn read_boxes_from_stdin() -> Vec<BoxInfo> {
    let mut stdio = std::io::stdin();
    if stdio.is_terminal() {
        eprintln!("No piped stdin, please pipe a list of boxes to stdin");
        std::process::exit(1);
    }
    let mut input_string = String::new();
    stdio
        .read_to_string(&mut input_string)
        .expect("Failed to read stdin");

    if input_string.trim().is_empty() {
        eprintln!(
            "Stdin is empty, please provide a list of boxes in the format `x,y WIDTHxHEIGHT`"
        );
        std::process::exit(1);
    }

    input_string
        .lines()
        .map(BoxInfo::get_box_from_str)
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|e| {
            eprintln!("Err: {e}");
            std::process::exit(1);
        })
}
