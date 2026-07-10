use std::path::Path;
use std::process::Command;

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

pub fn update_dev_dependencies(repo_root: &Path) -> Result<(), ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    update_rust_toolchain_pin(repo_root, &manifest)?;
    run_commands(repo_root, update_commands(&manifest))
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
    if !cargo_subcommand_matches("outdated", &manifest.cargo_outdated) {
        remove_stale_cargo_binaries(&["cargo-outdated"])?;
        run_commands(
            repo_root,
            vec![cargo_install_command(
                repo_root,
                [
                    "install",
                    "--force",
                    "--locked",
                    &format!("cargo-outdated@{}", manifest.cargo_outdated),
                ],
            )],
        )?;
    }

    if !cargo_subcommand_matches("deny", &manifest.cargo_deny) {
        remove_stale_cargo_binaries(&["cargo-deny"])?;
        run_commands(
            repo_root,
            vec![cargo_install_command(
                repo_root,
                [
                    "install",
                    "--force",
                    "--locked",
                    &format!("cargo-deny@{}", manifest.cargo_deny),
                ],
            )],
        )?;
    }

    if !cargo_subcommand_matches("audit", &manifest.cargo_audit) {
        remove_stale_cargo_binaries(&["cargo-audit", "cargo-audit-audit"])?;
        run_commands(
            repo_root,
            vec![cargo_install_command(
                repo_root,
                [
                    "install",
                    "--force",
                    "--locked",
                    &format!("cargo-audit@{}", manifest.cargo_audit),
                ],
            )],
        )?;
    }

    Ok(())
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

fn update_commands(manifest: &DevToolsManifest) -> Vec<HarnessCommand> {
    vec![
        command("rustup", ["update", &manifest.rust]),
        command("mise", ["install"]),
        command("uv", ["lock", "--upgrade"]),
        command("npm", ["update"]),
        command("uv", ["run", "prek", "autoupdate", "--freeze"]),
    ]
}

fn update_rust_toolchain_pin(
    repo_root: &Path,
    manifest: &DevToolsManifest,
) -> Result<(), ReleaseError> {
    let Some(latest) = latest_stable_rust() else {
        return Ok(());
    };
    if latest == manifest.rust {
        return Ok(());
    }

    replace_once(
        &repo_root.join(DEV_TOOLS_PATH),
        &format!("rust = \"{}\"", manifest.rust),
        &format!("rust = \"{latest}\""),
    )?;
    replace_once(
        &repo_root.join("rust-toolchain.toml"),
        &format!("channel = \"{}\"", manifest.rust),
        &format!("channel = \"{latest}\""),
    )?;
    replace_once(
        &repo_root.join("Cargo.toml"),
        &format!("rust-version = \"{}\"", minor_version(&manifest.rust)),
        &format!("rust-version = \"{}\"", minor_version(&latest)),
    )
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
