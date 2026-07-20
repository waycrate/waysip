mod cli;
#[cfg(feature = "freeze")]
mod freeze;
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
        #[cfg(feature = "benchmark")]
        if args.bench {
            print_bench_results(SelectionType::PredefinedBoxes, &info.timestamps_total);
        }
        print!("{}", apply_format(&info, &fmt, false));
    } else if let Some(mode) = SelectionDispatch::from_cli(&args) {
        let selection_type = mode.selection_type();
        let info = run_selection(&mut args, selection_type, None);
        #[cfg(feature = "benchmark")]
        if args.bench {
            print_bench_results(selection_type, &info.timestamps_total);
        }
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

#[cfg(feature = "benchmark")]
fn print_bench_results(selection_type: SelectionType, timestamps: &[u32]) {
    if timestamps.len() < 2 {
        eprintln!("Bench err: no data");
        return;
    }

    let first = timestamps[0];
    let mut rel: Vec<u32> = timestamps.iter().map(|&ts| ts - first).collect();
    let raw_duration = *rel.last().unwrap();

    let is_drag = matches!(
        selection_type,
        SelectionType::Area | SelectionType::DimensionsOrOutput
    );
    let trimmed_duration = if is_drag {
        let to_cut = (raw_duration % 1000) / 2;
        eprintln!("raw duration: {raw_duration}ms");
        eprintln!("trimming {to_cut}ms from each end");
        let cut_start = to_cut;
        let cut_end_threshold = raw_duration.saturating_sub(to_cut);
        rel.retain(|&ts| ts >= cut_start && ts <= cut_end_threshold);
        if rel.len() < 2 {
            eprintln!("bench err: not enough frames after trim");
            return;
        }
        let new_first = rel[0];
        for ts in rel.iter_mut() {
            *ts -= new_first;
        }
        *rel.last().unwrap()
    } else {
        raw_duration
    };

    let frames = rel.len();
    let frametimes: Vec<u32> = rel.windows(2).map(|w| w[1] - w[0]).collect();

    let sum_ms: u64 = frametimes.iter().map(|&f| f as u64).sum();
    let avg_ft = sum_ms as f64 / frametimes.len() as f64;
    let min_ft = *frametimes.iter().min().unwrap();
    let max_ft = *frametimes.iter().max().unwrap();
    let avg_fps = if trimmed_duration > 0 {
        frames as f64 / (trimmed_duration as f64 / 1000.0)
    } else {
        0.0
    };

    eprintln!("  avg fps       : {avg_fps:.1}");
    eprintln!("  avg frametime : {avg_ft:.2}ms");
    eprintln!("  min frametime : {min_ft}ms");
    eprintln!("  max frametime : {max_ft}ms");
    eprintln!("  latencies     : {frametimes:?}");
}
