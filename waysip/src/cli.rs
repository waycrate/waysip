#[cfg(feature = "completions")]
use clap::ValueEnum;
#[cfg(feature = "logger")]
use tracing::Level;

use clap::{
    Parser,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};

/// Shell variants for completion generation.
#[cfg(feature = "completions")]
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    Pwsh,
    Zsh,
    Nushell,
}

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser)]
#[command(version, about, styles=get_styles())]
pub struct Cli {
    // ─── Shell completions ────────────────────────────────────────────────────
    /// Generate shell completions and print to stdout.
    /// Example: waysip --completions fish | source
    #[cfg(feature = "completions")]
    #[arg(long, value_name = "SHELL", exclusive = true, verbatim_doc_comment)]
    pub completions: Option<Shell>,

    // ─── Colors ───────────────────────────────────────────────────────────────
    /// Set background color.
    #[arg(short = 'b', value_name = "#rrggbbaa/rrggbbaa")]
    pub background: Option<String>,

    /// Set border and text color.
    #[arg(short = 'c', value_name = "#rrggbbaa/rrggbbaa")]
    pub border_color: Option<String>,

    /// Set selection color.
    #[arg(short = 's', value_name = "#rrggbbaa/rrggbbaa")]
    pub selection_color: Option<String>,

    /// Set option box color.
    #[arg(short = 'B', value_name = "#rrggbbaa/rrggbbaa")]
    pub box_color: Option<String>,

    // ─── Typography & border ─────────────────────────────────────────────────
    /// Set the font family for the dimensions.
    #[arg(short = 'F', value_name = "string")]
    pub font_name: Option<String>,

    /// Set font size.
    #[arg(short = 'S', value_name = "integer")]
    pub font_size: Option<i32>,

    /// Set border weight.
    #[arg(short = 'w', value_name = "float")]
    pub border_weight: Option<String>,

    // ─── Output format ───────────────────────────────────────────────────────
    /// Set output format.
    #[arg(short = 'f', value_name = "string", default_value = "%x,%y %wx%h\n")]
    pub format: String,

    // ─── Selection mode ──────────────────────────────────────────────────────
    /// Select a single point.
    #[arg(short = 'p', conflicts_with_all = ["screen", "dimensions", "output", "boxes"])]
    pub point: bool,

    /// Display dimensions of selection.
    #[arg(short = 'd', conflicts_with_all = ["point", "screen", "boxes"])]
    pub dimensions: bool,

    /// Get screen information
    #[arg(short = 'i', conflicts_with_all = ["point", "dimensions", "output", "boxes"])]
    pub screen: bool,

    /// Select a display output.
    #[arg(short = 'o', conflicts_with_all = ["point", "screen", "boxes"])]
    pub output: bool,

    /// Restrict selection to predefined boxes.
    #[arg(short = 'r', conflicts_with_all = ["point", "dimensions", "output", "screen"])]
    pub boxes: bool,

    /// Force aspect ratio.
    #[arg(
        short = 'a',
        value_name = "width:height",
        conflicts_with_all = ["point", "screen", "output", "boxes"]
    )]
    pub aspect_ratio: Option<String>,

    // ─── Global options ───────────────────────────────────────────────────────
    /// Log level written to stderr.
    #[cfg(feature = "logger")]
    #[arg(long)]
    pub log_level: Option<Level>,
}
