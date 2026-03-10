use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "tq",
    bin_name = "tq",
    author,
    version,
    about = "Test quality toolkit"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Check(CheckArgs),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Text
    }
}

#[derive(Debug, clap::Args)]
pub struct CheckArgs {
    #[arg(long, value_enum, default_value = "text")]
    pub output_format: OutputFormat,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub isolated: bool,
    #[arg(long)]
    pub show_suggestions: bool,
}
