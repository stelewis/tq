use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "tq-release", about = "Run tq release policy checks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    VerifyArtifactContents(VerifyArtifactContentsArgs),
}

#[derive(Debug, clap::Args)]
struct VerifyArtifactContentsArgs {
    #[arg(long, default_value = "dist")]
    dist_dir: PathBuf,
    #[arg(long = "forbidden-prefix")]
    forbidden_prefixes: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::VerifyArtifactContents(args) => tq_release::verify_artifact_contents(
            &args.dist_dir,
            if args.forbidden_prefixes.is_empty() {
                None
            } else {
                Some(args.forbidden_prefixes)
            },
        ),
    };

    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
