use std::fs;

use tq_rules::{
    BuiltinRuleDoc, RuleDocExample, builtin_rule_docs, builtin_rule_severity_vocabulary,
};

#[test]
fn generate_rules_docs_writes_index_pages_and_sidebar() {
    let temp = tempfile::tempdir().expect("tempdir");

    tq_docsgen::generate_rules_docs(temp.path()).expect("generate rules docs");

    let index_content =
        fs::read_to_string(temp.path().join("docs/reference/rules/index.md")).expect("read index");
    let sidebar_content = fs::read_to_string(
        temp.path()
            .join("docs/.vitepress/generated/rules-sidebar.ts"),
    )
    .expect("read sidebar");

    assert!(index_content.contains("# Rules"));
    for severity in builtin_rule_severity_vocabulary() {
        assert!(index_content.contains(&format!("- `{severity}`")));
    }

    for entry in builtin_rule_docs() {
        assert_index_contains_entry(&index_content, entry);
        assert_rule_page_matches_contract(
            &fs::read_to_string(
                temp.path()
                    .join("docs/reference/rules")
                    .join(format!("{}.md", entry.id)),
            )
            .expect("read rule page"),
            entry,
        );
        assert_sidebar_contains_entry(&sidebar_content, entry);
    }

    assert!(index_content.contains("[governance policy](../../developer/governance.md)"));
    assert!(sidebar_content.contains("export const rulesSidebarItems = ["));
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

fn assert_index_contains_entry(index_content: &str, entry: &BuiltinRuleDoc) {
    assert!(index_content.contains(&format!(
        "[`{}`](./{}.md) (`{}`; default severity: `{}`)",
        entry.title,
        entry.id,
        entry.id,
        entry.default_severity.as_str()
    )));
}

fn assert_rule_page_matches_contract(page_content: &str, entry: &BuiltinRuleDoc) {
    let BuiltinRuleDoc {
        id,
        title,
        default_severity,
        added_in,
        behavior_changes,
        what_it_does,
        why_this_matters,
        trigger_conditions,
        examples,
        how_to_address,
        related_controls,
    } = entry;

    assert!(page_content.contains(&format!("# {title}")));
    assert!(page_content.contains(&format!("Rule ID: `{id}`")));
    assert!(page_content.contains("## What it does"));
    assert!(page_content.contains(what_it_does));
    assert!(page_content.contains("## Why this matters"));
    assert!(page_content.contains(why_this_matters));
    assert!(page_content.contains("## Default severity"));
    assert!(page_content.contains(&format!("`{}`", default_severity.as_str())));
    assert!(page_content.contains("## Trigger conditions"));
    assert!(page_content.contains("## Examples"));
    assert!(page_content.contains("## How to address"));
    assert!(page_content.contains("## Related configuration and suppression controls"));
    assert!(page_content.contains("## Added in"));
    assert!(page_content.contains(&format!("`{added_in}`")));
    assert!(page_content.contains("## Behavior changes"));
    assert!(page_content.contains(behavior_changes));

    for condition in *trigger_conditions {
        assert!(page_content.contains(&format!("- {condition}")));
    }

    for example in *examples {
        assert_example_is_rendered(page_content, *example);
    }

    for item in *how_to_address {
        assert!(page_content.contains(&format!("- {item}")));
    }

    for control in *related_controls {
        assert!(page_content.contains(&format!("- `{control}`")));
    }
}

fn assert_example_is_rendered(page_content: &str, example: RuleDocExample) {
    let RuleDocExample { source, test } = example;
    assert!(page_content.contains(&format!("- Source module: `{source}`")));
    assert!(page_content.contains(&format!("- Test module: `{test}`")));
}

fn assert_sidebar_contains_entry(sidebar_content: &str, entry: &BuiltinRuleDoc) {
    assert!(sidebar_content.contains(&format!("text: \"{}\"", entry.title)));
    assert!(sidebar_content.contains(&format!("link: \"/reference/rules/{}\"", entry.id)));
}
