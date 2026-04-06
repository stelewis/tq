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
    #[command(name = "verify-artifact-contents")]
    ArtifactContents(VerifyArtifactContentsArgs),
    #[command(name = "verify-dependabot")]
    Dependabot(VerifyDependabotArgs),
    #[command(name = "verify-release-policy")]
    ReleasePolicy(VerifyReleasePolicyArgs),
    #[command(name = "sync-workspace-dependency-versions")]
    SyncWorkspaceDependencyVersions(SyncWorkspaceDependencyVersionsArgs),
    #[command(name = "verify-workspace-version")]
    WorkspaceVersion(VerifyWorkspaceVersionArgs),
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

#[derive(Debug, clap::Args)]
struct SyncWorkspaceDependencyVersionsArgs {
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
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
        Command::SyncWorkspaceDependencyVersions(args) => {
            tq_release::sync_workspace_dependency_versions(&args.repo_root)
        }
        Command::WorkspaceVersion(args) => tq_release::verify_workspace_version(&args.repo_root),
    };

    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
