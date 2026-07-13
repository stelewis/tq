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

#[test]
fn publish_checks_out_validated_ci_head_sha_not_release_tag_ref() {
    let publish = fs::read_to_string(repo_root().join(".github/workflows/publish.yml"))
        .expect("publish workflow should be readable");

    assert!(publish.contains("source_head_sha: ${{ steps.release.outputs.source_head_sha }}"));
    assert!(publish.contains("echo \"source_head_sha=$head_sha\""));
    assert!(publish.contains("} >> \"$GITHUB_OUTPUT\""));
    assert!(publish.contains("ref: ${{ needs.prepare.outputs.source_head_sha }}"));
    assert!(!publish.contains("ref: refs/tags/${{ needs.prepare.outputs.release_tag }}"));
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
