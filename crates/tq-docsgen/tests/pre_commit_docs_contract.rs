use std::fs;
use std::path::Path;

#[test]
fn generate_pre_commit_docs_updates_marked_sections_from_release_version() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cargo_toml_path = temp.path().join("Cargo.toml");
    let pyproject_toml_path = temp.path().join("pyproject.toml");
    let readme_path = temp.path().join("README.md");
    let quickstart_path = temp.path().join("docs/guide/quickstart.md");

    write(
        &cargo_toml_path,
        "[workspace]\n\n[workspace.package]\nversion = \"0.9.0\"\n",
    );
    write(
        &pyproject_toml_path,
        "[tool.commitizen]\ntag_format = \"v$version\"\n",
    );
    write(
        &readme_path,
        "# README\n\n<!-- BEGIN GENERATED:pre-commit-config -->\nplaceholder\n<!-- END GENERATED:pre-commit-config -->\n",
    );
    write(
        &quickstart_path,
        "# Quickstart\n\n<!-- BEGIN GENERATED:pre-commit-config -->\nplaceholder\n<!-- END GENERATED:pre-commit-config -->\n",
    );

    tq_docsgen::generate_pre_commit_docs(temp.path()).expect("generate pre-commit docs");

    let readme = fs::read_to_string(&readme_path).expect("read README");
    let quickstart = fs::read_to_string(&quickstart_path).expect("read quickstart");

    assert!(readme.contains("rev: v0.9.0"));
    assert!(quickstart.contains("rev: v0.9.0"));
    assert!(!readme.contains("placeholder"));
    assert!(!quickstart.contains("placeholder"));
}

#[test]
fn generate_pre_commit_docs_fails_when_markers_are_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cargo_toml_path = temp.path().join("Cargo.toml");
    let pyproject_toml_path = temp.path().join("pyproject.toml");
    let readme_path = temp.path().join("README.md");
    let quickstart_path = temp.path().join("docs/guide/quickstart.md");

    write(
        &cargo_toml_path,
        "[workspace]\n\n[workspace.package]\nversion = \"0.9.0\"\n",
    );
    write(
        &pyproject_toml_path,
        "[tool.commitizen]\ntag_format = \"$version\"\n",
    );
    write(&readme_path, "# README\nNo markers here.\n");
    write(
        &quickstart_path,
        "# Quickstart\n\n<!-- BEGIN GENERATED:pre-commit-config -->\nplaceholder\n<!-- END GENERATED:pre-commit-config -->\n",
    );

    let error =
        tq_docsgen::generate_pre_commit_docs(temp.path()).expect_err("missing markers should fail");
    assert!(error.to_string().contains("missing or invalid markers"));
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent directory")).expect("create directories");
    fs::write(path, contents).expect("write file");
}
