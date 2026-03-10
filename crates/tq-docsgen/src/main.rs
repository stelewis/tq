use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "tq-docsgen", about = "Generate tq documentation artifacts")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Generate(GenerateArgs),
}

#[derive(Debug, clap::Args)]
struct GenerateArgs {
    #[arg(value_enum, default_value_t = DocsTarget::All)]
    target: DocsTarget,
    #[arg(long, default_value = ".")]
    workspace_root: PathBuf,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum DocsTarget {
    #[default]
    All,
    Cli,
    Config,
    Rules,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Generate(args) => match args.target {
            DocsTarget::All => tq_docsgen::generate_all(&args.workspace_root),
            DocsTarget::Cli => tq_docsgen::generate_cli_docs(&args.workspace_root),
            DocsTarget::Config => tq_docsgen::generate_config_examples(&args.workspace_root),
            DocsTarget::Rules => tq_docsgen::generate_rules_docs(&args.workspace_root),
        },
    };

    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
