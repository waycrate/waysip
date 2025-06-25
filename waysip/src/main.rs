use clap::Parser;
use libwaysip::{Position, SelectionType, Size, WaySip};
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "waysip")]
#[command(about="Wayland native area picker", long_about = None)]
struct Args {
    /// Set background color.
    #[arg(
        short = 'b',
        default_value = "#66666680",
        value_name = "#rrggbbaa/rrggbbaa"
    )]
    background: Option<String>,

    /// Set border and text color.
    #[arg(
        short = 'c',
        default_value = "#ffffffff",
        value_name = "#rrggbbaa/rrggbbaa"
    )]
    border_color: Option<String>,

    /// Set selection color.
    #[arg(
        short = 's',
        default_value = "#00000000",
        value_name = "#rrggbbaa/rrggbbaa"
    )]
    selection_color: Option<String>,

    /// Set border weight.
    #[arg(short = 'F', default_value = "Sans", value_name = "string")]
    font_name: Option<String>,

    /// Set fomt size.
    #[arg(short = 'S', default_value = "12", value_name = "integer")]
    font_size: Option<i32>,

    /// Set border weight.
    #[arg(short = 'w', default_value = "1.0", value_name = "float")]
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

    let args = Args::parse();

    let passing_data: libwaysip::state::PassingData = libwaysip::state::PassingData {
        background_color: hex_to_vec(args.background.unwrap())?,
        foreground_color: hex_to_vec(args.border_color.unwrap())?,
        border_text_color: hex_to_vec(args.selection_color.unwrap())?,
        border_size: match args
            .border_weight
            .as_ref()
            .and_then(|s| s.parse::<f64>().ok())
        {
            Some(weight) => weight,
            None => {
                eprintln!("Invalid border weight, use -w <n> to set it");
                std::process::exit(1);
            }
        },
        font_size: args.font_size.unwrap(),
        font_name: args.font_name.unwrap(),
    };

    macro_rules! get_info {
        ($x: expr) => {
            match WaySip::new()
                .with_selection_type($x, passing_data.clone())
                .get()
            {
                Ok(Some(info)) => info,
                Ok(None) => {
                    eprintln!("Get None, you cancel it");
                    // TODO: Have proper error types
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Error,{e}");
                    return Ok(());
                }
            }
        };
    }

    if args.point {
        let info = get_info!(SelectionType::Point);
        let Position { x, y } = info.left_top_point();
        println!("{x},{y} 1x1");
    }
    if args.dimensions {
        let info = get_info!(SelectionType::Area);
        let Position { x, y } = info.left_top_point();
        let width = info.width();
        let height = info.height();
        println!("{x},{y} {width}x{height}",);
    }
    if args.screen {
        let info = get_info!(SelectionType::Screen);
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
        let info = get_info!(SelectionType::Screen);
        let screen_info = info.selected_screen_info();
        let Position { x, y } = screen_info.get_position();
        let Size { width, height } = screen_info.get_size();
        println!("{x},{y} {width}x{height}",);
    }

    Ok(())
}

fn hex_to_vec(colors: String) -> Result<[f64; 4], Box<dyn std::error::Error>> {
    let bg_color = colors.trim_start_matches('#');

    if bg_color.len() != 8 || !bg_color.chars().all(|c| c.is_ascii_hexdigit()) {
        eprintln!("Invalid background color format, expected #rrggbbaa");
        std::process::exit(1);
    }
    let mut rgba = [0.0f64; 4];
    for i in 0..4 {
        let byte = u8::from_str_radix(&bg_color[i * 2..i * 2 + 2], 16)?;
        rgba[i] = byte as f64 / 255.0;
    }
    Ok(rgba)
}
