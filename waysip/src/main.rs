use clap::Parser;
use libwaysip::{WaySipKind, WaysipEv};

#[derive(Debug, Parser)]
#[command(name = "waysip")]
#[command(about="a tool like slurp, but in rust", long_about = None)]
enum Cli {
    #[command(short_flag = 'p')]
    Point,
    #[command(short_flag = 'd')]
    Dimesions,
    #[command(short_flag = 's')]
    Screen,
    #[command(short_flag = 'o')]
    Output,
}

fn main() {
    let args = Cli::parse();

    let waysip = WaysipEv::new().unwrap();
    macro_rules! get_info {
        ($x: expr) => {
            match waysip.get_area($x) {
                Ok(Some(info)) => info,
                Ok(None) => {
                    eprintln!("Get None, you cancel it");
                    return;
                }
                Err(e) => {
                    eprintln!("Error,{e}");
                    return;
                }
            }
        };
    }

    match args {
        Cli::Point => {
            let info = get_info!(WaySipKind::Point);
            let (x, y) = info.left_top_point();
            println!("{x},{y} 1x1");
        }
        Cli::Dimesions => {
            let info = get_info!(WaySipKind::Area);
            let (x, y) = info.left_top_point();
            let width = info.width();
            let height = info.height();
            println!("{x},{y} {width}x{height}",);
        }
        Cli::Screen => {
            let info = get_info!(WaySipKind::Screen);
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
            let info = get_info!(WaySipKind::Screen);
            let screen_info = info.selected_screen_info();
            let (x, y) = screen_info.get_position();
            let (width, height) = screen_info.get_size();
            println!("{x},{y} {width}x{height}",);
        }
    }
}
