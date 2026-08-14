//! The tq developer harness CLI.
//!
//! Exit-code contract: 0 = clean/passed, 1 = findings or failed checks,
//! 2 = harness error. Automation branches on the exit code and reuses the
//! rendered output directly.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use tq_dev::action::ActionPlan;
use tq_dev::audit::AuditReport;
use tq_dev::change_scope::{ChangeSource, classify_git_changes};
use tq_dev::check::{CheckProfile, CheckTarget};
use tq_dev::error::DevError;
use tq_dev::render::{self, OutputMode};
use tq_dev::runtime_deps::RuntimeDependencyChange;
use tq_dev::{
    artifacts, check, cleanup, dependabot, doctor, external_pins, native_env, pins, policy,
    release, runtime_deps, setup, update, workspace_version,
};

const EXIT_FINDINGS: u8 = 1;
const EXIT_ERROR: u8 = 2;

#[derive(Debug, Parser)]
#[command(name = "tq-dev", about = "The tq developer harness")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Run deterministic local validation gates")]
    Check(CheckArgs),
    #[command(about = "Audit or update repository-owned dependencies")]
    Deps {
        #[command(subcommand)]
        command: DepsCommand,
    },
    #[command(about = "Inspect the local developer environment")]
    Health {
        #[command(subcommand)]
        command: HealthCommand,
    },
    #[command(about = "Verify repository policy invariants")]
    Policy {
        #[command(subcommand)]
        command: PolicyCommand,
    },
    #[command(about = "Build and verify release artifacts")]
    Release {
        #[command(subcommand)]
        command: ReleaseCommand,
    },
    #[command(
        name = "change-scope",
        about = "Classify a CI change range into typed policy scopes"
    )]
    ChangeScope(ChangeScopeArgs),
    #[command(
        name = "runtime-deps",
        about = "Detect shipped runtime dependency changes between git refs"
    )]
    RuntimeDeps(RuntimeDepsArgs),
    #[command(about = "Install repository dependencies and hooks")]
    Setup(MutationArgs),
}

#[derive(Debug, clap::Args)]
struct CheckArgs {
    /// Validation target to run.
    #[arg(default_value = "routine", value_enum)]
    target: CheckTarget,
    /// Check depth: fast for daily work, full for release-sensitive gates.
    #[arg(long, value_enum, default_value_t = CheckProfile::Fast)]
    profile: CheckProfile,
    #[command(flatten)]
    common: CommonArgs,
    /// Suppress successful output.
    #[arg(long)]
    quiet: bool,
    /// Print the resolved check plan without running commands.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Subcommand)]
enum DepsCommand {
    #[command(
        name = "audit-latest",
        about = "Report available dependency and tool updates (read-only)"
    )]
    AuditLatest(ReportArgs),
    #[command(
        name = "audit-security",
        about = "Run dependency security audits (read-only)"
    )]
    AuditSecurity(ReportArgs),
    #[command(
        name = "audit-maintenance-tools",
        about = "Report drift in pinned Rust maintenance tools (read-only)"
    )]
    AuditMaintenanceTools(ReportArgs),
    #[command(
        about = "Apply deterministic dependency and toolchain updates",
        after_help = "Use --dry-run to preview the update plan without applying changes.\n\
                      Audit subcommands are read-only and take no --dry-run."
    )]
    Update(MutationArgs),
}

#[derive(Debug, Subcommand)]
enum HealthCommand {
    #[command(about = "Check pinned tools and native build prerequisites")]
    Doctor(ReportArgs),
    #[command(name = "automation-tools", about = "Check pinned workflow lint tools")]
    AutomationTools(ReportArgs),
    #[command(about = "Remove repository-generated developer artifacts")]
    Cleanup(MutationArgs),
}

