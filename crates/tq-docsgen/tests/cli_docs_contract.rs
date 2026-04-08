use std::fs;
use std::path::Path;

#[test]
fn generate_cli_docs_updates_marked_section_from_rust_cli_contract() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cli_doc_path = temp.path().join("docs/reference/cli.md");
    let manifest_path = temp.path().join("docs/reference/cli/options-manifest.json");

    write(
        &manifest_path,
        "{\n  \"version\": 1,\n  \"cli_options\": [\n    { \"arg_ids\": [\"config\"] },\n    { \"arg_ids\": [\"isolated\"] },\n    { \"arg_ids\": [\"target_names\"] },\n    { \"arg_ids\": [\"init_modules\"], \"config_key\": \"init_modules\" },\n    { \"arg_ids\": [\"max_test_file_non_blank_lines\"], \"config_key\": \"max_test_file_non_blank_lines\" },\n    { \"arg_ids\": [\"qualifier_strategy\"], \"config_key\": \"qualifier_strategy\" },\n    { \"arg_ids\": [\"allowed_qualifiers\"], \"config_key\": \"allowed_qualifiers\" },\n    { \"arg_ids\": [\"select_rules\"], \"config_key\": \"select\" },\n    { \"arg_ids\": [\"ignore_rules\"], \"config_key\": \"ignore\" },\n    { \"arg_ids\": [\"severity_overrides\"], \"config_key\": \"severity_overrides\" },\n    { \"arg_ids\": [\"output_format\"] },\n    { \"arg_ids\": [\"show_suggestions\"] },\n    { \"arg_ids\": [\"exit_zero\"] },\n    { \"arg_ids\": [\"fail_on\"], \"config_key\": \"fail_on\", \"default_display\": \"error\", \"description_note\": \"When omitted, the effective default is `error` unless configuration overrides it.\" }\n  ]\n}\n",
    );
    write(
        &cli_doc_path,
        "# CLI\n\n<!-- BEGIN GENERATED:check-options -->\nplaceholder\n<!-- END GENERATED:check-options -->\n",
    );

    tq_docsgen::generate_cli_docs(temp.path()).expect("generate CLI docs");

    let generated = fs::read_to_string(&cli_doc_path).expect("read generated CLI docs");
    assert!(generated.contains("| `--target` | — | `[]` | Run only listed target names. |"));
    assert!(generated.contains("| `--init-modules` | [`init_modules`](./configuration.md#init_modules-optional) | `none` | How mapping checks handle __init__.py modules. |"));
    assert!(generated.contains("| `--fail-on` | [`fail_on`](./configuration.md#fail_on-optional) | `error` | Minimum severity level that causes a nonzero exit. When omitted, the effective default is `error` unless configuration overrides it. |"));
    assert!(generated.contains("Run `tq check --help` for the runtime source of truth."));
}

#[test]
fn generate_cli_docs_fails_for_invalid_manifest_shape() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cli_doc_path = temp.path().join("docs/reference/cli.md");
    let manifest_path = temp.path().join("docs/reference/cli/options-manifest.json");

    write(
        &manifest_path,
        "{\"version\":1,\"cli_options\":\"invalid\"}\n",
    );
    write(
        &cli_doc_path,
        "# CLI\n\n<!-- BEGIN GENERATED:check-options -->\nplaceholder\n<!-- END GENERATED:check-options -->\n",
    );

    let error = tq_docsgen::generate_cli_docs(temp.path()).expect_err("manifest should fail");
    assert!(error.to_string().contains("failed to parse JSON file"));
}

#[test]
fn generate_cli_docs_fails_when_markers_are_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cli_doc_path = temp.path().join("docs/reference/cli.md");
    let manifest_path = temp.path().join("docs/reference/cli/options-manifest.json");

    write(
        &manifest_path,
        "{\n  \"version\": 1,\n  \"cli_options\": [\n    { \"arg_ids\": [\"config\"] },\n    { \"arg_ids\": [\"isolated\"] },\n    { \"arg_ids\": [\"target_names\"] },\n    { \"arg_ids\": [\"init_modules\"], \"config_key\": \"init_modules\" },\n    { \"arg_ids\": [\"max_test_file_non_blank_lines\"], \"config_key\": \"max_test_file_non_blank_lines\" },\n    { \"arg_ids\": [\"qualifier_strategy\"], \"config_key\": \"qualifier_strategy\" },\n    { \"arg_ids\": [\"allowed_qualifiers\"], \"config_key\": \"allowed_qualifiers\" },\n    { \"arg_ids\": [\"select_rules\"], \"config_key\": \"select\" },\n    { \"arg_ids\": [\"ignore_rules\"], \"config_key\": \"ignore\" },\n    { \"arg_ids\": [\"severity_overrides\"], \"config_key\": \"severity_overrides\" },\n    { \"arg_ids\": [\"output_format\"] },\n    { \"arg_ids\": [\"show_suggestions\"] },\n    { \"arg_ids\": [\"exit_zero\"] },\n    { \"arg_ids\": [\"fail_on\"], \"config_key\": \"fail_on\", \"default_display\": \"error\", \"description_note\": \"When omitted, the effective default is `error` unless configuration overrides it.\" }\n  ]\n}\n",
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
