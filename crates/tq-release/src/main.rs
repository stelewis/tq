use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tq_release::{ReleaseError, RuntimeDependencyChange};

#[derive(Debug, Parser)]
#[command(name = "tq-release", about = "Run tq release policy checks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(name = "check-runtime-deps")]
    RuntimeDeps(CheckRuntimeDepsArgs),
    #[command(name = "verify-artifact-contents")]
    ArtifactContents(VerifyArtifactContentsArgs),
    #[command(name = "verify-dependabot")]
    Dependabot(VerifyDependabotArgs),
    #[command(name = "verify-release-policy")]
    ReleasePolicy(VerifyReleasePolicyArgs),
    #[command(name = "verify-workspace-version")]
    WorkspaceVersion(VerifyWorkspaceVersionArgs),
}

#[derive(Debug, clap::Args)]
struct CheckRuntimeDepsArgs {
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
    #[arg(long)]
    base_ref: String,
    #[arg(long)]
    head_ref: String,
}

#[derive(Debug, clap::Args)]
struct VerifyArtifactContentsArgs {
    #[arg(long, default_value = "dist")]
    dist_dir: PathBuf,
    #[arg(long = "forbidden-prefix")]
    forbidden_prefixes: Vec<String>,
}

#[derive(Debug, clap::Args)]
struct VerifyDependabotArgs {
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
}

#[derive(Debug, clap::Args)]
struct VerifyReleasePolicyArgs {
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
}

#[derive(Debug, clap::Args)]
struct VerifyWorkspaceVersionArgs {
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::RuntimeDeps(args) => report_runtime_deps(&args),
        Command::ArtifactContents(args) => tq_release::verify_artifact_contents(
            &args.dist_dir,
            if args.forbidden_prefixes.is_empty() {
                None
            } else {
                Some(args.forbidden_prefixes)
            },
        ),
        Command::Dependabot(args) => tq_release::verify_dependabot(&args.repo_root),
        Command::ReleasePolicy(args) => tq_release::verify_release_policy(&args.repo_root),
        Command::WorkspaceVersion(args) => tq_release::verify_workspace_version(&args.repo_root),
    };

    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn report_runtime_deps(args: &CheckRuntimeDepsArgs) -> Result<(), ReleaseError> {
    let change =
        tq_release::check_runtime_dep_changes(&args.repo_root, &args.base_ref, &args.head_ref)?;

    match change {
        RuntimeDependencyChange::Changed => println!(
            "Runtime dependency changes detected in the shipped CLI path. \
             Commit the update as `fix:` (or `feat:` if it widens behavior) so the next \
             release includes it."
        ),
        RuntimeDependencyChange::Unchanged => {
            println!("No runtime dependency changes in the shipped CLI path.");
        }
    }

    Ok(())
}
