use std::fs;

#[test]
fn generate_rules_docs_writes_index_pages_and_sidebar() {
    let temp = tempfile::tempdir().expect("tempdir");

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
    assert!(index_content.contains("[`Orphaned Test`](./orphaned-test.md)"));
    assert!(index_content.contains("`orphaned-test`; default severity: `warning`"));
    assert!(index_content.contains("[governance policy](../../developer/governance.md)"));
    assert!(page_content.contains("# Orphaned Test"));
    assert!(page_content.contains("Rule ID: `orphaned-test`"));
    assert!(page_content.contains("## Trigger conditions"));
    assert!(sidebar_content.contains("export const rulesSidebarItems = ["));
    assert!(sidebar_content.contains("text: \"Orphaned Test\""));
}

#[test]
fn generate_rules_docs_creates_missing_output_directories() {
    let temp = tempfile::tempdir().expect("tempdir");
    tq_docsgen::generate_rules_docs(temp.path()).expect("generate rules docs");

    assert!(temp.path().join("docs/reference/rules/index.md").is_file());
    assert!(
        temp.path()
            .join("docs/.vitepress/generated/rules-sidebar.ts")
            .is_file()
    );
}
