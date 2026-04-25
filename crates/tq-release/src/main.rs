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
    #[command(name = "verify-pr-release-intent")]
    PrReleaseIntent(VerifyPrReleaseIntentArgs),
    #[command(name = "verify-release-intent")]
    ReleaseIntent(VerifyReleaseIntentArgs),
    #[command(name = "verify-release-policy")]
    ReleasePolicy(VerifyReleasePolicyArgs),
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
struct VerifyPrReleaseIntentArgs {
    #[arg(long = "label")]
    labels: Vec<String>,
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
    #[arg(long)]
    base_ref: String,
    #[arg(long)]
    head_ref: String,
}

#[derive(Debug, clap::Args)]
struct VerifyReleaseIntentArgs {
    #[arg(long = "label")]
    labels: Vec<String>,
    #[arg(long = "changed-file")]
    changed_files: Vec<PathBuf>,
    #[arg(long)]
    version_updated: bool,
    #[arg(long)]
    changelog_updated: bool,
    #[arg(long)]
    runtime_dependency_changed: bool,
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
        Command::ArtifactContents(args) => tq_release::verify_artifact_contents(
            &args.dist_dir,
            if args.forbidden_prefixes.is_empty() {
                None
            } else {
                Some(args.forbidden_prefixes)
            },
        ),
        Command::Dependabot(args) => tq_release::verify_dependabot(&args.repo_root),
        Command::PrReleaseIntent(args) => {
            tq_release::verify_pr_release_intent(tq_release::PrReleaseIntentCheck {
                repo_root: &args.repo_root,
                base_ref: &args.base_ref,
                head_ref: &args.head_ref,
                labels: &args.labels,
            })
        }
        Command::ReleaseIntent(args) => {
            tq_release::verify_release_intent(tq_release::ReleaseIntentCheck {
                labels: &args.labels,
                changed_files: &args.changed_files,
                version_updated: args.version_updated,
                changelog_updated: args.changelog_updated,
                runtime_dependency_changed: args.runtime_dependency_changed,
            })
        }
        Command::ReleasePolicy(args) => tq_release::verify_release_policy(&args.repo_root),
        Command::WorkspaceVersion(args) => tq_release::verify_workspace_version(&args.repo_root),
    };

    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
