use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;

use toml::{Table, Value};

use crate::PrReleaseIntentCheck;
use crate::ReleaseError;
use crate::ReleaseIntentCheck;
use crate::release_intent::verify_release_intent;

const CHANGELOG_PATH: &str = "CHANGELOG.md";
const LOCKFILE_PATH: &str = "Cargo.lock";
const ROOT_MANIFEST_PATH: &str = "Cargo.toml";
const SHIPPED_RUNTIME_ROOT_CRATE: &str = "tq-cli";

type DirectExternalPackages = BTreeSet<String>;
type Fingerprint = BTreeMap<String, BTreeSet<String>>;

#[derive(Debug, Eq, PartialEq)]
struct RuntimeDependencySnapshot {
    lock_fingerprint: Fingerprint,
    manifest_fingerprint: Fingerprint,
}

#[derive(Debug)]
struct LockDependencyRef {
    name: String,
    version: Option<String>,
}

#[derive(Debug)]
struct LockPackage {
    dependencies: Vec<LockDependencyRef>,
    name: String,
    source: Option<String>,
    version: String,
}

#[derive(Debug)]
struct MemberManifest {
    manifest: Table,
    path: PathBuf,
}

#[derive(Debug)]
struct ResolvedDependency {
    fingerprint: String,
    package_name: String,
}

pub fn verify_pr_release_intent(input: PrReleaseIntentCheck<'_>) -> Result<(), ReleaseError> {
    let PrReleaseIntentCheck {
        repo_root,
        base_ref,
        head_ref,
        labels,
    } = input;
    let merge_base = git_stdout(repo_root, &["merge-base", base_ref, head_ref])?;
    let merge_base = merge_base.trim();
    if merge_base.is_empty() {
        return Err(ReleaseError::RepositoryPolicyViolation {
            details: format!("could not determine merge base for refs {base_ref} and {head_ref}"),
        });
    }

    let changed_files = changed_files_between(repo_root, merge_base, head_ref)?;
    let version_updated =
        workspace_version_at(repo_root, merge_base)? != workspace_version_at(repo_root, head_ref)?;
    let changelog_updated = top_changelog_heading_at(repo_root, merge_base)?
        != top_changelog_heading_at(repo_root, head_ref)?;
    let runtime_dependency_changed = runtime_dependency_snapshot_at(repo_root, merge_base)?
        != runtime_dependency_snapshot_at(repo_root, head_ref)?;

    verify_release_intent(ReleaseIntentCheck {
        labels,
        changed_files: &changed_files,
        version_updated,
        changelog_updated,
        runtime_dependency_changed,
    })
}

fn changed_files_between(
    repo_root: &Path,
    base_ref: &str,
    head_ref: &str,
) -> Result<Vec<PathBuf>, ReleaseError> {
    let stdout = git_stdout(repo_root, &["diff", "--name-only", base_ref, head_ref])?;
    Ok(stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect())
}

fn runtime_dependency_snapshot_at(
    repo_root: &Path,
    git_ref: &str,
) -> Result<RuntimeDependencySnapshot, ReleaseError> {
    let root_manifest_path = repo_root.join(ROOT_MANIFEST_PATH);
    let root_manifest = parse_toml_table(
        &git_file_contents(repo_root, git_ref, ROOT_MANIFEST_PATH)?,
        &root_manifest_path,
    )?;
    let workspace_dependencies = workspace_dependencies(&root_manifest, &root_manifest_path)?;
    let members = workspace_members(&root_manifest, &root_manifest_path)?;
    let member_manifests = load_member_manifests(repo_root, git_ref, &members)?;
    let (manifest_fingerprint, direct_external_packages) =
        runtime_manifest_fingerprint(workspace_dependencies, &member_manifests)?;

    let lockfile_path = repo_root.join(LOCKFILE_PATH);
    let lock_packages = parse_lock_packages(
        &git_file_contents(repo_root, git_ref, LOCKFILE_PATH)?,
        &lockfile_path,
    )?;

    Ok(RuntimeDependencySnapshot {
        lock_fingerprint: runtime_lock_fingerprint(&lock_packages, &direct_external_packages),
        manifest_fingerprint,
    })
}

fn workspace_version_at(repo_root: &Path, git_ref: &str) -> Result<String, ReleaseError> {
    let manifest_path = repo_root.join(ROOT_MANIFEST_PATH);
    let manifest = parse_toml_table(
        &git_file_contents(repo_root, git_ref, ROOT_MANIFEST_PATH)?,
        &manifest_path,
    )?;

    manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(Value::as_table)
        .and_then(|package| package.get("version"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            release_intent_input_error(&manifest_path, "missing workspace.package.version")
        })
}

fn top_changelog_heading_at(
    repo_root: &Path,
    git_ref: &str,
) -> Result<Option<String>, ReleaseError> {
    Ok(git_file_contents(repo_root, git_ref, CHANGELOG_PATH)?
        .lines()
        .find(|line| line.starts_with("## ["))
        .map(ToOwned::to_owned))
}