#[derive(Debug, Subcommand)]
enum PolicyCommand {
    #[command(name = "verify-pins", about = "Verify repository pinning policy")]
    VerifyPins(RepoRootArgs),
    #[command(
        name = "verify-automation",
        about = "Verify fail-closed workflow and change-scope policy"
    )]
    VerifyAutomation(RepoRootArgs),
    #[command(name = "verify-release", about = "Verify release policy invariants")]
    VerifyRelease(RepoRootArgs),
    #[command(
        name = "verify-dependabot",
        about = "Verify Dependabot policy coverage"
    )]
    VerifyDependabot(RepoRootArgs),
    #[command(
        name = "verify-workspace-version",
        about = "Verify workspace version consistency"
    )]
    VerifyWorkspaceVersion(RepoRootArgs),
    #[command(
        name = "verify-action-pins",
        about = "Verify external GitHub Actions use immutable commit SHAs"
    )]
    VerifyActionPins(RepoRootArgs),
    #[command(
        name = "verify-pre-commit-pins",
        about = "Verify external pre-commit repositories use immutable commit SHAs"
    )]
    VerifyPreCommitPins(RepoRootArgs),
    #[command(
        name = "audit-external-pins",
        about = "Report drift in pinned external repositories"
    )]
    AuditExternalPins(ReportArgs),
}

#[derive(Debug, Subcommand)]
enum ReleaseCommand {
    #[command(about = "Build release artifacts through the harness")]
    Build(MutationArgs),
    #[command(
        name = "build-tool-requirements",
        about = "Print exact release build tool requirements as GitHub output"
    )]
    BuildToolRequirements(RepoRootArgs),
    #[command(name = "verify-artifacts", about = "Verify release artifact contents")]
    VerifyArtifacts(VerifyArtifactsArgs),
}

#[derive(Debug, clap::Args)]
struct CommonArgs {
    /// Repository root used to resolve workspace files.
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputMode::Human)]
    output: OutputMode,
}

#[derive(Debug, clap::Args)]
struct ReportArgs {
    #[command(flatten)]
    common: CommonArgs,
    /// Suppress successful output.
    #[arg(long)]
    quiet: bool,
}

