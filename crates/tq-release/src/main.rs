use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tq_release::{DevDoctorReport, DevToolStatus, ReleaseError, RuntimeDependencyChange};

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
    #[command(name = "dev")]
    Dev(DevArgs),
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
struct DevArgs {
    #[command(subcommand)]
    command: DevCommand,
}

#[derive(Debug, Subcommand)]
enum DevCommand {
    #[command(name = "audit-latest")]
    AuditLatest(RepoRootArgs),
    #[command(name = "doctor")]
    Doctor(RepoRootArgs),
    #[command(name = "setup")]
    Setup(RepoRootArgs),
    #[command(name = "update")]
    Update(RepoRootArgs),
    #[command(name = "verify-pins")]
    VerifyPins(RepoRootArgs),
}

#[derive(Debug, clap::Args)]
struct RepoRootArgs {
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
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
        Command::Dev(args) => run_dev_command(&args),
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

fn run_dev_command(args: &DevArgs) -> Result<(), ReleaseError> {
    match &args.command {
        DevCommand::AuditLatest(args) => tq_release::audit_latest_dev_dependencies(&args.repo_root),
        DevCommand::Doctor(args) => {
            let report = tq_release::doctor_dev_environment(&args.repo_root)?;
            print_doctor_report(&report);
            if report.is_healthy() {
                Ok(())
            } else {
                Err(ReleaseError::RepositoryPolicyViolation {
                    details: "developer environment does not match .github/dev-tools.toml"
                        .to_owned(),
                })
            }
        }
        DevCommand::Setup(args) => tq_release::setup_dev_environment(&args.repo_root),
        DevCommand::Update(args) => tq_release::update_dev_dependencies(&args.repo_root),
        DevCommand::VerifyPins(args) => tq_release::verify_dev_tool_pins(&args.repo_root),
    }
}

fn print_doctor_report(report: &DevDoctorReport) {
    for check in &report.checks {
        let status = match check.status {
            DevToolStatus::Ok => "ok",
            DevToolStatus::Missing => "missing",
            DevToolStatus::Mismatched => "mismatch",
        };
        let actual = check.actual.as_deref().unwrap_or("not found");
        println!(
            "{status}: {} expected {}, found {actual}",
            check.tool, check.expected
        );
    }
}
