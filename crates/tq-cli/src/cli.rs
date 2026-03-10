use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "tq",
    bin_name = "tq",
    visible_alias = "tqlint",
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Debug, clap::Args)]
pub struct CheckArgs {
    #[arg(
        long,
        value_enum,
        default_value = "text",
        help = "Select output format."
    )]
    pub output_format: OutputFormat,
    #[arg(
        long,
        help = "Use this pyproject file instead of discovered configuration."
    )]
    pub config: Option<PathBuf>,
    #[arg(long, help = "Ignore discovered configuration files.")]
    pub isolated: bool,
    #[arg(long = "target", help = "Run only listed target names.")]
    pub target_names: Vec<String>,
    #[arg(
        long,
        value_parser = clap::value_parser!(u64).range(1..),
        help = "Maximum non-blank, non-comment lines per test file."
    )]
    pub max_test_file_non_blank_lines: Option<u64>,
    #[arg(
        long,
        value_enum,
        help = "Module-name qualifier policy for qualified test files."
    )]
    pub qualifier_strategy: Option<QualifierStrategyArg>,
    #[arg(
        long = "allowed-qualifier",
        help = "Allowed qualifier suffix for allowlist strategy."
    )]
    pub allowed_qualifiers: Vec<String>,
    #[command(flatten)]
    pub init_module_args: InitModuleArgs,
    #[arg(long = "select", help = "Only run selected rule IDs.")]
    pub select_rules: Vec<String>,
    #[arg(long = "ignore", help = "Skip listed rule IDs.")]
    pub ignore_rules: Vec<String>,
    #[arg(long, help = "Always exit with code 0 regardless of findings.")]
    pub exit_zero: bool,
    #[arg(long, help = "Render remediation suggestions in diagnostics output.")]
    pub show_suggestions: bool,
}

#[derive(Debug, clap::Args)]
pub struct InitModuleArgs {
    #[arg(
        long = "ignore-init-modules",
        action = ArgAction::SetTrue,
        conflicts_with = "no_ignore_init_modules",
        help = "Ignore __init__.py modules in mapping checks."
    )]
    pub ignore_init_modules: bool,
    #[arg(
        long = "no-ignore-init-modules",
        action = ArgAction::SetTrue,
        help = "Include __init__.py modules in mapping checks."
    )]
    pub no_ignore_init_modules: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum QualifierStrategyArg {
    None,
    AnySuffix,
    Allowlist,
}