fn load_member_manifests(
    repo_root: &Path,
    git_ref: &str,
    members: &[PathBuf],
) -> Result<BTreeMap<String, MemberManifest>, ReleaseError> {
    let mut manifests = BTreeMap::new();

    for member in members {
        let manifest_relative_path = member.join(ROOT_MANIFEST_PATH);
        let manifest_path = repo_root.join(&manifest_relative_path);
        let manifest = parse_toml_table(
            &git_file_contents(
                repo_root,
                git_ref,
                &manifest_relative_path.to_string_lossy(),
            )?,
            &manifest_path,
        )?;
        let crate_name = manifest
            .get("package")
            .and_then(Value::as_table)
            .and_then(|package| package.get("name"))
            .and_then(Value::as_str)
            .ok_or_else(|| release_intent_input_error(&manifest_path, "missing package.name"))?;

        manifests.insert(
            crate_name.to_owned(),
            MemberManifest {
                manifest,
                path: manifest_path,
            },
        );
    }

    Ok(manifests)
}

fn runtime_manifest_fingerprint(
    workspace_dependencies: &Table,
    member_manifests: &BTreeMap<String, MemberManifest>,
) -> Result<(Fingerprint, DirectExternalPackages), ReleaseError> {
    let internal_crates = member_manifests.keys().cloned().collect::<BTreeSet<_>>();
    let mut direct_external_packages = BTreeSet::new();
    let mut manifest_fingerprint = BTreeMap::new();
    let mut queued = VecDeque::from([SHIPPED_RUNTIME_ROOT_CRATE.to_owned()]);
    let mut visited = BTreeSet::new();

    while let Some(crate_name) = queued.pop_front() {
        if !visited.insert(crate_name.clone()) {
            continue;
        }

        let member = member_manifests.get(&crate_name).ok_or_else(|| {
            release_intent_input_error(
                Path::new(ROOT_MANIFEST_PATH),
                &format!("missing shipped runtime crate manifest for {crate_name}"),
            )
        })?;

        let Some(dependencies) = member
            .manifest
            .get("dependencies")
            .and_then(Value::as_table)
        else {
            continue;
        };

        for (dependency_key, dependency_value) in dependencies {
            let resolved = resolve_dependency(
                dependency_key,
                dependency_value,
                workspace_dependencies,
                &member.path,
            )?;

            manifest_fingerprint
                .entry(crate_name.clone())
                .or_insert_with(BTreeSet::new)
                .insert(resolved.fingerprint.clone());

            if internal_crates.contains(&resolved.package_name) {
                queued.push_back(resolved.package_name);
            } else {
                direct_external_packages.insert(resolved.package_name);
            }
        }
    }

    Ok((manifest_fingerprint, direct_external_packages))
}

fn resolve_dependency(
    dependency_key: &str,
    dependency_value: &Value,
    workspace_dependencies: &Table,
    manifest_path: &Path,
) -> Result<ResolvedDependency, ReleaseError> {
    if dependency_value
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let workspace_dependency = workspace_dependencies.get(dependency_key).ok_or_else(|| {
            release_intent_input_error(
                manifest_path,
                &format!("missing workspace dependency entry for {dependency_key}"),
            )
        })?;

        return resolve_dependency(
            dependency_key,
            workspace_dependency,
            workspace_dependencies,
            manifest_path,
        );
    }

    let package_name = dependency_value
        .as_table()
        .and_then(|table| table.get("package"))
        .and_then(Value::as_str)
        .unwrap_or(dependency_key)
        .to_owned();

    Ok(ResolvedDependency {
        fingerprint: format!("{package_name}|{}", canonical_toml_value(dependency_value)),
        package_name,
    })
}

fn parse_lock_packages(contents: &str, path: &Path) -> Result<Vec<LockPackage>, ReleaseError> {
    let lockfile = parse_toml_table(contents, path)?;
    let packages = lockfile
        .get("package")
        .and_then(Value::as_array)
        .ok_or_else(|| release_intent_input_error(path, "missing package array"))?;

    packages
        .iter()
        .map(|package| {
            let package = package.as_table().ok_or_else(|| {
                release_intent_input_error(path, "lockfile package entry must be a table")
            })?;
            let name = package.get("name").and_then(Value::as_str).ok_or_else(|| {
                release_intent_input_error(path, "lockfile package is missing name")
            })?;
            let version = package
                .get("version")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    release_intent_input_error(path, "lockfile package is missing version")
                })?;
            let source = package
                .get("source")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);

            let dependencies = match package.get("dependencies") {
                Some(Value::Array(dependencies)) => dependencies
                    .iter()
                    .map(|dependency| {
                        let dependency = dependency.as_str().ok_or_else(|| {
                            release_intent_input_error(
                                path,
                                "lockfile dependency entries must be strings",
                            )
                        })?;
                        parse_lock_dependency(dependency, path)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                Some(_) => {
                    return Err(release_intent_input_error(
                        path,
                        "lockfile dependencies must be an array",
                    ));
                }
                None => Vec::new(),
            };

            Ok(LockPackage {
                dependencies,
                name: name.to_owned(),
                source,
                version: version.to_owned(),
            })
        })
        .collect()
}

