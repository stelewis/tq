use std::path::PathBuf;

fn labels(values: &[&str]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}

fn paths(values: &[&str]) -> Vec<PathBuf> {
    values.iter().map(PathBuf::from).collect()
}

const fn check_input<'a>(
    labels: &'a [String],
    changed_files: &'a [PathBuf],
    version_updated: bool,
    changelog_updated: bool,
    runtime_dependency_changed: bool,
) -> tq_release::ReleaseIntentCheck<'a> {
    tq_release::ReleaseIntentCheck {
        labels,
        changed_files,
        version_updated,
        changelog_updated,
        runtime_dependency_changed,
    }
}

#[test]
fn verify_release_intent_allows_release_none_for_repo_only_changes() {
    let labels = labels(&["release:none"]);
    let changed_files = paths(&[
        ".github/workflows/ci.yml",
        "crates/tq-release/src/main.rs",
        "docs/developer/releasing.md",
        "pyproject.toml",
    ]);

    tq_release::verify_release_intent(check_input(&labels, &changed_files, false, false, false))
        .expect("repo-only changes should allow release:none");
}

#[test]
fn verify_release_intent_rejects_release_none_for_shipped_runtime_source_changes() {
    let labels = labels(&["release:none"]);
    let changed_files = paths(&["crates/tq-cli/src/cli.rs"]);

    let error = tq_release::verify_release_intent(check_input(
        &labels,
        &changed_files,
        false,
        false,
        false,
    ))
    .expect_err("shipped runtime source changes should reject release:none");

    assert!(error.to_string().contains("shipped runtime source changes"));
    assert!(error.to_string().contains("crates/tq-cli/src/cli.rs"));
}

#[test]
fn verify_release_intent_rejects_release_none_for_tq_discovery_source_changes() {
    let labels = labels(&["release:none"]);
    let changed_files = paths(&["crates/tq-discovery/src/discover.rs"]);

    let error = tq_release::verify_release_intent(check_input(
        &labels,
        &changed_files,
        false,
        false,
        false,
    ))
    .expect_err("transitive shipped runtime source changes should reject release:none");

    assert!(error.to_string().contains("shipped runtime source changes"));
    assert!(
        error
            .to_string()
            .contains("crates/tq-discovery/src/discover.rs")
    );
}

#[test]
fn verify_release_intent_rejects_release_none_for_contract_reference_docs() {
    let labels = labels(&["release:none"]);
    let changed_files = paths(&["docs/reference/cli.md"]);

    let error = tq_release::verify_release_intent(check_input(
        &labels,
        &changed_files,
        false,
        false,
        false,
    ))
    .expect_err("contract reference docs should reject release:none");

    assert!(
        error
            .to_string()
            .contains("contract policy or reference doc changes")
    );
    assert!(error.to_string().contains("docs/reference/cli.md"));
}

#[test]
fn verify_release_intent_rejects_release_none_for_runtime_dependency_changes() {
    let labels = labels(&["release:none"]);
    let changed_files = paths(&["Cargo.toml"]);

    let error =
        tq_release::verify_release_intent(check_input(&labels, &changed_files, false, false, true))
            .expect_err("runtime dependency changes should reject release:none");

    assert!(error.to_string().contains("runtime dependency change"));
}

#[test]
fn verify_release_intent_accepts_release_patch_when_version_and_changelog_are_updated() {
    let labels = labels(&["release:patch"]);
    let changed_files = paths(&["crates/tq-cli/src/cli.rs"]);

    tq_release::verify_release_intent(check_input(&labels, &changed_files, true, true, false))
        .expect("release:patch should pass with version and changelog updates");
}

#[test]
fn verify_release_intent_rejects_release_minor_when_version_or_changelog_is_missing() {
    let labels = labels(&["release:minor"]);
    let changed_files = paths(&["crates/tq-core/src/lib.rs"]);

    let error =
        tq_release::verify_release_intent(check_input(&labels, &changed_files, true, false, false))
            .expect_err("release:minor should fail when changelog update is missing");

    assert!(
        error
            .to_string()
            .contains("release:minor requires a new top CHANGELOG.md release heading")
    );
}

#[test]
fn verify_release_intent_rejects_missing_release_label_before_semantic_checks() {
    let labels = labels(&[]);
    let changed_files = paths(&["crates/tq-cli/src/cli.rs"]);

    let error = tq_release::verify_release_intent(check_input(
        &labels,
        &changed_files,
        false,
        false,
        false,
    ))
    .expect_err("missing release label should fail");

    let message = error.to_string();
    assert!(message.contains("must declare exactly one release intent label"));
    assert!(!message.contains("shipped runtime source changes"));
}

#[test]
fn verify_release_intent_rejects_multiple_release_labels_before_semantic_checks() {
    let labels = labels(&["release:none", "release:patch"]);
    let changed_files = paths(&["docs/developer/releasing.md"]);

    let error =
        tq_release::verify_release_intent(check_input(&labels, &changed_files, true, true, false))
            .expect_err("multiple release labels should fail");

    let message = error.to_string();
    assert!(message.contains("found release:none, release:patch"));
    assert!(!message.contains("workspace version update"));
}

#[test]
fn verify_release_intent_rejects_unknown_release_prefixed_labels() {
    let labels = labels(&["release:none", "release:major"]);
    let changed_files = paths(&["docs/developer/releasing.md"]);

    let error = tq_release::verify_release_intent(check_input(
        &labels,
        &changed_files,
        false,
        false,
        false,
    ))
    .expect_err("unknown release-prefixed labels should fail");

    let message = error.to_string();
    assert!(message.contains("unknown release intent labels"));
    assert!(message.contains("release:major"));
}
