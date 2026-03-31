mod cli;
#[cfg(feature = "logger")]
mod logger;

use clap::Parser;
use cli::Cli;
use libwaysip::{AreaInfo, BoxInfo, Color, Position, SelectionType, Size, WaySip};
use std::io::{IsTerminal, Read};

/// Interactive selection mode derived from CLI flags (excluding `--boxes`, which is handled separately).
#[derive(Clone, Copy)]
enum SelectionDispatch {
    Point,
    /// Single click = output, drag = dimensions.
    DimensionsOrOutput,
    Area,
    Screen,
}

impl SelectionDispatch {
    fn from_cli(args: &Cli) -> Option<Self> {
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

    fn selection_type(self) -> SelectionType {
        match self {
            Self::Point => SelectionType::Point,
            Self::DimensionsOrOutput => SelectionType::DimensionsOrOutput,
            Self::Area => SelectionType::Area,
            Self::Screen => SelectionType::Screen,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Cli::parse();

    #[cfg(feature = "logger")]
    logger::setup(&args);

    let fmt = resolve_output_format(&mut args);

    if args.boxes {
        let boxes = read_boxes_from_stdin();
        let info = run_selection(&mut args, SelectionType::PredefinedBoxes, Some(boxes));
        print!("{}", apply_format(&info, &fmt, false));
    } else if let Some(mode) = SelectionDispatch::from_cli(&args) {
        let info = run_selection(&mut args, mode.selection_type(), None);
        let use_screen_format = match mode {
            SelectionDispatch::DimensionsOrOutput => {
                matches!(info.effective_selection_type, Some(SelectionType::Screen))
            }
            SelectionDispatch::Screen => true,
            SelectionDispatch::Point | SelectionDispatch::Area => false,
        };
        print!("{}", apply_format(&info, &fmt, use_screen_format));
    }

    Ok(())
}

fn parse_hex_color(s: String) -> Color {
    Color::hex_to_color(s).unwrap_or_else(|e| {
        eprintln!("Err: {e}");
        std::process::exit(1);
    })
}

fn parse_aspect_ratio(s: String) -> (f64, f64) {
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

fn run_selection(args: &mut Cli, sel: SelectionType, boxes: Option<Vec<BoxInfo>>) -> AreaInfo {
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

fn resolve_output_format(args: &mut Cli) -> String {
    match args.screen {
        true => {
            "Screen : %o %d\nlogic_width: %w, logic_height: %h\nwidth: %L, height: %T".to_string()
        }
        false => std::mem::take(&mut args.format),
    }
}

fn read_boxes_from_stdin() -> Vec<BoxInfo> {
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

fn apply_format(info: &AreaInfo, fmt: &str, screen: bool) -> String {
    let screen_info = info.selected_screen_info();
    let Position { x: sx, y: sy } = screen_info.get_position();
    let Size {
        width: sw,
        height: sh,
    } = screen_info.get_size();
    let Size {
        width: wl_w,
        height: wl_h,
    } = screen_info.get_wloutput_size();

    let (x, y, width, height) = if !screen {
        let Position { x, y } = info.left_top_point();
        let w = info.width().max(1);
        let h = info.height().max(1);
        (x, y, w, h)
    } else {
        let w = sw.max(1);
        let h = sh.max(1);
        (sx, sy, w, h)
    };

    let rel_x = x.saturating_sub(sx);
    let rel_y = y.saturating_sub(sy);
    let rel_width = width.min(sw.saturating_sub(rel_x));
    let rel_height = height.min(sh.saturating_sub(rel_y));

    let out_name = screen_info.get_name();
    let out_description = screen_info.get_description();

    let mut out = String::with_capacity(fmt.len() * 2);
    let mut chars = fmt.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.next().unwrap_or('%') {
                '%' => out.push('%'),
                'x' => out.push_str(&x.to_string()),
                'y' => out.push_str(&y.to_string()),
                'w' => out.push_str(&width.to_string()),
                'h' => out.push_str(&height.to_string()),
                'X' => out.push_str(&rel_x.to_string()),
                'Y' => out.push_str(&rel_y.to_string()),
                'W' => out.push_str(&rel_width.to_string()),
                'H' => out.push_str(&rel_height.to_string()),
                'o' => out.push_str(out_name),
                'l' => out.push_str(out_name),
                'd' => out.push_str(out_description),
                // Length
                'L' => out.push_str(&wl_w.to_string()),
                // Tall
                'T' => out.push_str(&wl_h.to_string()),
                other => out.push(other),
            }
        } else if c == '\\' {
            match chars.next().unwrap_or('\\') {
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                other => out.push(other),
            }
        } else {
            out.push(c);
        }
    }
    out
}