fn parse_lock_dependency(dependency: &str, path: &Path) -> Result<LockDependencyRef, ReleaseError> {
    let mut tokens = dependency.split_whitespace();
    let name = tokens.next().ok_or_else(|| {
        release_intent_input_error(path, "lockfile dependency entry must not be empty")
    })?;
    let version = tokens
        .next()
        .filter(|token| {
            token
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_digit())
        })
        .map(ToOwned::to_owned);

    Ok(LockDependencyRef {
        name: name.to_owned(),
        version,
    })
}

fn runtime_lock_fingerprint(
    packages: &[LockPackage],
    direct_external_packages: &BTreeSet<String>,
) -> Fingerprint {
    let mut packages_by_name = BTreeMap::<String, Vec<&LockPackage>>::new();
    for package in packages {
        packages_by_name
            .entry(package.name.clone())
            .or_default()
            .push(package);
    }

    let mut fingerprint = BTreeMap::<String, BTreeSet<String>>::new();
    let mut queued = VecDeque::new();
    let mut visited = BTreeSet::new();

    for direct_external_package in direct_external_packages {
        if let Some(candidates) = packages_by_name.get(direct_external_package) {
            queued.extend(candidates.iter().copied());
        }
    }

    while let Some(package) = queued.pop_front() {
        let package_id = package_fingerprint(package);
        if !visited.insert(package_id) {
            continue;
        }

        fingerprint
            .entry(package.name.clone())
            .or_default()
            .insert(package_fingerprint(package));

        for dependency in &package.dependencies {
            if let Some(candidates) = packages_by_name.get(&dependency.name) {
                for candidate in candidates {
                    if dependency
                        .version
                        .as_deref()
                        .is_none_or(|version| candidate.version == version)
                    {
                        queued.push_back(candidate);
                    }
                }
            }
        }
    }

    fingerprint
}

fn package_fingerprint(package: &LockPackage) -> String {
    package.source.as_ref().map_or_else(
        || format!("{}@{}", package.name, package.version),
        |source| format!("{}@{}|{source}", package.name, package.version),
    )
}

fn workspace_dependencies<'a>(manifest: &'a Table, path: &Path) -> Result<&'a Table, ReleaseError> {
    manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Value::as_table)
        .ok_or_else(|| release_intent_input_error(path, "missing workspace.dependencies"))
}

fn workspace_members(manifest: &Table, path: &Path) -> Result<Vec<PathBuf>, ReleaseError> {
    let members = manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .ok_or_else(|| release_intent_input_error(path, "missing workspace.members"))?;

    members
        .iter()
        .map(|member| {
            member.as_str().map(PathBuf::from).ok_or_else(|| {
                release_intent_input_error(path, "workspace.members entries must be strings")
            })
        })
        .collect()
}

fn parse_toml_table(contents: &str, path: &Path) -> Result<Table, ReleaseError> {
    contents
        .parse::<Table>()
        .map_err(|source| release_intent_input_error(path, &format!("invalid TOML: {source}")))
}

fn git_file_contents(
    repo_root: &Path,
    git_ref: &str,
    relative_path: &str,
) -> Result<String, ReleaseError> {
    git_stdout(repo_root, &["show", &format!("{git_ref}:{relative_path}")])
}

fn git_stdout(repo_root: &Path, args: &[&str]) -> Result<String, ReleaseError> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .output()
        .map_err(|source| ReleaseError::GitIo {
            repo_root: repo_root.to_path_buf(),
            args: args.join(" "),
            source,
        })?;

    if !output.status.success() {
        return Err(ReleaseError::Git {
            repo_root: repo_root.to_path_buf(),
            args: args.join(" "),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn canonical_toml_value(value: &Value) -> String {
    match value {
        Value::Array(array) => format!(
            "[{}]",
            array
                .iter()
                .map(canonical_toml_value)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Boolean(boolean) => format!("bool:{boolean}"),
        Value::Datetime(datetime) => format!("datetime:{datetime}"),
        Value::Float(float) => format!("float:{float}"),
        Value::Integer(integer) => format!("int:{integer}"),
        Value::String(string) => format!("string:{string}"),
        Value::Table(table) => {
            let mut entries = table
                .iter()
                .map(|(key, table_value)| format!("{key}={}", canonical_toml_value(table_value)))
                .collect::<Vec<_>>();
            entries.sort_unstable();
            format!("{{{}}}", entries.join(","))
        }
    }
}

fn release_intent_input_error(path: &Path, message: &str) -> ReleaseError {
    ReleaseError::ReleaseIntentInput {
        path: path.to_path_buf(),
        message: message.to_owned(),
    }
}
