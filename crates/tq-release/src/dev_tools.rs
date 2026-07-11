use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use serde::Serialize;
use toml::Value;

use crate::ReleaseError;

const DEV_TOOLS_PATH: &str = ".github/dev-tools.toml";

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevDoctorReport {
    pub summary: DevDoctorSummary,
    pub checks: Vec<DevDoctorCheck>,
}

impl DevDoctorReport {
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.summary.status == DevDoctorStatus::Healthy
    }

    fn new(checks: Vec<DevDoctorCheck>) -> Self {
        Self {
            summary: DevDoctorSummary::from_checks(&checks),
            checks,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevDoctorSummary {
    pub status: DevDoctorStatus,
    pub total: usize,
    pub ok: usize,
    pub missing: usize,
    pub mismatched: usize,
}

impl DevDoctorSummary {
    fn from_checks(checks: &[DevDoctorCheck]) -> Self {
        let ok = checks
            .iter()
            .filter(|check| check.status == DevToolStatus::Ok)
            .count();
        let missing = checks
            .iter()
            .filter(|check| check.status == DevToolStatus::Missing)
            .count();
        let mismatched = checks
            .iter()
            .filter(|check| check.status == DevToolStatus::Mismatched)
            .count();

        Self {
            status: if missing == 0 && mismatched == 0 {
                DevDoctorStatus::Healthy
            } else {
                DevDoctorStatus::Unhealthy
            },
            total: checks.len(),
            ok,
            missing,
            mismatched,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DevDoctorStatus {
    Healthy,
    Unhealthy,
}

impl DevDoctorStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Unhealthy => "unhealthy",
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevDoctorCheck {
    pub tool: String,
    pub expected: String,
    pub actual: Option<String>,
    pub status: DevToolStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DevToolStatus {
    Ok,
    Missing,
    Mismatched,
}

impl DevToolStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Missing => "missing",
            Self::Mismatched => "mismatched",
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevAuditReport {
    pub summary: DevAuditSummary,
    pub checks: Vec<DevAuditCheck>,
}

impl DevAuditReport {
    #[must_use]
    pub fn has_findings(&self) -> bool {
        self.summary.status != DevAuditReportStatus::Clean
    }

    fn new(checks: Vec<DevAuditCheck>) -> Self {
        Self {
            summary: DevAuditSummary::from_checks(&checks),
            checks,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevAuditSummary {
    pub status: DevAuditReportStatus,
    pub total: usize,
    pub clean: usize,
    pub findings: usize,
    pub failed: usize,
}

impl DevAuditSummary {
    fn from_checks(checks: &[DevAuditCheck]) -> Self {
        let clean = checks
            .iter()
            .filter(|check| check.status == DevAuditStatus::Clean)
            .count();
        let findings = checks
            .iter()
            .filter(|check| check.status == DevAuditStatus::Findings)
            .count();
        let failed = checks
            .iter()
            .filter(|check| check.status == DevAuditStatus::Failed)
            .count();

        let status = if failed > 0 {
            DevAuditReportStatus::Failed
        } else if findings > 0 {
            DevAuditReportStatus::Findings
        } else {
            DevAuditReportStatus::Clean
        };

        Self {
            status,
            total: checks.len(),
            clean,
            findings,
            failed,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DevAuditReportStatus {
    Clean,
    Findings,
    Failed,
}

impl DevAuditReportStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Findings => "findings",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevAuditCheck {
    pub name: String,
    pub command: String,
    pub status: DevAuditStatus,
    pub output: String,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevCheckPlan {
    pub target: DevCheckTarget,
    pub profile: DevCheckProfile,
    pub tasks: Vec<DevCheckTask>,
}

impl DevCheckPlan {
    const fn new(
        target: DevCheckTarget,
        profile: DevCheckProfile,
        tasks: Vec<DevCheckTask>,
    ) -> Self {
        Self {
            target,
            profile,
            tasks,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevCheckTask {
    pub id: String,
    pub label: String,
    pub command: DevCheckCommand,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevCheckCommand {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevCommandPlan {
    pub title: String,
    pub commands: Vec<DevPlannedCommand>,
}

impl DevCommandPlan {
    fn new(title: &str, commands: Vec<HarnessCommand>) -> Self {
        Self {
            title: title.to_owned(),
            commands: commands.into_iter().map(DevPlannedCommand::from).collect(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevPlannedCommand {
    pub command: DevCheckCommand,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevActionPlan {
    pub title: String,
    pub actions: Vec<DevPlannedAction>,
}

impl DevActionPlan {
    fn new(title: &str, actions: Vec<DevPlannedAction>) -> Self {
        Self {
            title: title.to_owned(),
            actions,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DevPlannedAction {
    pub label: String,
    pub action: DevAction,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum DevAction {
    Command {
        command: DevCheckCommand,
    },
    ReplaceText {
        path: PathBuf,
        from: String,
        to: String,
    },
    RemovePath {
        path: PathBuf,
    },
}

#[derive(Debug, PartialEq, Serialize)]
pub struct DevCheckReport {
    pub summary: DevCheckSummary,
    pub checks: Vec<DevCheckResult>,
}

impl DevCheckReport {
    #[must_use]
    pub fn passed(&self) -> bool {
        self.summary.status == DevCheckReportStatus::Passed
    }

    fn new(checks: Vec<DevCheckResult>, elapsed_seconds: f64) -> Self {
        Self {
            summary: DevCheckSummary::from_results(&checks, elapsed_seconds),
            checks,
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct DevCheckSummary {
    pub status: DevCheckReportStatus,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub elapsed_seconds: f64,
}

impl DevCheckSummary {
    fn from_results(results: &[DevCheckResult], elapsed_seconds: f64) -> Self {
        let passed = results
            .iter()
            .filter(|result| result.status == DevCheckStatus::Passed)
            .count();
        let failed = results.len() - passed;
        Self {
            status: if failed == 0 {
                DevCheckReportStatus::Passed
            } else {
                DevCheckReportStatus::Failed
            },
            total: results.len(),
            passed,
            failed,
            elapsed_seconds,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DevCheckReportStatus {
    Passed,
    Failed,
}

impl DevCheckReportStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct DevCheckResult {
    pub task: DevCheckTask,
    pub status: DevCheckStatus,
    pub output: String,
    pub elapsed_seconds: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DevCheckStatus {
    Passed,
    Failed,
}

impl DevCheckStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DevCheckTarget {
    Routine,
    Docs,
    ReleasePolicy,
    Package,
    ReleaseBuild,
    All,
}

impl DevCheckTarget {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Routine => "routine",
            Self::Docs => "docs",
            Self::ReleasePolicy => "release-policy",
            Self::Package => "package",
            Self::ReleaseBuild => "release-build",
            Self::All => "all",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DevCheckProfile {
    Fast,
    Full,
}

impl DevCheckProfile {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Full => "full",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DevAuditStatus {
    Clean,
    Findings,
    Failed,
}

impl DevAuditStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Findings => "findings",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug)]
struct DevToolsManifest {
    rust: String,
    python: String,
    uv: String,
    node: String,
    npm: String,
    mise: String,
    cargo_outdated: String,
    cargo_audit: String,
    cargo_deny: String,
}

#[derive(Debug)]
struct HarnessCommand {
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
}

pub fn verify_dev_tool_pins(repo_root: &Path) -> Result<(), ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    let mut violations = Vec::new();

    verify_rust_toolchain(repo_root, &manifest, &mut violations)?;
    verify_cargo_policy(repo_root, &manifest, &mut violations)?;
    verify_mise_tools(repo_root, &manifest, &mut violations)?;
    verify_node_package(repo_root, &manifest, &mut violations)?;
    verify_python_uv_action(repo_root, &manifest, &mut violations)?;
    verify_rust_maintenance_action(repo_root, &manifest, &mut violations)?;
    verify_dependabot_ecosystems(repo_root, &mut violations)?;

    if violations.is_empty() {
        return Ok(());
    }

    Err(ReleaseError::RepositoryPolicyViolation {
        details: violations.join("\n"),
    })
}

pub fn doctor_dev_environment(repo_root: &Path) -> Result<DevDoctorReport, ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    let mut checks = vec![
        check_version("rustc", &manifest.rust, "rustc", &["--version"]),
        check_version("cargo", &manifest.rust, "cargo", &["--version"]),
        check_version("uv", &manifest.uv, "uv", &["--version"]),
        check_version("node", &manifest.node, "node", &["--version"]),
        check_version("npm", &manifest.npm, "npm", &["--version"]),
        check_version("mise", &manifest.mise, "mise", &["--version"]),
        check_python(&manifest.python),
        check_version(
            "cargo-outdated",
            &manifest.cargo_outdated,
            "cargo",
            &["outdated", "--version"],
        ),
        check_version(
            "cargo-deny",
            &manifest.cargo_deny,
            "cargo",
            &["deny", "--version"],
        ),
        check_version(
            "cargo-audit",
            &manifest.cargo_audit,
            "cargo",
            &["audit", "--version"],
        ),
        check_pkg_config(),
        check_homebrew_openssl(),
    ];

    let cleanup = obsolete_rust_toolchains(repo_root, &manifest.rust);
    if cleanup.is_empty() {
        return Ok(DevDoctorReport::new(checks));
    }

    checks.push(DevDoctorCheck {
        tool: "rustup cleanup".to_owned(),
        expected: format!(
            "only {} and explicitly installed non-project toolchains",
            manifest.rust
        ),
        actual: Some(cleanup.join(", ")),
        status: DevToolStatus::Mismatched,
    });
    Ok(DevDoctorReport::new(checks))
}

pub fn setup_dev_environment(repo_root: &Path) -> Result<(), ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    verify_native_build_prerequisites()?;
    run_commands(repo_root, setup_commands(&manifest))?;
    install_missing_cargo_tools(repo_root, &manifest)
}

pub fn plan_setup_dev_environment(repo_root: &Path) -> Result<DevCommandPlan, ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    let mut commands = setup_commands(&manifest);
    commands.extend(
        cargo_tool_installs_needed(repo_root, &manifest)
            .into_iter()
            .map(|install| install.command),
    );
    Ok(DevCommandPlan::new("Developer setup", commands))
}

#[must_use]
pub fn plan_dev_checks(target: DevCheckTarget, profile: DevCheckProfile) -> DevCheckPlan {
    DevCheckPlan::new(target, profile, check_tasks(target, profile))
}

pub fn run_dev_checks(
    repo_root: &Path,
    target: DevCheckTarget,
    profile: DevCheckProfile,
) -> Result<DevCheckReport, ReleaseError> {
    let plan = plan_dev_checks(target, profile);
    run_check_tasks(repo_root, plan.tasks)
}

pub fn update_dev_dependencies(repo_root: &Path) -> Result<(), ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    let rust = rust_update_version(&manifest);
    update_rust_toolchain_pin(repo_root, &manifest, &rust)?;
    run_commands(repo_root, update_commands(&rust))
}

pub fn plan_update_dev_dependencies(repo_root: &Path) -> Result<DevActionPlan, ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    let rust = rust_update_version(&manifest);
    let mut actions = rust_toolchain_update_actions(repo_root, &manifest, &rust);
    actions.extend(
        update_commands(&rust)
            .into_iter()
            .map(update_command_action),
    );
    Ok(DevActionPlan::new("Dependency update", actions))
}

pub fn audit_latest_dev_dependencies(repo_root: &Path) -> Result<DevAuditReport, ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    let mut report = audit_commands(repo_root, latest_audit_commands())?;
    report
        .checks
        .insert(0, rust_toolchain_latest_check(&manifest));
    Ok(report)
}

pub fn audit_security_dev_dependencies(repo_root: &Path) -> Result<DevAuditReport, ReleaseError> {
    audit_commands(repo_root, security_audit_commands())
}

pub fn cleanup_dev_environment(repo_root: &Path) -> Result<(), ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    remove_obsolete_rust_toolchains(repo_root, &manifest.rust)?;
    remove_cargo_tools_target_dir(repo_root)
}

pub fn plan_cleanup_dev_environment(repo_root: &Path) -> Result<DevActionPlan, ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    let mut actions = obsolete_rust_toolchains(repo_root, &manifest.rust)
        .into_iter()
        .map(|toolchain| {
            command_action(
                &format!("Remove obsolete Rust toolchain {toolchain}"),
                command("rustup", ["toolchain", "uninstall", &toolchain]),
            )
        })
        .collect::<Vec<_>>();
    actions.push(remove_path_action(
        "Remove Cargo maintenance-tool build cache",
        repo_root.join("target/cargo-tools"),
    ));
    Ok(DevActionPlan::new("Developer environment cleanup", actions))
}

pub fn build_release_artifacts(repo_root: &Path) -> Result<(), ReleaseError> {
    let dist_dir = repo_root.join("dist");
    match std::fs::remove_dir_all(&dist_dir) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(ReleaseError::Io {
                path: dist_dir,
                source,
            });
        }
    }

    run_commands(repo_root, release_build_commands())
}

#[must_use]
pub fn plan_release_artifacts() -> DevCommandPlan {
    DevCommandPlan::new("Release artifact build", release_build_commands())
}

fn read_manifest(repo_root: &Path) -> Result<DevToolsManifest, ReleaseError> {
    let path = repo_root.join(DEV_TOOLS_PATH);
    let document = read_toml(&path)?;

    let schema_version = required_integer(&document, &["schema", "version"], &path)?;
    if schema_version != 1 {
        return Err(ReleaseError::InvalidInput {
            path,
            message: format!("unsupported dev-tools schema version {schema_version}"),
        });
    }

    Ok(DevToolsManifest {
        rust: required_string(&document, &["tools", "rust"], &path)?,
        python: required_string(&document, &["tools", "python"], &path)?,
        uv: required_string(&document, &["tools", "uv"], &path)?,
        node: required_string(&document, &["tools", "node"], &path)?,
        npm: required_string(&document, &["tools", "npm"], &path)?,
        mise: required_string(&document, &["tools", "mise"], &path)?,
        cargo_outdated: required_string(&document, &["rust-maintenance", "cargo-outdated"], &path)?,
        cargo_audit: required_string(&document, &["rust-maintenance", "cargo-audit"], &path)?,
        cargo_deny: required_string(&document, &["rust-maintenance", "cargo-deny"], &path)?,
    })
}

fn verify_rust_toolchain(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), ReleaseError> {
    let path = repo_root.join("rust-toolchain.toml");
    let document = read_toml(&path)?;
    require_equal(
        violations,
        "rust-toolchain.toml toolchain.channel",
        &manifest.rust,
        &required_string(&document, &["toolchain", "channel"], &path)?,
    );
    require_array_contains(
        violations,
        "rust-toolchain.toml toolchain.components",
        &required_string_array(&document, &["toolchain", "components"], &path)?,
        "rustfmt",
    );
    require_array_contains(
        violations,
        "rust-toolchain.toml toolchain.components",
        &required_string_array(&document, &["toolchain", "components"], &path)?,
        "clippy",
    );
    Ok(())
}

fn verify_cargo_policy(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), ReleaseError> {
    let path = repo_root.join("Cargo.toml");
    let document = read_toml(&path)?;
    require_equal(
        violations,
        "Cargo.toml workspace.package.rust-version",
        &minor_version(&manifest.rust),
        &required_string(&document, &["workspace", "package", "rust-version"], &path)?,
    );
    Ok(())
}

fn verify_mise_tools(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), ReleaseError> {
    let path = repo_root.join("mise.toml");
    let document = read_toml(&path)?;
    require_equal(
        violations,
        "mise.toml tools.node",
        &manifest.node,
        &required_string(&document, &["tools", "node"], &path)?,
    );
    Ok(())
}

fn verify_node_package(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), ReleaseError> {
    let path = repo_root.join("package.json");
    let contents = read_to_string(&path)?;
    require_contains(
        violations,
        "package.json packageManager",
        &contents,
        &format!("\"packageManager\": \"npm@{}\"", manifest.npm),
    );
    require_contains(
        violations,
        "package.json engines.node",
        &contents,
        &format!("\"node\": \"{}\"", manifest.node),
    );
    require_contains(
        violations,
        "package.json engines.npm",
        &contents,
        &format!("\"npm\": \"{}\"", manifest.npm),
    );
    Ok(())
}

fn verify_python_uv_action(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), ReleaseError> {
    let path = repo_root.join(".github/actions/setup-python-uv/action.yml");
    let contents = read_to_string(&path)?;
    require_contains(
        violations,
        ".github/actions/setup-python-uv/action.yml python-version default",
        &contents,
        &format!("default: \"{}\"", manifest.python),
    );
    require_contains(
        violations,
        ".github/actions/setup-python-uv/action.yml uv-version default",
        &contents,
        &format!("default: \"{}\"", manifest.uv),
    );
    require_contains(
        violations,
        ".github/actions/setup-python-uv/action.yml setup-uv version input",
        &contents,
        "version: ${{ inputs.uv-version }}",
    );
    Ok(())
}

fn verify_rust_maintenance_action(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), ReleaseError> {
    let path = repo_root.join(".github/actions/setup-rust-maintenance-tools/action.yml");
    let contents = read_to_string(&path)?;
    require_contains(
        violations,
        ".github/actions/setup-rust-maintenance-tools/action.yml cargo-outdated-version",
        &contents,
        &format!("default: \"{}\"", manifest.cargo_outdated),
    );
    require_contains(
        violations,
        ".github/actions/setup-rust-maintenance-tools/action.yml cargo-audit-version",
        &contents,
        &format!("default: \"{}\"", manifest.cargo_audit),
    );
    require_contains(
        violations,
        ".github/actions/setup-rust-maintenance-tools/action.yml cargo-deny-version",
        &contents,
        &format!("default: \"{}\"", manifest.cargo_deny),
    );
    Ok(())
}

fn verify_dependabot_ecosystems(
    repo_root: &Path,
    violations: &mut Vec<String>,
) -> Result<(), ReleaseError> {
    let path = repo_root.join(".github/dependabot.yml");
    let contents = read_to_string(&path)?;
    for ecosystem in [
        "github-actions",
        "pre-commit",
        "uv",
        "cargo",
        "rust-toolchain",
        "npm",
    ] {
        require_contains(
            violations,
            ".github/dependabot.yml update ecosystems",
            &contents,
            &format!("package-ecosystem: \"{ecosystem}\""),
        );
    }
    Ok(())
}

fn check_version(tool: &str, expected: &str, program: &str, args: &[&str]) -> DevDoctorCheck {
    match Command::new(program).args(args).output() {
        Ok(output) if output.status.success() => {
            let actual = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            let status = if actual.contains(expected) {
                DevToolStatus::Ok
            } else {
                DevToolStatus::Mismatched
            };
            DevDoctorCheck {
                tool: tool.to_owned(),
                expected: expected.to_owned(),
                actual: Some(actual),
                status,
            }
        }
        Ok(output) => DevDoctorCheck {
            tool: tool.to_owned(),
            expected: expected.to_owned(),
            actual: Some(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
            status: DevToolStatus::Missing,
        },
        Err(_) => DevDoctorCheck {
            tool: tool.to_owned(),
            expected: expected.to_owned(),
            actual: None,
            status: DevToolStatus::Missing,
        },
    }
}

fn check_python(expected: &str) -> DevDoctorCheck {
    match Command::new("uv")
        .args(["python", "find", expected])
        .output()
    {
        Ok(output) if output.status.success() => DevDoctorCheck {
            tool: "python".to_owned(),
            expected: expected.to_owned(),
            actual: Some(String::from_utf8_lossy(&output.stdout).trim().to_owned()),
            status: DevToolStatus::Ok,
        },
        Ok(output) => DevDoctorCheck {
            tool: "python".to_owned(),
            expected: expected.to_owned(),
            actual: Some(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
            status: DevToolStatus::Missing,
        },
        Err(_) => DevDoctorCheck {
            tool: "python".to_owned(),
            expected: expected.to_owned(),
            actual: None,
            status: DevToolStatus::Missing,
        },
    }
}

fn obsolete_rust_toolchains(repo_root: &Path, pinned: &str) -> Vec<String> {
    let output = Command::new("rustup")
        .args(["toolchain", "list"])
        .current_dir(repo_root)
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|toolchain| toolchain.starts_with("1."))
        .filter(|toolchain| !toolchain.starts_with(pinned))
        .map(ToOwned::to_owned)
        .collect()
}

fn setup_commands(manifest: &DevToolsManifest) -> Vec<HarnessCommand> {
    vec![
        command(
            "rustup",
            [
                "toolchain",
                "install",
                &manifest.rust,
                "--profile",
                "minimal",
                "--component",
                "rustfmt",
                "--component",
                "clippy",
            ],
        ),
        command("mise", ["install"]),
        command("uv", ["python", "install", &manifest.python]),
        command("uv", ["sync", "--locked"]),
        command("npm", ["ci"]),
        command("uv", ["run", "prek", "install", "--install-hooks"]),
    ]
}

fn install_missing_cargo_tools(
    repo_root: &Path,
    manifest: &DevToolsManifest,
) -> Result<(), ReleaseError> {
    for install in cargo_tool_installs_needed(repo_root, manifest) {
        remove_stale_cargo_binaries(install.stale_binaries)?;
        run_commands(repo_root, vec![install.command])?;
    }

    Ok(())
}

struct CargoToolInstall {
    stale_binaries: &'static [&'static str],
    command: HarnessCommand,
}

fn cargo_tool_installs_needed(
    repo_root: &Path,
    manifest: &DevToolsManifest,
) -> Vec<CargoToolInstall> {
    let mut installs = Vec::new();

    if !cargo_subcommand_matches("outdated", &manifest.cargo_outdated) {
        installs.push(CargoToolInstall {
            stale_binaries: &["cargo-outdated"],
            command: cargo_install_command(
                repo_root,
                [
                    "install",
                    "--force",
                    "--locked",
                    &format!("cargo-outdated@{}", manifest.cargo_outdated),
                ],
            ),
        });
    }

    if !cargo_subcommand_matches("deny", &manifest.cargo_deny) {
        installs.push(CargoToolInstall {
            stale_binaries: &["cargo-deny"],
            command: cargo_install_command(
                repo_root,
                [
                    "install",
                    "--force",
                    "--locked",
                    &format!("cargo-deny@{}", manifest.cargo_deny),
                ],
            ),
        });
    }

    if !cargo_subcommand_matches("audit", &manifest.cargo_audit) {
        installs.push(CargoToolInstall {
            stale_binaries: &["cargo-audit", "cargo-audit-audit"],
            command: cargo_install_command(
                repo_root,
                [
                    "install",
                    "--force",
                    "--locked",
                    &format!("cargo-audit@{}", manifest.cargo_audit),
                ],
            ),
        });
    }

    installs
}

fn cargo_subcommand_matches(subcommand: &str, expected: &str) -> bool {
    Command::new("cargo")
        .args([subcommand, "--version"])
        .envs(native_build_env().iter().map(|(key, value)| (key, value)))
        .output()
        .is_ok_and(|output| {
            output.status.success() && String::from_utf8_lossy(&output.stdout).contains(expected)
        })
}

fn remove_stale_cargo_binaries(names: &[&str]) -> Result<(), ReleaseError> {
    let Some(cargo_bin_dir) = cargo_bin_dir() else {
        return Ok(());
    };

    for name in names {
        let path = cargo_bin_dir.join(name);
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => return Err(ReleaseError::Io { path, source }),
        }
    }

    Ok(())
}

fn cargo_bin_dir() -> Option<std::path::PathBuf> {
    if let Some(cargo_home) = std::env::var_os("CARGO_HOME") {
        return Some(Path::new(&cargo_home).join("bin"));
    }

    std::env::var_os("HOME").map(|home| Path::new(&home).join(".cargo/bin"))
}

fn update_commands(rust: &str) -> Vec<HarnessCommand> {
    vec![
        command("rustup", ["update", rust]),
        command("mise", ["install"]),
        command("uv", ["lock", "--upgrade"]),
        command("npm", ["update"]),
        command("uv", ["run", "prek", "autoupdate", "--freeze"]),
    ]
}

fn rust_toolchain_update_actions(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    rust: &str,
) -> Vec<DevPlannedAction> {
    if rust == manifest.rust {
        return Vec::new();
    }

    vec![
        replace_text_action(
            "Update Rust pin in dev tools manifest",
            repo_root.join(DEV_TOOLS_PATH),
            format!("rust = \"{}\"", manifest.rust),
            format!("rust = \"{rust}\""),
        ),
        replace_text_action(
            "Update Rust toolchain file",
            repo_root.join("rust-toolchain.toml"),
            format!("channel = \"{}\"", manifest.rust),
            format!("channel = \"{rust}\""),
        ),
        replace_text_action(
            "Update Cargo MSRV metadata",
            repo_root.join("Cargo.toml"),
            format!("rust-version = \"{}\"", minor_version(&manifest.rust)),
            format!("rust-version = \"{}\"", minor_version(rust)),
        ),
    ]
}

fn command_action(label: &str, command: HarnessCommand) -> DevPlannedAction {
    DevPlannedAction {
        label: label.to_owned(),
        action: DevAction::Command {
            command: DevCheckCommand::from(command),
        },
    }
}

fn update_command_action(command: HarnessCommand) -> DevPlannedAction {
    let label = match command.display().as_str() {
        command if command.starts_with("rustup update ") => "Update selected Rust toolchain",
        "mise install" => "Install pinned mise tools",
        "uv lock --upgrade" => "Upgrade uv lockfile",
        "npm update" => "Update npm dependencies",
        "uv run prek autoupdate --freeze" => "Freeze-update pre-commit hooks",
        _ => "Run dependency update command",
    };
    command_action(label, command)
}

fn replace_text_action(label: &str, path: PathBuf, from: String, to: String) -> DevPlannedAction {
    DevPlannedAction {
        label: label.to_owned(),
        action: DevAction::ReplaceText { path, from, to },
    }
}

fn remove_path_action(label: &str, path: PathBuf) -> DevPlannedAction {
    DevPlannedAction {
        label: label.to_owned(),
        action: DevAction::RemovePath { path },
    }
}

fn check_tasks(target: DevCheckTarget, profile: DevCheckProfile) -> Vec<DevCheckTask> {
    let mut tasks = Vec::new();
    match target {
        DevCheckTarget::Routine => push_routine_check_tasks(&mut tasks),
        DevCheckTarget::Docs => push_docs_check_tasks(&mut tasks),
        DevCheckTarget::ReleasePolicy => push_release_policy_check_tasks(&mut tasks),
        DevCheckTarget::Package => push_package_check_tasks(&mut tasks),
        DevCheckTarget::ReleaseBuild => push_release_build_check_tasks(&mut tasks),
        DevCheckTarget::All => {
            push_routine_check_tasks(&mut tasks);
            push_docs_check_tasks(&mut tasks);
            push_release_policy_check_tasks(&mut tasks);
            if profile == DevCheckProfile::Full {
                push_package_check_tasks(&mut tasks);
                push_release_build_check_tasks(&mut tasks);
            }
        }
    }
    tasks
}

fn push_routine_check_tasks(tasks: &mut Vec<DevCheckTask>) {
    tasks.extend([
        check_task(
            "rust-format",
            "Rust format",
            command("cargo", ["fmt", "--all", "--check"]),
        ),
        check_task(
            "rust-lint",
            "Rust lint",
            command(
                "cargo",
                [
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--locked",
                    "--",
                    "-D",
                    "warnings",
                ],
            ),
        ),
        check_task(
            "rust-tests",
            "Rust tests",
            command("cargo", ["test", "--workspace", "--locked"]),
        ),
    ]);
}

fn push_docs_check_tasks(tasks: &mut Vec<DevCheckTask>) {
    tasks.push(check_task(
        "docs-sync",
        "Generated docs",
        command(
            "cargo",
            [
                "run",
                "-p",
                "tq-docsgen",
                "--locked",
                "--",
                "generate",
                "all",
            ],
        ),
    ));
}

fn push_release_policy_check_tasks(tasks: &mut Vec<DevCheckTask>) {
    tasks.push(check_task(
        "release-policy",
        "Release policy",
        command(
            "cargo",
            [
                "run",
                "-p",
                "tq-release",
                "--locked",
                "--",
                "verify-release-policy",
                "--repo-root",
                ".",
            ],
        ),
    ));
}

fn push_package_check_tasks(tasks: &mut Vec<DevCheckTask>) {
    tasks.push(check_task(
        "cargo-package",
        "Cargo package",
        command("cargo", ["package", "--workspace", "--locked"]),
    ));
}

fn push_release_build_check_tasks(tasks: &mut Vec<DevCheckTask>) {
    tasks.push(check_task(
        "release-build",
        "Release build",
        command("cargo", ["dev", "release", "build", "--repo-root", "."]),
    ));
}

fn check_task(id: &str, label: &str, command: HarnessCommand) -> DevCheckTask {
    DevCheckTask {
        id: id.to_owned(),
        label: label.to_owned(),
        command: DevCheckCommand::from(command),
    }
}

fn update_rust_toolchain_pin(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    rust: &str,
) -> Result<(), ReleaseError> {
    if rust == manifest.rust {
        return Ok(());
    }

    replace_once(
        &repo_root.join(DEV_TOOLS_PATH),
        &format!("rust = \"{}\"", manifest.rust),
        &format!("rust = \"{rust}\""),
    )?;
    replace_once(
        &repo_root.join("rust-toolchain.toml"),
        &format!("channel = \"{}\"", manifest.rust),
        &format!("channel = \"{rust}\""),
    )?;
    replace_once(
        &repo_root.join("Cargo.toml"),
        &format!("rust-version = \"{}\"", minor_version(&manifest.rust)),
        &format!("rust-version = \"{}\"", minor_version(rust)),
    )
}

fn rust_update_version(manifest: &DevToolsManifest) -> String {
    latest_stable_rust()
        .filter(|latest| latest != &manifest.rust)
        .unwrap_or_else(|| manifest.rust.clone())
}

fn rust_toolchain_latest_check(manifest: &DevToolsManifest) -> DevAuditCheck {
    let Some(latest) = latest_stable_rust() else {
        return DevAuditCheck {
            name: "rust".to_owned(),
            command: "rustup check".to_owned(),
            status: DevAuditStatus::Failed,
            output: "unable to determine latest stable Rust toolchain".to_owned(),
        };
    };

    let status = if latest == manifest.rust {
        DevAuditStatus::Clean
    } else {
        DevAuditStatus::Findings
    };
    DevAuditCheck {
        name: "rust".to_owned(),
        command: "rustup check".to_owned(),
        status,
        output: format!("pinned: {}, latest stable: {latest}", manifest.rust),
    }
}

fn latest_stable_rust() -> Option<String> {
    let output = Command::new("rustup").arg("check").output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(parse_stable_rustup_check_line)
}

fn parse_stable_rustup_check_line(line: &str) -> Option<String> {
    let line = line.strip_prefix("stable-")?;
    let (_, version) = line.split_once("up to date:")?;
    version.split_whitespace().next().map(ToOwned::to_owned)
}

fn replace_once(path: &Path, old: &str, new: &str) -> Result<(), ReleaseError> {
    let contents = read_to_string(path)?;
    let count = contents.matches(old).count();
    if count != 1 {
        return Err(ReleaseError::InvalidInput {
            path: path.to_path_buf(),
            message: format!("expected exactly one occurrence of {old:?}, found {count}"),
        });
    }

    std::fs::write(path, contents.replace(old, new)).map_err(|source| ReleaseError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn latest_audit_commands() -> Vec<HarnessCommand> {
    vec![
        command(
            "cargo",
            [
                "+stable",
                "outdated",
                "--workspace",
                "--root-deps-only",
                "--exit-code",
                "1",
            ],
        ),
        command("npm", ["outdated"]),
        command("uv", ["tree", "--outdated"]),
    ]
}

fn security_audit_commands() -> Vec<HarnessCommand> {
    vec![
        command("cargo", ["audit"]),
        command("cargo", ["deny", "check"]),
        command("npm", ["audit", "--audit-level", "moderate"]),
    ]
}

fn release_build_commands() -> Vec<HarnessCommand> {
    vec![
        command("uv", ["build", "--sdist"]),
        command(
            "uv",
            [
                "run",
                "--isolated",
                "--with",
                "maturin>=1.11,<2.0",
                "--",
                "maturin",
                "build",
                "--release",
                "--locked",
                "--manifest-path",
                "crates/tq-cli/Cargo.toml",
                "--bindings",
                "bin",
                "--out",
                "dist",
                "-i",
                "python",
            ],
        ),
    ]
}

fn command<const N: usize>(program: &str, args: [&str; N]) -> HarnessCommand {
    HarnessCommand {
        program: program.to_owned(),
        args: args.into_iter().map(ToOwned::to_owned).collect(),
        env: Vec::new(),
    }
}

fn cargo_install_command<const N: usize>(repo_root: &Path, args: [&str; N]) -> HarnessCommand {
    HarnessCommand {
        program: "cargo".to_owned(),
        args: args.into_iter().map(ToOwned::to_owned).collect(),
        env: cargo_tool_build_env(repo_root),
    }
}

fn cargo_tool_build_env(repo_root: &Path) -> Vec<(String, String)> {
    vec![(
        "CARGO_TARGET_DIR".to_owned(),
        repo_root.join("target/cargo-tools").display().to_string(),
    )]
}

fn audit_commands(
    repo_root: &Path,
    commands: Vec<HarnessCommand>,
) -> Result<DevAuditReport, ReleaseError> {
    let native_env = native_build_env();
    let mut checks = Vec::new();

    for command in commands {
        let output = Command::new(&command.program)
            .args(&command.args)
            .current_dir(repo_root)
            .envs(native_env.iter().map(|(key, value)| (key, value)))
            .envs(command.env.iter().map(|(key, value)| (key, value)))
            .output()
            .map_err(|source| ReleaseError::CommandIo {
                repo_root: repo_root.to_path_buf(),
                program: command.program.clone(),
                args: command.args.clone(),
                source,
            })?;

        let combined_output = command_output(&output.stdout, &output.stderr);
        let status = audit_status(
            &command,
            output.status.code(),
            output.status.success(),
            &combined_output,
        );

        checks.push(DevAuditCheck {
            name: command.program.clone(),
            command: command.display(),
            status,
            output: combined_output,
        });
    }

    Ok(DevAuditReport::new(checks))
}

fn run_check_tasks(
    repo_root: &Path,
    tasks: Vec<DevCheckTask>,
) -> Result<DevCheckReport, ReleaseError> {
    let native_env = native_build_env();
    let started = Instant::now();
    let mut results = Vec::new();

    for task in tasks {
        let task_started = Instant::now();
        let output = Command::new(&task.command.program)
            .args(&task.command.args)
            .current_dir(repo_root)
            .envs(native_env.iter().map(|(key, value)| (key, value)))
            .envs(task.command.env.iter().map(|(key, value)| (key, value)))
            .output()
            .map_err(|source| ReleaseError::CommandIo {
                repo_root: repo_root.to_path_buf(),
                program: task.command.program.clone(),
                args: task.command.args.clone(),
                source,
            })?;

        results.push(DevCheckResult {
            task,
            status: if output.status.success() {
                DevCheckStatus::Passed
            } else {
                DevCheckStatus::Failed
            },
            output: command_output(&output.stdout, &output.stderr),
            elapsed_seconds: task_started.elapsed().as_secs_f64(),
        });
    }

    Ok(DevCheckReport::new(
        results,
        started.elapsed().as_secs_f64(),
    ))
}

fn audit_status(
    command: &HarnessCommand,
    code: Option<i32>,
    success: bool,
    output: &str,
) -> DevAuditStatus {
    if command.program == "uv"
        && command.args == ["tree", "--outdated"]
        && output.contains("latest:")
    {
        return DevAuditStatus::Findings;
    }

    if success {
        DevAuditStatus::Clean
    } else if code == Some(1) {
        DevAuditStatus::Findings
    } else {
        DevAuditStatus::Failed
    }
}

fn run_commands(repo_root: &Path, commands: Vec<HarnessCommand>) -> Result<(), ReleaseError> {
    let native_env = native_build_env();
    for command in commands {
        let status = Command::new(&command.program)
            .args(&command.args)
            .current_dir(repo_root)
            .envs(native_env.iter().map(|(key, value)| (key, value)))
            .envs(command.env.iter().map(|(key, value)| (key, value)))
            .status()
            .map_err(|source| ReleaseError::CommandIo {
                repo_root: repo_root.to_path_buf(),
                program: command.program.clone(),
                args: command.args.clone(),
                source,
            })?;
        if !status.success() {
            return Err(ReleaseError::CommandFailed {
                repo_root: repo_root.to_path_buf(),
                program: command.program,
                args: command.args,
                code: status.code(),
            });
        }
    }
    Ok(())
}

impl HarnessCommand {
    fn display(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl DevCheckCommand {
    #[must_use]
    pub fn display(&self) -> String {
        let mut parts = self
            .env
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>();
        parts.push(self.program.clone());
        parts.extend(self.args.iter().cloned());
        parts.join(" ")
    }
}

impl From<HarnessCommand> for DevCheckCommand {
    fn from(command: HarnessCommand) -> Self {
        Self {
            program: command.program,
            args: command.args,
            env: command.env,
        }
    }
}

impl From<HarnessCommand> for DevPlannedCommand {
    fn from(command: HarnessCommand) -> Self {
        Self {
            command: DevCheckCommand::from(command),
        }
    }
}

fn command_output(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(stderr).trim().to_owned();

    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => String::new(),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("{stdout}\n{stderr}"),
    }
}

fn remove_obsolete_rust_toolchains(repo_root: &Path, pinned: &str) -> Result<(), ReleaseError> {
    for toolchain in obsolete_rust_toolchains(repo_root, pinned) {
        run_commands(
            repo_root,
            vec![command("rustup", ["toolchain", "uninstall", &toolchain])],
        )?;
    }

    Ok(())
}

fn remove_cargo_tools_target_dir(repo_root: &Path) -> Result<(), ReleaseError> {
    let path = repo_root.join("target/cargo-tools");
    match std::fs::remove_dir_all(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(ReleaseError::Io { path, source }),
    }
}

fn native_build_env() -> Vec<(String, String)> {
    let mut env = Vec::new();

    if std::env::var_os("PKG_CONFIG").is_none()
        && let Some(pkg_config) = pkg_config_path()
    {
        env.push(("PKG_CONFIG".to_owned(), pkg_config));
    }

    if std::env::var_os("OPENSSL_DIR").is_none()
        && let Some(openssl_dir) = brew_prefix("openssl@3")
    {
        env.push(("OPENSSL_DIR".to_owned(), openssl_dir));
    }

    env
}

fn verify_native_build_prerequisites() -> Result<(), ReleaseError> {
    let mut missing = Vec::new();
    if pkg_config_path().is_none() {
        missing.push("pkg-config/pkgconf is required to build pinned Cargo maintenance tools; install it with `brew install pkgconf` on macOS".to_owned());
    }

    if cfg!(target_os = "macos") && brew_prefix("openssl@3").is_none() {
        missing.push("OpenSSL is required to build pinned Cargo maintenance tools; install it with `brew install openssl@3` on macOS".to_owned());
    }

    if missing.is_empty() {
        return Ok(());
    }

    Err(ReleaseError::RepositoryPolicyViolation {
        details: missing.join("\n"),
    })
}

fn pkg_config_path() -> Option<String> {
    executable_path("pkg-config")
        .or_else(|| executable_path("pkgconf"))
        .or_else(|| brew_executable("pkgconf", "pkgconf"))
        .or_else(|| brew_executable("pkg-config", "pkgconf"))
}

fn executable_path(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .map(|candidate| candidate.display().to_string())
}

fn brew_executable(formula: &str, name: &str) -> Option<String> {
    let path = Path::new(&brew_prefix(formula)?).join("bin").join(name);
    if path.is_file() {
        return Some(path.display().to_string());
    }
    None
}

fn brew_prefix(formula: &str) -> Option<String> {
    let output = Command::new("brew")
        .args(["--prefix", formula])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if path.is_empty() || !Path::new(&path).exists() {
        None
    } else {
        Some(path)
    }
}

fn check_pkg_config() -> DevDoctorCheck {
    if let Some(path) = pkg_config_path() {
        return DevDoctorCheck {
            tool: "pkg-config".to_owned(),
            expected: "pkg-config or pkgconf on PATH".to_owned(),
            actual: Some(path),
            status: DevToolStatus::Ok,
        };
    }

    DevDoctorCheck {
        tool: "pkg-config".to_owned(),
        expected: "pkg-config or pkgconf on PATH".to_owned(),
        actual: None,
        status: DevToolStatus::Missing,
    }
}

fn check_homebrew_openssl() -> DevDoctorCheck {
    if let Some(path) = brew_prefix("openssl@3") {
        return DevDoctorCheck {
            tool: "openssl@3".to_owned(),
            expected: "Homebrew openssl@3".to_owned(),
            actual: Some(path),
            status: DevToolStatus::Ok,
        };
    }

    DevDoctorCheck {
        tool: "openssl@3".to_owned(),
        expected: "Homebrew openssl@3".to_owned(),
        actual: None,
        status: DevToolStatus::Missing,
    }
}

fn read_toml(path: &Path) -> Result<Value, ReleaseError> {
    let contents = read_to_string(path)?;
    contents
        .parse::<toml::Table>()
        .map(Value::Table)
        .map_err(|source| ReleaseError::InvalidInput {
            path: path.to_path_buf(),
            message: source.to_string(),
        })
}

fn read_to_string(path: &Path) -> Result<String, ReleaseError> {
    std::fs::read_to_string(path).map_err(|source| ReleaseError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn required_string(document: &Value, path: &[&str], source: &Path) -> Result<String, ReleaseError> {
    required_value(document, path, source)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| invalid_type(source, path, "string"))
}

fn required_integer(document: &Value, path: &[&str], source: &Path) -> Result<i64, ReleaseError> {
    required_value(document, path, source)?
        .as_integer()
        .ok_or_else(|| invalid_type(source, path, "integer"))
}

fn required_string_array(
    document: &Value,
    path: &[&str],
    source: &Path,
) -> Result<Vec<String>, ReleaseError> {
    required_value(document, path, source)?
        .as_array()
        .ok_or_else(|| invalid_type(source, path, "array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| invalid_type(source, path, "string array"))
        })
        .collect()
}

fn required_value<'a>(
    document: &'a Value,
    path: &[&str],
    source: &Path,
) -> Result<&'a Value, ReleaseError> {
    let mut current = document;
    for segment in path {
        current = current
            .get(*segment)
            .ok_or_else(|| missing_value(source, path))?;
    }
    Ok(current)
}

fn invalid_type(source: &Path, path: &[&str], expected: &str) -> ReleaseError {
    ReleaseError::InvalidInput {
        path: source.to_path_buf(),
        message: format!("{} must be a {expected}", path.join(".")),
    }
}

fn missing_value(source: &Path, path: &[&str]) -> ReleaseError {
    ReleaseError::InvalidInput {
        path: source.to_path_buf(),
        message: format!("missing {}", path.join(".")),
    }
}

fn require_equal(violations: &mut Vec<String>, field: &str, expected: &str, actual: &str) {
    if actual == expected {
        return;
    }
    violations.push(format!("{field} must be {expected:?}, found {actual:?}"));
}

fn require_array_contains(
    violations: &mut Vec<String>,
    field: &str,
    values: &[String],
    expected: &str,
) {
    if values.iter().any(|value| value == expected) {
        return;
    }
    violations.push(format!("{field} must contain {expected:?}"));
}

fn require_contains(violations: &mut Vec<String>, field: &str, contents: &str, expected: &str) {
    if contents.contains(expected) {
        return;
    }
    violations.push(format!("{field} must contain {expected:?}"));
}

fn minor_version(version: &str) -> String {
    let mut parts = version.split('.');
    match (parts.next(), parts.next()) {
        (Some(major), Some(minor)) => format!("{major}.{minor}"),
        _ => version.to_owned(),
    }
}
