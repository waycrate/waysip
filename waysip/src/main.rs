use atty::Stream;
use clap::Parser;
use libwaysip::{AreaInfo, BoxInfo, Color, Position, SelectionType, Size, WaySip};
use std::io::Read;
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "waysip")]
#[command(about="Wayland native area picker", long_about = None)]
struct Args {
    /// Set background color.
    #[arg(short = 'b', value_name = "#rrggbbaa/rrggbbaa")]
    background: Option<String>,

    /// Set border and text color.
    #[arg(short = 'c', value_name = "#rrggbbaa/rrggbbaa")]
    border_color: Option<String>,

    /// Set selection color.
    #[arg(short = 's', value_name = "#rrggbbaa/rrggbbaa")]
    selection_color: Option<String>,

    /// Set option box color.
    #[arg(short = 'B', value_name = "#rrggbbaa/rrggbbaa")]
    box_color: Option<String>,

    /// Set the font family for the dimensions.
    #[arg(short = 'F', value_name = "string")]
    font_name: Option<String>,

    /// Set font size.
    #[arg(short = 'S', value_name = "integer")]
    font_size: Option<i32>,

    /// Set border weight.
    #[arg(short = 'w', value_name = "float")]
    border_weight: Option<String>,

    /// Set output format.
    #[arg(short = 'f', value_name = "string", default_value = "%x,%y %wx%h\n")]
    format: String,

    /// Select a single point.
    #[arg(short = 'p', conflicts_with_all = ["screen", "dimensions", "output", "boxes"])]
    point: bool,

    /// Display dimensions of selection.
    #[arg(short = 'd', conflicts_with_all = ["point", "screen", "output", "boxes"])]
    dimensions: bool,

    /// Get screen information
    #[arg(short = 'i', conflicts_with_all = ["point", "dimensions", "output", "boxes"])]
    screen: bool,

    /// Select a display output.
    #[arg(short = 'o', conflicts_with_all = ["point", "dimensions", "screen", "boxes"])]
    output: bool,

    /// Restrict selection to predefined boxes.
    #[arg(short = 'r', conflicts_with_all = ["point", "dimensions", "output" , "screen"])]
    boxes: bool,

    /// Force aspect ratio.
    #[arg(short = 'a', value_name = "width:height", conflicts_with_all = ["point", "screen", "output", "boxes"])]
    aspect_ratio: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str("trace")?)
        .with_writer(std::io::stderr)
        .init();

    let mut args = Args::parse();

    let mut run_selection = |sel: SelectionType, boxes: Option<Vec<BoxInfo>>| {
        let mut builder = WaySip::new().with_selection_type(sel);

        if let Some(color) = args.background.take() {
            builder =
                builder.with_background_color(Color::hex_to_color(color).unwrap_or_else(|e| {
                    eprintln!("Err: {e}");
                    std::process::exit(1);
                }));
        }
        if let Some(color) = args.border_color.take() {
            builder =
                builder.with_foreground_color(Color::hex_to_color(color).unwrap_or_else(|e| {
                    eprintln!("Err: {e}");
                    std::process::exit(1);
                }));
        }
        if let Some(color) = args.selection_color.take() {
            builder =
                builder.with_border_text_color(Color::hex_to_color(color).unwrap_or_else(|e| {
                    eprintln!("Err: {e}");
                    std::process::exit(1);
                }));
        }
        if let Some(color) = args.box_color.take() {
            builder = builder.with_box_color(Color::hex_to_color(color).unwrap_or_else(|e| {
                eprintln!("Err: {e}");
                std::process::exit(1);
            }));
        }
        if let Some(border_weight) = args.border_weight.take() {
            let bw = border_weight.parse::<f64>().unwrap_or_else(|_| {
                eprintln!("Invalid border weight, use -w <n> to set it");
                std::process::exit(1);
            });
            builder = builder.with_border_weight(bw);
        }
        if let Some(font_size) = args.font_size {
            builder = builder.with_font_size(font_size);
        }
        if let Some(font_name) = args.font_name.take() {
            builder = builder.with_font_name(font_name);
        }
        if let Some(boxes) = boxes {
            builder = builder.with_predefined_boxes(boxes);
        }
        if let Some(aspect_ratio) = args.aspect_ratio.take() {
            let parts: Vec<&str> = aspect_ratio.split(':').collect();
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
    };

    let fmt = if args.screen {
        "Screen : %o %d\nlogic_width: %w, logic_height: %h\nwidth: %L, height: %T".to_string()
    } else {
        args.format
    };

    if args.point {
        let info = run_selection(SelectionType::Point, None);
        print!("{}", apply_format(&info, &fmt, false));
    }
    if args.dimensions {
        let info = run_selection(SelectionType::Area, None);
        print!("{}", apply_format(&info, &fmt, false));
    }
    if args.output || args.screen {
        let info = run_selection(SelectionType::Screen, None);
        print!("{}", apply_format(&info, &fmt, true));
    }
    if args.boxes {
        if atty::is(Stream::Stdin) {
            eprintln!("No piped stdin, please pipe a list of boxes to stdin");
            std::process::exit(1);
        }
        let mut input_string = String::new();
        std::io::stdin()
            .read_to_string(&mut input_string)
            .expect("Failed to read stdin");

        if input_string.trim().is_empty() {
            eprintln!(
                "Stdin is empty, please provide a list of boxes in the format `x,y WIDTHxHEIGHT`"
            );
            std::process::exit(1);
        }
        let boxes_strings: Vec<&str> = input_string.lines().collect();
        let boxes: Vec<BoxInfo> = boxes_strings
            .iter()
            .map(|s| BoxInfo::get_box_from_str(s))
            .collect::<Result<Vec<BoxInfo>, _>>()
            .unwrap_or_else(|e| {
                eprintln!("Err: {e}");
                std::process::exit(1);
            });
        let info = run_selection(SelectionType::PredefinedBoxes, Some(boxes));
        print!("{}", apply_format(&info, &fmt, false));
    }

    Ok(())
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
