use std::path::Path;
use std::process::Command;

use toml::Value;

use crate::ReleaseError;

const DEV_TOOLS_PATH: &str = ".github/dev-tools.toml";

#[derive(Debug, Eq, PartialEq)]
pub struct DevDoctorReport {
    pub checks: Vec<DevDoctorCheck>,
}

impl DevDoctorReport {
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.checks
            .iter()
            .all(|check| check.status == DevToolStatus::Ok)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DevDoctorCheck {
    pub tool: String,
    pub expected: String,
    pub actual: Option<String>,
    pub status: DevToolStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DevToolStatus {
    Ok,
    Missing,
    Mismatched,
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
    cargo_audit_rev: String,
    cargo_deny: String,
}

#[derive(Debug)]
struct HarnessCommand {
    program: String,
    args: Vec<String>,
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
    let checks = vec![
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
            "cargo-audit",
            "cargo",
            &["audit", "--version"],
        ),
    ];

    let report = DevDoctorReport { checks };
    let cleanup = obsolete_rust_toolchains(repo_root, &manifest.rust);
    if cleanup.is_empty() {
        return Ok(report);
    }

    let mut checks = report.checks;
    checks.push(DevDoctorCheck {
        tool: "rustup cleanup".to_owned(),
        expected: format!(
            "only {} and explicitly installed non-project toolchains",
            manifest.rust
        ),
        actual: Some(cleanup.join(", ")),
        status: DevToolStatus::Mismatched,
    });
    Ok(DevDoctorReport { checks })
}

pub fn setup_dev_environment(repo_root: &Path) -> Result<(), ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    remove_stale_cargo_binaries(&[
        "cargo-outdated",
        "cargo-deny",
        "cargo-audit",
        "cargo-audit-audit",
    ])?;
    run_commands(repo_root, setup_commands(&manifest))
}

pub fn update_dev_dependencies(repo_root: &Path) -> Result<(), ReleaseError> {
    let manifest = read_manifest(repo_root)?;
    run_commands(repo_root, update_commands(&manifest))
}

pub fn audit_latest_dev_dependencies(repo_root: &Path) -> Result<(), ReleaseError> {
    run_commands(repo_root, audit_commands())
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
        cargo_audit_rev: required_string(
            &document,
            &["rust-maintenance", "cargo-audit-rev"],
            &path,
        )?,
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
        ".github/actions/setup-python-uv/action.yml setup-uv version",
        &contents,
        &format!("version: \"{}\"", manifest.uv),
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
        ".github/actions/setup-rust-maintenance-tools/action.yml cargo-audit-rev",
        &contents,
        &format!("default: \"{}\"", manifest.cargo_audit_rev),
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
        command(
            "cargo",
            [
                "install",
                "--force",
                "--locked",
                &format!("cargo-outdated@{}", manifest.cargo_outdated),
            ],
        ),
        command(
            "cargo",
            [
                "install",
                "--force",
                "--locked",
                &format!("cargo-deny@{}", manifest.cargo_deny),
            ],
        ),
        command(
            "cargo",
            [
                "install",
                "--force",
                "--git",
                "https://github.com/RustSec/rustsec.git",
                "--rev",
                &manifest.cargo_audit_rev,
                "--locked",
                "cargo-audit",
            ],
        ),
    ]
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

fn audit_commands() -> Vec<HarnessCommand> {
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

fn command<const N: usize>(program: &str, args: [&str; N]) -> HarnessCommand {
    HarnessCommand {
        program: program.to_owned(),
        args: args.into_iter().map(ToOwned::to_owned).collect(),
    }
}

fn run_commands(repo_root: &Path, commands: Vec<HarnessCommand>) -> Result<(), ReleaseError> {
    for command in commands {
        let status = Command::new(&command.program)
            .args(&command.args)
            .current_dir(repo_root)
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
