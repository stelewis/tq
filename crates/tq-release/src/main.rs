use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use dev_output::{
    render_audit_agent, render_audit_human, render_doctor_agent, render_doctor_human,
    render_external_pin_agent, render_external_pin_human,
};
use tq_release::{
    DevAuditReport, DevDoctorReport, ExternalPinReport, ReleaseError, RuntimeDependencyChange,
};

mod dev_output;

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
    #[command(name = "deps")]
    Deps(DepsArgs),
    #[command(name = "health")]
    Health(HealthArgs),
    #[command(name = "policy")]
    Policy(PolicyArgs),
    #[command(name = "release")]
    Release(ReleaseArgs),
    #[command(name = "setup")]
    Setup(RepoRootArgs),
}

#[derive(Debug, clap::Args)]
struct DepsArgs {
    #[command(subcommand)]
    command: DepsCommand,
}

#[derive(Debug, Subcommand)]
enum DepsCommand {
    #[command(name = "audit-latest")]
    AuditLatest(ReportArgs),
    #[command(name = "audit-security")]
    AuditSecurity(ReportArgs),
    #[command(name = "update")]
    Update(RepoRootArgs),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputMode {
    Human,
    Json,
    Agent,
}

#[derive(Debug, clap::Args)]
struct ReportArgs {
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
    #[arg(long, value_enum, default_value_t = OutputMode::Human)]
    output: OutputMode,
    #[arg(long)]
    quiet: bool,
}

#[derive(Debug, clap::Args)]
struct HealthArgs {
    #[command(subcommand)]
    command: HealthCommand,
}

#[derive(Debug, Subcommand)]
enum HealthCommand {
    #[command(name = "cleanup")]
    Cleanup(RepoRootArgs),
    #[command(name = "doctor")]
    Doctor(ReportArgs),
}

#[derive(Debug, clap::Args)]
struct PolicyArgs {
    #[command(subcommand)]
    command: PolicyCommand,
}

#[derive(Debug, Subcommand)]
enum PolicyCommand {
    #[command(name = "audit-external-pins")]
    AuditExternalPins(ReportArgs),
    #[command(name = "verify-pins")]
    VerifyPins(RepoRootArgs),
}

#[derive(Debug, clap::Args)]
struct ReleaseArgs {
    #[command(subcommand)]
    command: ReleaseCommand,
}

#[derive(Debug, Subcommand)]
enum ReleaseCommand {
    #[command(name = "build")]
    Build(RepoRootArgs),
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
        DevCommand::Deps(args) => run_deps_command(args),
        DevCommand::Health(args) => run_health_command(args),
        DevCommand::Policy(args) => run_policy_command(args),
        DevCommand::Release(args) => run_release_command(args),
        DevCommand::Setup(args) => tq_release::setup_dev_environment(&args.repo_root),
    }
}

fn run_deps_command(args: &DepsArgs) -> Result<(), ReleaseError> {
    match &args.command {
        DepsCommand::AuditLatest(args) => report_audit(
            &tq_release::audit_latest_dev_dependencies(&args.repo_root)?,
            "dependency freshness audit found updates or failures",
            args.output,
            args.quiet,
        ),
        DepsCommand::AuditSecurity(args) => report_audit(
            &tq_release::audit_security_dev_dependencies(&args.repo_root)?,
            "dependency security audit found issues or failures",
            args.output,
            args.quiet,
        ),
        DepsCommand::Update(args) => tq_release::update_dev_dependencies(&args.repo_root),
    }
}

fn run_health_command(args: &HealthArgs) -> Result<(), ReleaseError> {
    match &args.command {
        HealthCommand::Cleanup(args) => tq_release::cleanup_dev_environment(&args.repo_root),
        HealthCommand::Doctor(args) => {
            let report = tq_release::doctor_dev_environment(&args.repo_root)?;
            if !args.quiet || !report.is_healthy() {
                print_doctor_report(&report, args.output)?;
            }
            if report.is_healthy() {
                Ok(())
            } else {
                Err(ReleaseError::RepositoryPolicyViolation {
                    details: "developer environment does not match .github/dev-tools.toml"
                        .to_owned(),
                })
            }
        }
    }
}

fn run_policy_command(args: &PolicyArgs) -> Result<(), ReleaseError> {
    match &args.command {
        PolicyCommand::AuditExternalPins(args) => report_external_pins(
            &tq_release::audit_external_pin_drift(&args.repo_root)?,
            args.output,
            args.quiet,
        ),
        PolicyCommand::VerifyPins(args) => tq_release::verify_dev_tool_pins(&args.repo_root),
    }
}

fn run_release_command(args: &ReleaseArgs) -> Result<(), ReleaseError> {
    match &args.command {
        ReleaseCommand::Build(args) => tq_release::build_release_artifacts(&args.repo_root),
    }
}

fn print_doctor_report(report: &DevDoctorReport, output: OutputMode) -> Result<(), ReleaseError> {
    match output {
        OutputMode::Human => print!("{}", render_doctor_human(report)),
        OutputMode::Json => print_json(report)?,
        OutputMode::Agent => print!("{}", render_doctor_agent(report)),
    }

    Ok(())
}

fn report_audit(
    report: &DevAuditReport,
    failure_message: &str,
    output: OutputMode,
    quiet: bool,
) -> Result<(), ReleaseError> {
    if !quiet || report.has_findings() {
        print_audit_report(report, output)?;
    }
    if report.has_findings() {
        Err(ReleaseError::RepositoryPolicyViolation {
            details: failure_message.to_owned(),
        })
    } else {
        Ok(())
    }
}

fn print_audit_report(report: &DevAuditReport, output: OutputMode) -> Result<(), ReleaseError> {
    match output {
        OutputMode::Human => print!("{}", render_audit_human(report)),
        OutputMode::Json => print_json(report)?,
        OutputMode::Agent => print!("{}", render_audit_agent(report)),
    }

    Ok(())
}

fn report_external_pins(
    report: &ExternalPinReport,
    output: OutputMode,
    quiet: bool,
) -> Result<(), ReleaseError> {
    if !quiet || report.drift_detected {
        print_external_pin_report(report, output)?;
    }
    if report.drift_detected {
        Err(ReleaseError::RepositoryPolicyViolation {
            details: "frozen external pins need review".to_owned(),
        })
    } else {
        Ok(())
    }
}

fn print_external_pin_report(
    report: &ExternalPinReport,
    output: OutputMode,
) -> Result<(), ReleaseError> {
    match output {
        OutputMode::Human => print!("{}", render_external_pin_human(report)),
        OutputMode::Json => print_json(report)?,
        OutputMode::Agent => print!("{}", render_external_pin_agent(report)),
    }

    Ok(())
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<(), ReleaseError> {
    let output =
        serde_json::to_string_pretty(value).map_err(|source| ReleaseError::InvalidInput {
            path: PathBuf::from("<json-output>"),
            message: source.to_string(),
        })?;
    println!("{output}");
    Ok(())
}
