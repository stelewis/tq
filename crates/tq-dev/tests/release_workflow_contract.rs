use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate directory should have a parent")
        .parent()
        .expect("crates directory should have a parent")
        .to_path_buf()
}

fn job<'a>(workflow: &'a str, name: &str, next: Option<&str>) -> &'a str {
    let start = workflow
        .find(&format!("  {name}:\n"))
        .expect("job must exist");
    let end = next
        .and_then(|next_name| workflow[start + 1..].find(&format!("\n  {next_name}:\n")))
        .map_or(workflow.len(), |offset| start + 1 + offset);
    &workflow[start..end]
}

#[test]
fn publish_workflow_never_checks_out_triggering_code() {
    let publish = fs::read_to_string(repo_root().join(".github/workflows/publish.yml"))
        .expect("publish workflow should be readable");

    assert!(publish.contains("source_head_sha: ${{ steps.release.outputs.source_head_sha }}"));
    assert!(publish.contains("echo \"source_head_sha=$head_sha\""));
    assert!(!publish.contains("actions/checkout@"));
    assert!(!publish.contains("uses: ./"));
    assert!(!publish.contains("cargo dev"));
}

#[test]
fn candidate_execution_is_separated_from_publish_credentials() {
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/publish.yml"))
        .expect("publish workflow should be readable");
    let verify = job(&workflow, "verify", Some("publish"));
    let publish = job(&workflow, "publish", Some("post-publish"));
    let post_publish = job(&workflow, "post-publish", None);

    assert!(verify.contains("contents: read"));
    assert!(verify.contains("attestations: read"));
    assert!(!verify.contains("contents: write"));
    assert!(!verify.contains("id-token: write"));
    assert!(verify.contains("Smoke check Linux wheel and sdist entrypoints"));
    assert!(verify.contains("Smoke check fixture from Linux wheel"));

    assert!(publish.contains("needs: [prepare, verify]"));
    assert!(publish.contains("contents: write"));
    assert!(publish.contains("id-token: write"));
    assert!(publish.contains("uv publish dist/*.whl dist/*.tar.gz"));
    assert!(!publish.contains("uvx --from"));
    assert!(!publish.contains("twine check"));
    assert!(!publish.contains("tq check"));

    assert!(post_publish.contains("needs: [prepare, publish]"));
    assert!(post_publish.contains("contents: read"));
    assert!(!post_publish.contains("contents: write"));
    assert!(!post_publish.contains("id-token: write"));
    assert!(post_publish.contains("uvx --from \"tqlint==$RELEASE_TAG\""));
}

#[test]
fn publish_rechecks_tag_binding_and_artifact_attestations() {
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/publish.yml"))
        .expect("publish workflow should be readable");
    let prepare = job(&workflow, "prepare", Some("verify"));
    let publish = job(&workflow, "publish", Some("post-publish"));

    assert!(prepare.contains("Require immutable tag ruleset"));
    assert!(prepare.contains("(.bypass_actors | length) == 0"));
    assert!(prepare.contains("index(\"~ALL\")"));
    assert!(prepare.contains("index(\"update\")"));
    assert!(prepare.contains("index(\"deletion\")"));
    assert!(publish.contains("Reverify CI artifact attestations"));
    assert!(publish.contains("Require release tag to remain bound to validated revision"));
    assert!(publish.contains("SOURCE_HEAD_SHA: ${{ needs.prepare.outputs.source_head_sha }}"));
    assert!(publish.contains("object_sha\" != \"$SOURCE_HEAD_SHA"));

    let first_binding = publish
        .find("- name: Require release tag to remain bound to validated revision")
        .expect("first tag binding check must exist");
    let pypi_publish = publish
        .find("- name: Publish")
        .expect("PyPI publish step must exist");
    let second_binding = publish
        .find("- name: Recheck release tag binding")
        .expect("second tag binding check must exist");
    let github_release = publish
        .find("- name: Create GitHub release")
        .expect("GitHub release step must exist");
    assert!(first_binding < pypi_publish);
    assert!(pypi_publish < second_binding);
    assert!(second_binding < github_release);
}

#[test]
fn checkout_free_jobs_use_manifest_owned_tool_versions() {
    let root = repo_root();
    let workflow = fs::read_to_string(root.join(".github/workflows/publish.yml"))
        .expect("publish workflow should be readable");
    let manifest = tq_dev::manifest::DevToolsManifest::load(&root)
        .expect("developer tool manifest should load");

    let python_pin = format!("python-version: \"{}\"", manifest.python);
    let uv_pin = format!("version: \"{}\"", manifest.uv);
    assert_eq!(workflow.matches(&python_pin).count(), 2);
    assert_eq!(workflow.matches(&uv_pin).count(), 3);
}

#[test]
fn publish_is_serialized_by_release_tag() {
    let publish = fs::read_to_string(repo_root().join(".github/workflows/publish.yml"))
        .expect("publish workflow should be readable");

    assert!(publish.contains("concurrency:\n      group: publish-${{ needs.prepare.outputs.release_tag }}\n      cancel-in-progress: false"));
}

#[test]
fn release_attestation_waits_for_automation_quality_security_and_package_jobs() {
    let ci = fs::read_to_string(repo_root().join(".github/workflows/ci.yml"))
        .expect("CI workflow should be readable");

    assert!(ci.contains("needs: [automation-policy, commit-messages, hygiene, format, lint, tests, build, release-wheels, security, package-compatibility]"));
    assert!(ci.contains("tag-release-artifacts:\n    name: Tag release artifact attestation\n    needs: tag-release-validation"));
}

#[test]
fn tag_artifact_validation_is_separated_from_attestation_credentials() {
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/ci.yml"))
        .expect("CI workflow should be readable");
    let validation = job(
        &workflow,
        "tag-release-validation",
        Some("tag-release-artifacts"),
    );
    let attestation = job(
        &workflow,
        "tag-release-artifacts",
        Some("package-compatibility"),
    );

    assert!(validation.contains("name: Tag release artifact validation"));
    assert!(validation.contains("actions/checkout@"));
    assert!(validation.contains("uses: ./.github/actions/setup-rust"));
    assert!(validation.contains("cargo dev release verify-artifacts"));
    assert!(validation.contains("name: validated-dist-candidate"));
    assert!(!validation.contains("attestations: write"));
    assert!(!validation.contains("id-token: write"));

    assert!(attestation.contains("name: Tag release artifact attestation"));
    assert!(attestation.contains("attestations: write"));
    assert!(attestation.contains("id-token: write"));
    assert!(attestation.contains("name: validated-dist-candidate"));
    assert!(attestation.contains("actions/attest-build-provenance@"));
    assert!(attestation.contains("name: validated-dist"));
    assert!(!attestation.contains("actions/checkout@"));
    assert!(!attestation.contains("uses: ./"));
    assert!(!attestation.contains("cargo dev"));
}

#[test]
fn tagged_release_requires_exactly_one_validated_artifact() {
    let publish = fs::read_to_string(repo_root().join(".github/workflows/publish.yml"))
        .expect("publish workflow should be readable");
    let release_position = publish
        .find("- name: Resolve SemVer tag for successful CI run")
        .expect("release tag resolution step must exist");
    let artifact_position = publish
        .find("- name: Require validated release artifact")
        .expect("artifact requirement step must exist");

    assert!(release_position < artifact_position);
    assert!(publish.contains("if: ${{ steps.release.outputs.should_publish == 'true' }}"));
    assert!(publish.contains("Expected exactly one validated-dist artifact for tagged CI run"));
    assert!(!publish.contains("has no validated-dist artifact; skipping publish"));
}
