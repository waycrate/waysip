use clap::Parser;
use libwaysip::get_area;

#[derive(Debug, Parser)]
#[command(name = "waysip")]
#[command(about="a tool like slurp, but in rust", long_about = None)]
enum Cli {
    #[command(short_flag = 'p')]
    Point,
    #[command(short_flag = 'd')]
    Dimesions,
}

fn main() {
    let args = Cli::parse();

    let info = match get_area() {
        Ok(Some(info)) => info,
        Ok(None) => {
            eprintln!("Get None, you cancel it");
            return;
        }
        Err(e) => {
            eprintln!("Error,{e}");
            return;
        }
    };

    match args {
        Cli::Point => {
            let (x, y) = info.left_top_point();
            println!("{},{} 1x1", x as i32, y as i32);
        }
        Cli::Dimesions => {
            let (x, y) = info.left_top_point();
            let width = info.width();
            let height = info.height();
            println!(
                "{},{} {}x{}",
                x as i32, y as i32, width as i32, height as i32
            );
        }
    }
}
