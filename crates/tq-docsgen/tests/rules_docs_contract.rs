use std::fs;
use std::path::Path;

#[test]
fn generate_rules_docs_writes_index_pages_and_sidebar() {
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_path = temp.path().join("docs/reference/rules/manifest.json");

    write(
        &manifest_path,
        "{\n  \"version\": 1,\n  \"severity_vocabulary\": [\"error\", \"warning\", \"info\"],\n  \"rules\": [\n    {\n      \"id\": \"orphaned-test\",\n      \"title\": \"Orphaned Test\",\n      \"default_severity\": \"warning\",\n      \"added_in\": \"0.4.0\",\n      \"behavior_changes\": \"none\",\n      \"what_it_does\": \"detects tests with no source module\",\n      \"why_this_matters\": \"avoids stale tests\",\n      \"trigger_conditions\": [\"no corresponding source module exists\"],\n      \"examples\": [\n        { \"source\": \"n/a\", \"test\": \"tests/tq/rules/test_obsolete.py\" }\n      ],\n      \"how_to_address\": [\"delete stale test or restore source module\"],\n      \"related_controls\": [\"--ignore\"]\n    }\n  ]\n}\n",
    );

    tq_docsgen::generate_rules_docs(temp.path()).expect("generate rules docs");

    let index_content =
        fs::read_to_string(temp.path().join("docs/reference/rules/index.md")).expect("read index");
    let page_content =
        fs::read_to_string(temp.path().join("docs/reference/rules/orphaned-test.md"))
            .expect("read page");
    let sidebar_content = fs::read_to_string(
        temp.path()
            .join("docs/.vitepress/generated/rules-sidebar.ts"),
    )
    .expect("read sidebar");

    assert!(index_content.contains("# Rules"));
    assert!(index_content.contains("[`orphaned-test`](./orphaned-test.md)"));
    assert!(index_content.contains("[governance policy](../../developer/governance.md)"));
    assert!(page_content.contains("# orphaned-test"));
    assert!(page_content.contains("## Trigger conditions"));
    assert!(sidebar_content.contains("export const rulesSidebarItems = ["));
    assert!(sidebar_content.contains("text: \"orphaned-test\""));
}

#[test]
fn generate_rules_docs_fails_for_invalid_manifest() {
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_path = temp.path().join("docs/reference/rules/manifest.json");

    write(&manifest_path, "{\"version\":1,\"rules\":\"invalid\"}\n");

    let error = tq_docsgen::generate_rules_docs(temp.path()).expect_err("manifest should fail");
    assert!(error.to_string().contains("failed to parse JSON file"));
}

#[test]
fn generate_rules_docs_fails_when_severity_vocabulary_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_path = temp.path().join("docs/reference/rules/manifest.json");

    write(
        &manifest_path,
        "{\n  \"version\": 1,\n  \"rules\": [\n    {\n      \"id\": \"orphaned-test\",\n      \"title\": \"Orphaned Test\",\n      \"default_severity\": \"warning\",\n      \"added_in\": \"0.4.0\",\n      \"behavior_changes\": \"none\",\n      \"what_it_does\": \"detects tests with no source module\",\n      \"why_this_matters\": \"avoids stale tests\",\n      \"trigger_conditions\": [\"no corresponding source module exists\"],\n      \"examples\": [\n        { \"source\": \"n/a\", \"test\": \"tests/tq/rules/test_obsolete.py\" }\n      ],\n      \"how_to_address\": [\"delete stale test or restore source module\"],\n      \"related_controls\": [\"--ignore\"]\n    }\n  ]\n}\n",
    );

    let error =
        tq_docsgen::generate_rules_docs(temp.path()).expect_err("missing severity should fail");
    assert!(
        error.to_string().contains("failed to parse JSON file")
            || error.to_string().contains("severity_vocabulary")
    );
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent directory")).expect("create directories");
    fs::write(path, contents).expect("write file");
}
