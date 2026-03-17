use std::fs;
use std::path::Path;

#[test]
fn generate_config_examples_updates_marked_sections() {
    let temp = tempfile::tempdir().expect("tempdir");
    let quickstart_path = temp.path().join("docs/guide/quickstart.md");
    let configuration_path = temp.path().join("docs/reference/configuration.md");
    let manifest_path = temp
        .path()
        .join("docs/reference/config/examples-manifest.yaml");

    write(
        &quickstart_path,
        "# QuickStart\n\n<!-- BEGIN GENERATED:quickstart-minimal-config -->\nplaceholder\n<!-- END GENERATED:quickstart-minimal-config -->\n",
    );
    write(
        &configuration_path,
        "# Configuration\n\n<!-- BEGIN GENERATED:configuration-minimal-config -->\nplaceholder\n<!-- END GENERATED:configuration-minimal-config -->\n\n<!-- BEGIN GENERATED:configuration-typical-config -->\nplaceholder\n<!-- END GENERATED:configuration-typical-config -->\n",
    );
    write(
        &manifest_path,
        "examples:\n  quickstart_minimal: |\n    [tool.tq]\n    [[tool.tq.targets]]\n    name = \"app\"\n    package = \"your_package\"\n    source_root = \"src\"\n    test_root = \"tests\"\n  configuration_minimal: |\n    [tool.tq]\n    [[tool.tq.targets]]\n    name = \"app\"\n    package = \"your_package\"\n    source_root = \"src\"\n    test_root = \"tests\"\n  configuration_typical: |\n    [tool.tq]\n    init_modules = \"ignore\"\n    [[tool.tq.targets]]\n    name = \"app\"\n    package = \"your_package\"\n    source_root = \"src\"\n    test_root = \"tests\"\n",
    );

    tq_docsgen::generate_config_examples(temp.path()).expect("generate config examples");

    let quickstart = fs::read_to_string(&quickstart_path).expect("read quickstart");
    let configuration = fs::read_to_string(&configuration_path).expect("read configuration");
    assert!(quickstart.contains("```toml"));
    assert!(quickstart.contains("name = \"app\""));
    assert!(!quickstart.contains("placeholder"));
    assert!(configuration.contains("init_modules = \"ignore\""));
    assert!(!configuration.contains("placeholder"));
}

#[test]
fn generate_config_examples_fails_for_invalid_manifest_shape() {
    let temp = tempfile::tempdir().expect("tempdir");
    let quickstart_path = temp.path().join("docs/guide/quickstart.md");
    let configuration_path = temp.path().join("docs/reference/configuration.md");
    let manifest_path = temp
        .path()
        .join("docs/reference/config/examples-manifest.yaml");

    write(
        &quickstart_path,
        "# QuickStart\n\n<!-- BEGIN GENERATED:quickstart-minimal-config -->\nplaceholder\n<!-- END GENERATED:quickstart-minimal-config -->\n",
    );
    write(
        &configuration_path,
        "# Configuration\n\n<!-- BEGIN GENERATED:configuration-minimal-config -->\nplaceholder\n<!-- END GENERATED:configuration-minimal-config -->\n\n<!-- BEGIN GENERATED:configuration-typical-config -->\nplaceholder\n<!-- END GENERATED:configuration-typical-config -->\n",
    );
    write(&manifest_path, "examples: invalid\n");

    let error =
        tq_docsgen::generate_config_examples(temp.path()).expect_err("manifest should fail");
    assert!(error.to_string().contains("failed to parse YAML file"));
}

#[test]
fn generate_config_examples_fails_when_markers_are_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let quickstart_path = temp.path().join("docs/guide/quickstart.md");
    let configuration_path = temp.path().join("docs/reference/configuration.md");
    let manifest_path = temp
        .path()
        .join("docs/reference/config/examples-manifest.yaml");

    write(&quickstart_path, "# QuickStart\nNo markers here.\n");
    write(
        &configuration_path,
        "# Configuration\n\n<!-- BEGIN GENERATED:configuration-minimal-config -->\nplaceholder\n<!-- END GENERATED:configuration-minimal-config -->\n\n<!-- BEGIN GENERATED:configuration-typical-config -->\nplaceholder\n<!-- END GENERATED:configuration-typical-config -->\n",
    );
    write(
        &manifest_path,
        "examples:\n  quickstart_minimal: \"x\"\n  configuration_minimal: \"x\"\n  configuration_typical: \"x\"\n",
    );

    let error =
        tq_docsgen::generate_config_examples(temp.path()).expect_err("missing markers should fail");
    assert!(error.to_string().contains("missing or invalid markers"));
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent directory")).expect("create directories");
    fs::write(path, contents).expect("write file");
}
