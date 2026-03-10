use std::fs;
use std::path::Path;

#[test]
fn generate_cli_docs_updates_marked_section_from_rust_cli_contract() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cli_doc_path = temp.path().join("docs/reference/cli.md");
    let manifest_path = temp.path().join("docs/reference/cli/options-manifest.yaml");

    write(
        &manifest_path,
        "version: 1\ncli_options:\n  - arg_ids:\n      - config\n    config_key: null\n  - arg_ids:\n      - isolated\n    config_key: null\n  - arg_ids:\n      - target_names\n    config_key: null\n  - arg_ids:\n      - max_test_file_non_blank_lines\n    config_key: max_test_file_non_blank_lines\n  - arg_ids:\n      - qualifier_strategy\n    config_key: qualifier_strategy\n  - arg_ids:\n      - allowed_qualifiers\n    config_key: allowed_qualifiers\n  - arg_ids:\n      - ignore_init_modules\n      - no_ignore_init_modules\n    config_key: ignore_init_modules\n  - arg_ids:\n      - select_rules\n    config_key: select\n  - arg_ids:\n      - ignore_rules\n    config_key: ignore\n  - arg_ids:\n      - exit_zero\n    config_key: null\n  - arg_ids:\n      - show_suggestions\n    config_key: null\n  - arg_ids:\n      - output_format\n    config_key: null\n",
    );
    write(
        &cli_doc_path,
        "# CLI\n\n<!-- BEGIN GENERATED:check-options -->\nplaceholder\n<!-- END GENERATED:check-options -->\n",
    );

    tq_docsgen::generate_cli_docs(temp.path()).expect("generate CLI docs");

    let generated = fs::read_to_string(&cli_doc_path).expect("read generated CLI docs");
    assert!(generated.contains("| `--target` | — | `[]` | Run only listed target names. |"));
    assert!(generated.contains("--ignore-init-modules, --no-ignore-init-modules"));
    assert!(generated.contains("Run `tq check --help` for the runtime source of truth."));
}

#[test]
fn generate_cli_docs_fails_for_invalid_manifest_shape() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cli_doc_path = temp.path().join("docs/reference/cli.md");
    let manifest_path = temp.path().join("docs/reference/cli/options-manifest.yaml");

    write(&manifest_path, "version: 1\ncli_options: invalid\n");
    write(
        &cli_doc_path,
        "# CLI\n\n<!-- BEGIN GENERATED:check-options -->\nplaceholder\n<!-- END GENERATED:check-options -->\n",
    );

    let error = tq_docsgen::generate_cli_docs(temp.path()).expect_err("manifest should fail");
    assert!(error.to_string().contains("failed to parse YAML file"));
}

#[test]
fn generate_cli_docs_fails_when_markers_are_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cli_doc_path = temp.path().join("docs/reference/cli.md");
    let manifest_path = temp.path().join("docs/reference/cli/options-manifest.yaml");

    write(
        &manifest_path,
        "version: 1\ncli_options:\n  - arg_ids:\n      - config\n    config_key: null\n  - arg_ids:\n      - isolated\n    config_key: null\n  - arg_ids:\n      - target_names\n    config_key: null\n  - arg_ids:\n      - max_test_file_non_blank_lines\n    config_key: max_test_file_non_blank_lines\n  - arg_ids:\n      - qualifier_strategy\n    config_key: qualifier_strategy\n  - arg_ids:\n      - allowed_qualifiers\n    config_key: allowed_qualifiers\n  - arg_ids:\n      - ignore_init_modules\n      - no_ignore_init_modules\n    config_key: ignore_init_modules\n  - arg_ids:\n      - select_rules\n    config_key: select\n  - arg_ids:\n      - ignore_rules\n    config_key: ignore\n  - arg_ids:\n      - exit_zero\n    config_key: null\n  - arg_ids:\n      - show_suggestions\n    config_key: null\n  - arg_ids:\n      - output_format\n    config_key: null\n",
    );
    write(&cli_doc_path, "# CLI\nNo markers here.\n");

    let error =
        tq_docsgen::generate_cli_docs(temp.path()).expect_err("missing markers should fail");
    assert!(error.to_string().contains("missing or invalid markers"));
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent directory")).expect("create directories");
    fs::write(path, contents).expect("write file");
}
