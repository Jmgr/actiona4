use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about = "Screen region/position selector")]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Clone, Subcommand)]
pub enum Mode {
    /// Select a rectangular region. Outputs "x y w h" to stdout.
    Rect,
    /// Click to select a position. Outputs "x y" to stdout.
    Pos {
        /// Magnifier zoom factor
        #[arg(short, long, default_value_t = 10.0)]
        zoom: f32,
    },
}
