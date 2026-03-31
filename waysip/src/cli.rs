use clap::Parser;

#[derive(Parser)]
#[command(name = "waysip")]
#[command(about="Wayland native area picker", long_about = None)]
#[command(version)]
pub(crate) struct Cli {
    /// Set background color.
    #[arg(short = 'b', value_name = "#rrggbbaa/rrggbbaa")]
    pub(crate) background: Option<String>,

    /// Set border and text color.
    #[arg(short = 'c', value_name = "#rrggbbaa/rrggbbaa")]
    pub(crate) border_color: Option<String>,

    /// Set selection color.
    #[arg(short = 's', value_name = "#rrggbbaa/rrggbbaa")]
    pub(crate) selection_color: Option<String>,

    /// Set option box color.
    #[arg(short = 'B', value_name = "#rrggbbaa/rrggbbaa")]
    pub(crate) box_color: Option<String>,

    /// Set the font family for the dimensions.
    #[arg(short = 'F', value_name = "string")]
    pub(crate) font_name: Option<String>,

    /// Set font size.
    #[arg(short = 'S', value_name = "integer")]
    pub(crate) font_size: Option<i32>,

    /// Set border weight.
    #[arg(short = 'w', value_name = "float")]
    pub(crate) border_weight: Option<String>,

    /// Set output format.
    #[arg(short = 'f', value_name = "string", default_value = "%x,%y %wx%h\n")]
    pub(crate) format: String,

    /// Select a single point.
    #[arg(short = 'p', conflicts_with_all = ["screen", "dimensions", "output", "boxes"])]
    pub(crate) point: bool,

    /// Display dimensions of selection.
    #[arg(short = 'd', conflicts_with_all = ["point", "screen", "boxes"])]
    pub(crate) dimensions: bool,

    /// Get screen information
    #[arg(short = 'i', conflicts_with_all = ["point", "dimensions", "output", "boxes"])]
    pub(crate) screen: bool,

    /// Select a display output.
    #[arg(short = 'o', conflicts_with_all = ["point", "screen", "boxes"])]
    pub(crate) output: bool,

    /// Restrict selection to predefined boxes.
    #[arg(short = 'r', conflicts_with_all = ["point", "dimensions", "output" , "screen"])]
    pub(crate) boxes: bool,

    /// Force aspect ratio.
    #[arg(short = 'a', value_name = "width:height", conflicts_with_all = ["point", "screen", "output", "boxes"])]
    pub(crate) aspect_ratio: Option<String>,
}
