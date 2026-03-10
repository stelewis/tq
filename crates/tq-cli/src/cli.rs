use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

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
    #[arg(long = "target")]
    pub target_names: Vec<String>,
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
    pub max_test_file_non_blank_lines: Option<u64>,
    #[arg(long, value_enum)]
    pub qualifier_strategy: Option<QualifierStrategyArg>,
    #[arg(long = "allowed-qualifier")]
    pub allowed_qualifiers: Vec<String>,
    #[command(flatten)]
    pub init_module_args: InitModuleArgs,
    #[arg(long = "select")]
    pub select_rules: Vec<String>,
    #[arg(long = "ignore")]
    pub ignore_rules: Vec<String>,
    #[arg(long)]
    pub exit_zero: bool,
    #[arg(long)]
    pub show_suggestions: bool,
}

#[derive(Debug, clap::Args)]
pub struct InitModuleArgs {
    #[arg(
        long = "ignore-init-modules",
        action = ArgAction::SetTrue,
        conflicts_with = "no_ignore_init_modules"
    )]
    pub ignore_init_modules: bool,
    #[arg(long = "no-ignore-init-modules", action = ArgAction::SetTrue)]
    pub no_ignore_init_modules: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum QualifierStrategyArg {
    None,
    AnySuffix,
    Allowlist,
}
