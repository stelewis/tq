use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use dev_output::{
    render_action_plan_agent, render_action_plan_human, render_audit_agent, render_audit_human,
    render_check_agent, render_check_human, render_check_plan_agent, render_check_plan_human,
    render_command_plan_agent, render_command_plan_human, render_doctor_agent, render_doctor_human,
    render_external_pin_agent, render_external_pin_human,
};
use tq_release::{
    DevActionPlan, DevAuditReport, DevCheckPlan, DevCheckProfile, DevCheckReport, DevCheckTarget,
    DevCommandPlan, DevDoctorReport, ExternalPinReport, ReleaseError, RuntimeDependencyChange,
};

mod dev_output;

#[derive(Debug, Parser)]
#[command(
    name = "tq-release",
    about = "Run tq release and developer harness tooling"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(
        name = "check-runtime-deps",
        about = "Detect shipped runtime dependency changes"
    )]
    RuntimeDeps(CheckRuntimeDepsArgs),
    #[command(name = "dev", about = "Run the project developer harness")]
    Dev(DevArgs),
    #[command(
        name = "verify-artifact-contents",
        about = "Verify release artifact contents"
    )]
    ArtifactContents(VerifyArtifactContentsArgs),
    #[command(
        name = "verify-dependabot",
        about = "Verify Dependabot policy coverage"
    )]
    Dependabot(VerifyDependabotArgs),
    #[command(
        name = "verify-release-policy",
        about = "Verify release policy invariants"
    )]
    ReleasePolicy(VerifyReleasePolicyArgs),
    #[command(
        name = "verify-workspace-version",
        about = "Verify workspace version consistency"
    )]
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
    #[command(name = "check", about = "Run deterministic local validation gates")]
    Check(CheckArgs),
    #[command(name = "deps", about = "Audit or update repository-owned dependencies")]
    Deps(DepsArgs),
    #[command(
        name = "health",
        about = "Inspect or clean the local developer environment"
    )]
    Health(HealthArgs),
    #[command(name = "policy", about = "Verify repository policy invariants")]
    Policy(PolicyArgs),
    #[command(name = "release", about = "Build and verify release artifacts")]
    Release(ReleaseArgs),
    #[command(
        name = "setup",
        about = "Install pinned developer toolchain prerequisites"
    )]
    Setup(SetupArgs),
}

#[derive(Debug, clap::Args)]
struct CheckArgs {
    /// Validation target to run.
    #[arg(default_value = "routine", value_enum)]
    target: CheckTargetArg,
    /// Check depth: fast for daily work, full for release-sensitive gates.
    #[arg(long, value_enum, default_value_t = CheckProfileArg::Fast)]
    profile: CheckProfileArg,
    /// Repository root used to resolve workspace files.
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
    /// Output format. Use agent for concise Markdown; use json for automation.
    #[arg(long, value_enum, default_value_t = OutputMode::Human)]
    output: OutputMode,
    /// Suppress successful human output.
    #[arg(long)]
    quiet: bool,
    /// Print the resolved check plan without running commands.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum CheckTargetArg {
    Routine,
    Docs,
    ReleasePolicy,
    Package,
    ReleaseBuild,
    All,
}

impl From<CheckTargetArg> for DevCheckTarget {
    fn from(value: CheckTargetArg) -> Self {
        match value {
            CheckTargetArg::Routine => Self::Routine,
            CheckTargetArg::Docs => Self::Docs,
            CheckTargetArg::ReleasePolicy => Self::ReleasePolicy,
            CheckTargetArg::Package => Self::Package,
            CheckTargetArg::ReleaseBuild => Self::ReleaseBuild,
            CheckTargetArg::All => Self::All,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum CheckProfileArg {
    Fast,
    Full,
}

impl From<CheckProfileArg> for DevCheckProfile {
    fn from(value: CheckProfileArg) -> Self {
        match value {
            CheckProfileArg::Fast => Self::Fast,
            CheckProfileArg::Full => Self::Full,
        }
    }
}

#[derive(Debug, clap::Args)]
struct DepsArgs {
    #[command(subcommand)]
    command: DepsCommand,
}

#[derive(Debug, Subcommand)]
enum DepsCommand {
    #[command(
        name = "audit-latest",
        about = "Report available dependency and tool updates"
    )]
    AuditLatest(ReportArgs),
    #[command(name = "audit-security", about = "Run dependency security audits")]
    AuditSecurity(ReportArgs),
    #[command(
        name = "update",
        about = "Apply deterministic dependency and toolchain updates"
    )]
    Update(MutationArgs),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputMode {
    Human,
    Json,
    Agent,
}

#[derive(Debug, clap::Args)]
struct ReportArgs {
    /// Repository root used to resolve workspace files.
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
    /// Output format. Use agent for concise Markdown; use json for automation.
    #[arg(long, value_enum, default_value_t = OutputMode::Human)]
    output: OutputMode,
    /// Suppress successful human output.
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
    #[command(
        name = "cleanup",
        about = "Remove obsolete local toolchains and harness caches"
    )]
    Cleanup(MutationArgs),
    #[command(
        name = "doctor",
        about = "Check pinned tools and native build prerequisites"
    )]
    Doctor(ReportArgs),
}

#[derive(Debug, clap::Args)]
struct PolicyArgs {
    #[command(subcommand)]
    command: PolicyCommand,
}

#[derive(Debug, Subcommand)]
enum PolicyCommand {
    #[command(
        name = "audit-external-pins",
        about = "Report drift in pinned external repositories"
    )]
    AuditExternalPins(ReportArgs),
    #[command(name = "verify-pins", about = "Verify repository pinning policy")]
    VerifyPins(RepoRootArgs),
}