#[derive(Debug, clap::Args)]
struct MutationArgs {
    #[command(flatten)]
    common: CommonArgs,
    /// Print the action plan without applying changes.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, clap::Args)]
struct RepoRootArgs {
    /// Repository root used to resolve workspace files.
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum ChangeEvent {
    #[value(name = "pull_request")]
    PullRequest,
    Push,
    #[value(name = "workflow_dispatch")]
    WorkflowDispatch,
}

#[derive(Debug, clap::Args)]
struct ChangeScopeArgs {
    /// Repository root used to resolve workspace files.
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
    #[arg(long, value_enum)]
    event: ChangeEvent,
    #[arg(long, default_value = "")]
    base_ref: String,
    #[arg(long, default_value = "")]
    head_ref: String,
}

#[derive(Debug, clap::Args)]
struct RuntimeDepsArgs {
    /// Repository root used to resolve workspace files.
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
    #[arg(long)]
    base_ref: String,
    #[arg(long)]
    head_ref: String,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum ArtifactProfile {
    #[value(name = "sdist")]
    Sdist,
    #[value(name = "linux-wheel")]
    LinuxWheel,
    #[value(name = "macos-x86-64-wheel")]
    MacosX86_64Wheel,
    #[value(name = "macos-arm64-wheel")]
    MacosArm64Wheel,
    #[value(name = "windows-x86-64-wheel")]
    WindowsX86_64Wheel,
    #[value(name = "linux-pair")]
    LinuxPair,
    #[value(name = "macos-x86-64-pair")]
    MacosX86_64Pair,
    #[value(name = "macos-arm64-pair")]
    MacosArm64Pair,
    #[value(name = "windows-x86-64-pair")]
    WindowsX86_64Pair,
    #[value(name = "full-release")]
    FullRelease,
}

impl ArtifactProfile {
    const fn expectation(self) -> artifacts::ArtifactExpectation {
        use artifacts::{ArtifactExpectation, WheelPlatform};

        match self {
            Self::Sdist => ArtifactExpectation::SdistOnly,
            Self::LinuxWheel => ArtifactExpectation::WheelOnly(WheelPlatform::PortableLinux),
            Self::MacosX86_64Wheel => ArtifactExpectation::WheelOnly(WheelPlatform::MacosX86_64),
            Self::MacosArm64Wheel => ArtifactExpectation::WheelOnly(WheelPlatform::MacosArm64),
            Self::WindowsX86_64Wheel => {
                ArtifactExpectation::WheelOnly(WheelPlatform::WindowsX86_64)
            }
            Self::LinuxPair => ArtifactExpectation::SdistAndWheel(WheelPlatform::PortableLinux),
            Self::MacosX86_64Pair => ArtifactExpectation::SdistAndWheel(WheelPlatform::MacosX86_64),
            Self::MacosArm64Pair => ArtifactExpectation::SdistAndWheel(WheelPlatform::MacosArm64),
            Self::WindowsX86_64Pair => {
                ArtifactExpectation::SdistAndWheel(WheelPlatform::WindowsX86_64)
            }
            Self::FullRelease => ArtifactExpectation::FullRelease,
        }
    }
}

#[derive(Debug, clap::Args)]
struct VerifyArtifactsArgs {
    #[arg(long, default_value = "dist")]
    dist_dir: PathBuf,
    #[arg(long, value_enum)]
    profile: ArtifactProfile,
    #[arg(long = "forbidden-prefix")]
    forbidden_prefixes: Vec<String>,
}

/// Whether a command found actionable findings.
enum Outcome {
    Clean,
    Findings,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(Outcome::Clean) => ExitCode::SUCCESS,
        Ok(Outcome::Findings) => ExitCode::from(EXIT_FINDINGS),
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::from(EXIT_ERROR)
        }
    }
}

fn run(cli: Cli) -> Result<Outcome, DevError> {
    match cli.command {
        Command::Check(args) => run_check(&args),
        Command::Deps { command } => run_deps(&command),
        Command::Health { command } => run_health(&command),
        Command::Policy { command } => run_policy(&command),
        Command::Release { command } => run_release(&command),
        Command::ChangeScope(args) => run_change_scope(&args),
        Command::RuntimeDeps(args) => run_runtime_deps(&args),
        Command::Setup(args) => {
            if !args.dry_run {
                native_env::verify_build_prerequisites()?;
            }
            run_mutation(&args, &setup::plan())
        }
    }
}

fn run_check(args: &CheckArgs) -> Result<Outcome, DevError> {
    if args.dry_run {
        let plan = check::plan_checks(args.target, args.profile);
        render::check_plan_document(&plan).print(args.common.output, &plan)?;
        return Ok(Outcome::Clean);
    }

    let report = check::run_checks(&args.common.repo_root, args.target, args.profile)?;
    if !args.quiet || !report.passed() {
        render::check_report_document(&report).print(args.common.output, &report)?;
    }
    Ok(if report.passed() {
        Outcome::Clean
    } else {
        Outcome::Findings
    })
}

fn run_deps(command: &DepsCommand) -> Result<Outcome, DevError> {
    match command {
        DepsCommand::AuditLatest(args) => report_audit(
            args,
            "Dependency Freshness Audit",
            &pins::audit_latest(&args.common.repo_root)?,
        ),
        DepsCommand::AuditSecurity(args) => report_audit(
            args,
            "Dependency Security Audit",
            &pins::audit_security(&args.common.repo_root)?,
        ),
        DepsCommand::AuditMaintenanceTools(args) => report_audit(
            args,
            "Rust Maintenance Tool Pin Review",
            &pins::audit_maintenance_tools(&args.common.repo_root)?,
        ),
        DepsCommand::Update(args) => {
            let latest_rust = update::latest_stable_rust(&args.common.repo_root)?;
            run_mutation(args, &update::plan(&args.common.repo_root, &latest_rust)?)
        }
    }
}

fn run_health(command: &HealthCommand) -> Result<Outcome, DevError> {
    match command {
        HealthCommand::Doctor(args) => {
            report_doctor(args, &doctor::diagnose(&args.common.repo_root)?)
        }
        HealthCommand::AutomationTools(args) => {
            report_doctor(args, &doctor::diagnose_automation(&args.common.repo_root)?)
        }
        HealthCommand::Cleanup(args) => run_mutation(args, &cleanup::plan(&args.common.repo_root)),
    }
}

fn report_doctor(args: &ReportArgs, report: &doctor::DoctorReport) -> Result<Outcome, DevError> {
    if !args.quiet || !report.is_healthy() {
        render::doctor_document(report).print(args.common.output, report)?;
    }
    Ok(if report.is_healthy() {
        Outcome::Clean
    } else {
        Outcome::Findings
    })
}

fn run_policy(command: &PolicyCommand) -> Result<Outcome, DevError> {
    match command {
        PolicyCommand::VerifyPins(args) => {
            policy::verify_tool_pins(&args.repo_root).map(|()| Outcome::Clean)
        }
        PolicyCommand::VerifyAutomation(args) => {
            policy::verify_automation_policy(&args.repo_root).map(|()| Outcome::Clean)
        }
        PolicyCommand::VerifyRelease(args) => {
            policy::verify_release_policy(&args.repo_root).map(|()| Outcome::Clean)
        }
        PolicyCommand::VerifyDependabot(args) => {
            dependabot::verify_dependabot(&args.repo_root).map(|()| Outcome::Clean)
        }
        PolicyCommand::VerifyWorkspaceVersion(args) => {
            workspace_version::verify_workspace_version(&args.repo_root).map(|()| Outcome::Clean)
        }
        PolicyCommand::VerifyActionPins(args) => {
            external_pins::verify_action_pins(&args.repo_root).map(|()| Outcome::Clean)
        }
        PolicyCommand::VerifyPreCommitPins(args) => {
            external_pins::verify_pre_commit_pins(&args.repo_root).map(|()| Outcome::Clean)
        }
        PolicyCommand::AuditExternalPins(args) => {
            let report = external_pins::audit_external_pin_drift(&args.common.repo_root)?;
            if !args.quiet || report.drift_detected {
                render::external_pin_document(&report).print(args.common.output, &report)?;
            }
            Ok(if report.drift_detected {
                Outcome::Findings
            } else {
                Outcome::Clean
            })
        }
    }
}

fn run_change_scope(args: &ChangeScopeArgs) -> Result<Outcome, DevError> {
    let source = match args.event {
        ChangeEvent::WorkflowDispatch => ChangeSource::All,
        ChangeEvent::PullRequest => ChangeSource::PullRequest {
            base_ref: &args.base_ref,
            head_ref: &args.head_ref,
        },
        ChangeEvent::Push => ChangeSource::Push {
            base_ref: &args.base_ref,
            head_ref: &args.head_ref,
        },
    };
    let scope = classify_git_changes(&args.repo_root, source)?;
    for path in &scope.unknown_paths {
        eprintln!(
            "Warning: {} is not classified by the change-scope contract; all scoped gates are enabled",
            path.as_path().display()
        );
    }
    print!("{}", scope.github_output());
    Ok(Outcome::Clean)
}

fn run_release(command: &ReleaseCommand) -> Result<Outcome, DevError> {
    match command {
        ReleaseCommand::Build(args) => run_mutation(args, &release::plan(&args.common.repo_root)?),
        ReleaseCommand::BuildToolRequirements(args) => {
            print!(
                "{}",
                release::build_tool_requirements(&args.repo_root)?.github_output()
            );
            Ok(Outcome::Clean)
        }
        ReleaseCommand::VerifyArtifacts(args) => artifacts::verify_artifacts(
            &args.dist_dir,
            args.profile.expectation(),
            if args.forbidden_prefixes.is_empty() {
                None
            } else {
                Some(args.forbidden_prefixes.clone())
            },
        )
        .map(|_| Outcome::Clean),
    }
}

fn run_runtime_deps(args: &RuntimeDepsArgs) -> Result<Outcome, DevError> {
    let change =
        runtime_deps::check_runtime_dep_changes(&args.repo_root, &args.base_ref, &args.head_ref)?;

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

    Ok(Outcome::Clean)
}

fn run_mutation(args: &MutationArgs, plan: &ActionPlan) -> Result<Outcome, DevError> {
    if args.dry_run {
        render::action_plan_document(plan).print(args.common.output, plan)?;
        return Ok(Outcome::Clean);
    }
    plan.apply(&args.common.repo_root).map(|()| Outcome::Clean)
}

fn report_audit(args: &ReportArgs, title: &str, report: &AuditReport) -> Result<Outcome, DevError> {
    if !args.quiet || report.has_findings() {
        render::audit_document(title, report).print(args.common.output, report)?;
    }
    Ok(if report.has_findings() {
        Outcome::Findings
    } else {
        Outcome::Clean
    })
}
