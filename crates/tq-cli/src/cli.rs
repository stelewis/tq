use std::path::PathBuf;

use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects};
use clap::{Parser, Subcommand, ValueEnum};

const CLI_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Debug, Parser)]
#[command(
    name = "tq",
    bin_name = "tq",
    author,
    version,
    about = "Check Python test layout quality.",
    after_help = "For help with the check command, run `tq check --help`.",
    disable_help_subcommand = true,
    arg_required_else_help = true,
    subcommand_required = true,
    propagate_version = true,
    styles = CLI_STYLES
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run test quality checks against configured targets.
    Check(CheckArgs),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Debug, clap::Args)]
#[command(
    about = "Run test quality checks against configured targets.",
    after_help = "CLI flags override resolved configuration values. Use `--config <PATH>` to pin the input configuration file."
)]
pub struct CheckArgs {
    #[arg(
        long,
        value_name = "PATH",
        help_heading = "Configuration",
        help = "Use this pyproject file instead of discovered configuration."
    )]
    pub config: Option<PathBuf>,
    #[arg(
        long,
        help_heading = "Configuration",
        help = "Ignore discovered configuration files."
    )]
    pub isolated: bool,
    #[arg(
        long = "target",
        value_name = "NAME",
        help_heading = "Configuration",
        help = "Run only listed target names."
    )]
    pub target_names: Vec<String>,
    #[arg(
        long = "init-modules",
        value_enum,
        value_name = "MODE",
        help_heading = "Rule configuration",
        help = "How mapping checks handle __init__.py modules."
    )]
    pub init_modules: Option<InitModuleModeArg>,
    #[arg(
        long,
        value_parser = clap::value_parser!(u64).range(1..),
        value_name = "COUNT",
        help_heading = "Rule configuration",
        help = "Maximum non-blank, non-comment lines per test file."
    )]
    pub max_test_file_non_blank_lines: Option<u64>,
    #[arg(
        long,
        value_enum,
        value_name = "STRATEGY",
        help_heading = "Rule configuration",
        help = "Module-name qualifier policy for qualified test files."
    )]
    pub qualifier_strategy: Option<QualifierStrategyArg>,
    #[arg(
        long = "allowed-qualifier",
        value_name = "SUFFIX",
        help_heading = "Rule configuration",
        help = "Allowed qualifier suffix for allowlist strategy."
    )]
    pub allowed_qualifiers: Vec<String>,
    #[arg(
        long = "select",
        value_name = "RULE_ID",
        help_heading = "Rule selection",
        help = "Only run selected rule IDs."
    )]
    pub select_rules: Vec<String>,
    #[arg(
        long = "ignore",
        value_name = "RULE_ID",
        help_heading = "Rule selection",
        help = "Skip listed rule IDs."
    )]
    pub ignore_rules: Vec<String>,
    #[arg(
        long,
        value_enum,
        default_value = "text",
        value_name = "FORMAT",
        help_heading = "Output",
        help = "Select output format."
    )]
    pub output_format: OutputFormat,
    #[arg(
        long,
        help_heading = "Output",
        help = "Render remediation suggestions in diagnostics output."
    )]
    pub show_suggestions: bool,
    #[arg(
        long,
        help_heading = "Output",
        help = "Always exit with code 0 regardless of findings."
    )]
    pub exit_zero: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum InitModuleModeArg {
    Include,
    Ignore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum QualifierStrategyArg {
    None,
    AnySuffix,
    Allowlist,
}
