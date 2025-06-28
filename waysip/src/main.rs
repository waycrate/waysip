use clap::Parser;
use libwaysip::{Color, Position, SelectionType, Size, WaySip};
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

    /// Set border weight.
    #[arg(short = 'F', value_name = "string")]
    font_name: Option<String>,

    /// Set fomt size.
    #[arg(short = 'S', value_name = "integer")]
    font_size: Option<i32>,

    /// Set border weight.
    #[arg(short = 'w', value_name = "float")]
    border_weight: Option<String>,

    /// Select a single point.
    #[arg(short = 'p', conflicts_with_all = ["dimensions", "screen", "output"])]
    point: bool,

    /// Display dimensions of selection.
    #[arg(short = 'd', conflicts_with_all = ["point", "screen", "output"])]
    dimensions: bool,

    /// Get screen information
    #[arg(short = 'i', conflicts_with_all = ["point", "dimensions", "output"])]
    screen: bool,

    /// Select a display output.
    #[arg(short = 'o', conflicts_with_all = ["point", "dimensions", "screen"])]
    output: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str("trace")?)
        .with_writer(std::io::stderr)
        .init();

    let mut args = Args::parse();

    let mut run_selection = |sel: SelectionType| {
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

    if args.point {
        let info = run_selection(SelectionType::Point);
        let Position { x, y } = info.left_top_point();
        println!("{x},{y} 1x1");
    }
    if args.dimensions {
        let info = run_selection(SelectionType::Area);
        let Position { x, y } = info.left_top_point();
        let width = info.width();
        let height = info.height();
        println!("{x},{y} {width}x{height}",);
    }
    if args.screen {
        let info = run_selection(SelectionType::Screen);
        let screen_info = info.selected_screen_info();
        let Size {
            width: w,
            height: h,
        } = screen_info.get_size();
        let name = screen_info.get_name();
        let description = screen_info.get_description();
        let Size {
            width: wl_w,
            height: wl_h,
        } = screen_info.get_wloutput_size();

        println!("Screen : {name} {description}");
        println!("logic_width: {w}, logic_height: {h}");
        println!("width: {wl_w}, height: {wl_h}");
    }
    if args.output {
        let info = run_selection(SelectionType::Screen);
        let screen_info = info.selected_screen_info();
        let Position { x, y } = screen_info.get_position();
        let Size { width, height } = screen_info.get_size();
        println!("{x},{y} {width}x{height}",);
    }

    Ok(())
}
