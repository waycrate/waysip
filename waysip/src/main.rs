mod cli;
#[cfg(feature = "logger")]
mod logger;
mod settings;
mod utils;

use clap::Parser;
use cli::Cli;
use libwaysip::SelectionType;
use settings::{SelectionDispatch, read_boxes_from_stdin, resolve_output_format, run_selection};
use utils::apply_format;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Cli::parse();

    #[cfg(feature = "completions")]
    if let Some(shell) = args.completions {
        utils::print_completions(shell);
        return Ok(());
    }

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
