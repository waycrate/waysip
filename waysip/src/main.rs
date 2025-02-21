use clap::Parser;
use libwaysip::{SelectionType, get_area};
use std::str::FromStr;

#[derive(Debug, Parser)]
#[command(name = "waysip")]
#[command(about="Wayland native area picker", long_about = None)]
enum Cli {
    #[command(short_flag = 'p')]
    Point,
    #[command(short_flag = 'd')]
    Dimensions,
    #[command(short_flag = 's')]
    Screen,
    #[command(short_flag = 'o')]
    Output,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::from_str("trace")?)
        .with_writer(std::io::stderr)
        .init();

    let args = Cli::parse();

    // TODO: Enable tracing
    // TODO: Make errors go through the cli into output

    macro_rules! get_info {
        ($x: expr) => {
            match get_area(None, $x) {
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

    match args {
        Cli::Point => {
            let info = get_info!(SelectionType::Point);
            let (x, y) = info.left_top_point();
            println!("{x},{y} 1x1");
        }
        Cli::Dimensions => {
            let info = get_info!(SelectionType::Area);
            let (x, y) = info.left_top_point();
            let width = info.width();
            let height = info.height();
            println!("{x},{y} {width}x{height}",);
        }
        Cli::Screen => {
            let info = get_info!(SelectionType::Screen);
            let screen_info = info.selected_screen_info();
            let (w, h) = screen_info.get_size();
            let name = screen_info.get_name();
            let description = screen_info.get_description();
            let wlinfo = screen_info.get_outputinfo();
            let (wl_w, wl_h) = wlinfo.get_size();
            println!("Screen : {name} {description}");
            println!("logic_width: {w}, logic_height: {h}");
            println!("width: {wl_w}, height: {wl_h}");
        }
        Cli::Output => {
            let info = get_info!(SelectionType::Screen);
            let screen_info = info.selected_screen_info();
            let (x, y) = screen_info.get_position();
            let (width, height) = screen_info.get_size();
            println!("{x},{y} {width}x{height}",);
        }
    }

    Ok(())
}