#[derive(Debug, clap::Args)]
struct ReleaseArgs {
    #[command(subcommand)]
    command: ReleaseCommand,
}

#[derive(Debug, Subcommand)]
enum ReleaseCommand {
    #[command(name = "build", about = "Build release artifacts through the harness")]
    Build(ReleaseBuildArgs),
}

#[derive(Debug, clap::Args)]
struct RepoRootArgs {
    /// Repository root used to resolve workspace files.
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
}

#[derive(Debug, clap::Args)]
struct MutationArgs {
    /// Repository root used to resolve workspace files.
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
    /// Output format. Use agent for concise Markdown; use json for automation.
    #[arg(long, value_enum, default_value_t = OutputMode::Human)]
    output: OutputMode,
    /// Print the planned commands and mutations without applying changes.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, clap::Args)]
struct SetupArgs {
    /// Repository root used to resolve workspace files.
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
    /// Output format. Use agent for concise Markdown; use json for automation.
    #[arg(long, value_enum, default_value_t = OutputMode::Human)]
    output: OutputMode,
    /// Print the setup command plan without running commands.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, clap::Args)]
struct ReleaseBuildArgs {
    /// Repository root used to resolve workspace files.
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,
    /// Output format. Use agent for concise Markdown; use json for automation.
    #[arg(long, value_enum, default_value_t = OutputMode::Human)]
    output: OutputMode,
    /// Print the release build command plan without running commands.
    #[arg(long)]
    dry_run: bool,
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
        DevCommand::Check(args) => run_check_command(args),
        DevCommand::Deps(args) => run_deps_command(args),
        DevCommand::Health(args) => run_health_command(args),
        DevCommand::Policy(args) => run_policy_command(args),
        DevCommand::Release(args) => run_release_command(args),
        DevCommand::Setup(args) => run_setup_command(args),
    }
}

fn run_setup_command(args: &SetupArgs) -> Result<(), ReleaseError> {
    if args.dry_run {
        let plan = tq_release::plan_setup_dev_environment(&args.repo_root)?;
        print_command_plan(&plan, args.output)?;
        return Ok(());
    }
    tq_release::setup_dev_environment(&args.repo_root)
}

fn run_check_command(args: &CheckArgs) -> Result<(), ReleaseError> {
    let target = DevCheckTarget::from(args.target);
    let profile = DevCheckProfile::from(args.profile);
    if args.dry_run {
        let plan = tq_release::plan_dev_checks(target, profile);
        print_check_plan(&plan, args.output)?;
        return Ok(());
    }

    let report = tq_release::run_dev_checks(&args.repo_root, target, profile)?;
    if !args.quiet || !report.passed() {
        print_check_report(&report, args.output)?;
    }
    if report.passed() {
        Ok(())
    } else {
        Err(ReleaseError::RepositoryPolicyViolation {
            details: "developer checks failed".to_owned(),
        })
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
        DepsCommand::Update(args) => run_deps_update_command(args),
    }
}

fn run_deps_update_command(args: &MutationArgs) -> Result<(), ReleaseError> {
    if args.dry_run {
        let plan = tq_release::plan_update_dev_dependencies(&args.repo_root)?;
        print_action_plan(&plan, args.output)?;
        return Ok(());
    }
    tq_release::update_dev_dependencies(&args.repo_root)
}

fn run_health_command(args: &HealthArgs) -> Result<(), ReleaseError> {
    match &args.command {
        HealthCommand::Cleanup(args) => run_health_cleanup_command(args),
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

fn run_health_cleanup_command(args: &MutationArgs) -> Result<(), ReleaseError> {
    if args.dry_run {
        let plan = tq_release::plan_cleanup_dev_environment(&args.repo_root)?;
        print_action_plan(&plan, args.output)?;
        return Ok(());
    }
    tq_release::cleanup_dev_environment(&args.repo_root)
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
        ReleaseCommand::Build(args) => run_release_build_command(args),
    }
}

fn run_release_build_command(args: &ReleaseBuildArgs) -> Result<(), ReleaseError> {
    if args.dry_run {
        let plan = tq_release::plan_release_artifacts();
        print_command_plan(&plan, args.output)?;
        return Ok(());
    }
    tq_release::build_release_artifacts(&args.repo_root)
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

fn print_check_plan(plan: &DevCheckPlan, output: OutputMode) -> Result<(), ReleaseError> {
    match output {
        OutputMode::Human => print!("{}", render_check_plan_human(plan)),
        OutputMode::Json => print_json(plan)?,
        OutputMode::Agent => print!("{}", render_check_plan_agent(plan)),
    }

    Ok(())
}

fn print_check_report(report: &DevCheckReport, output: OutputMode) -> Result<(), ReleaseError> {
    match output {
        OutputMode::Human => print!("{}", render_check_human(report)),
        OutputMode::Json => print_json(report)?,
        OutputMode::Agent => print!("{}", render_check_agent(report)),
    }

    Ok(())
}

fn print_command_plan(plan: &DevCommandPlan, output: OutputMode) -> Result<(), ReleaseError> {
    match output {
        OutputMode::Human => print!("{}", render_command_plan_human(plan)),
        OutputMode::Json => print_json(plan)?,
        OutputMode::Agent => print!("{}", render_command_plan_agent(plan)),
    }

    Ok(())
}

fn print_action_plan(plan: &DevActionPlan, output: OutputMode) -> Result<(), ReleaseError> {
    match output {
        OutputMode::Human => print!("{}", render_action_plan_human(plan)),
        OutputMode::Json => print_json(plan)?,
        OutputMode::Agent => print!("{}", render_action_plan_agent(plan)),
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
