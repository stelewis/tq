use std::path::Path;

use crate::DocsgenError;
const RULES_DIR: &str = "docs/reference/rules";
use tq_rules::{BuiltinRuleDoc, builtin_rule_docs, builtin_rule_severity_vocabulary};

const SIDEBAR_PATH: &str = "docs/.vitepress/generated/rules-sidebar.ts";

pub fn generate(workspace_root: &Path) -> Result<(), DocsgenError> {
    let rules_dir = workspace_root.join(RULES_DIR);
    let sidebar_path = workspace_root.join(SIDEBAR_PATH);
    let rules = builtin_rule_docs();

    std::fs::create_dir_all(&rules_dir).map_err(|source| DocsgenError::Io {
        path: rules_dir.clone(),
        source,
    })?;

    let index_path = rules_dir.join("index.md");
    std::fs::write(
        &index_path,
        render_index(rules, builtin_rule_severity_vocabulary()),
    )
    .map_err(|source| DocsgenError::Io {
        path: index_path,
        source,
    })?;

    for rule in rules {
        let page_path = rules_dir.join(format!("{}.md", rule.id));
        std::fs::write(&page_path, render_rule_page(rule)).map_err(|source| DocsgenError::Io {
            path: page_path,
            source,
        })?;
    }

    let sidebar_dir = sidebar_path.parent().ok_or_else(|| {
        DocsgenError::manifest(
            sidebar_path.clone(),
            "generated sidebar path must have a parent directory",
        )
    })?;
    std::fs::create_dir_all(sidebar_dir).map_err(|source| DocsgenError::Io {
        path: sidebar_dir.to_path_buf(),
        source,
    })?;
    std::fs::write(&sidebar_path, render_rules_sidebar_items(rules)).map_err(|source| {
        DocsgenError::Io {
            path: sidebar_path,
            source,
        }
    })
}

fn render_index(entries: &[BuiltinRuleDoc], severity_vocabulary: &[&str]) -> String {
    let mut lines = vec![
        "# Rules".to_owned(),
        String::new(),
        "User-facing rules and default severities.".to_owned(),
        String::new(),
        "## Stable rule IDs".to_owned(),
        String::new(),
    ];

    lines.extend(entries.iter().map(|entry| {
        format!(
            "- [`{}`](./{}.md) (`{}`; default severity: `{}`)",
            entry.title,
            entry.id,
            entry.id,
            entry.default_severity.as_str()
        )
    }));

    lines.push(String::new());
    lines.push("## Severity vocabulary".to_owned());
    lines.push(String::new());
    lines.extend(
        severity_vocabulary
            .iter()
            .map(|severity| format!("- `{severity}`")),
    );

    lines.extend([
        String::new(),
        "## Rule policy".to_owned(),
        String::new(),
        "- Rule IDs are stable kebab-case identifiers.".to_owned(),
        "- Severity defaults are part of the external contract.".to_owned(),
        "- Rule selection and suppression use `--select`/`--ignore` and `[tool.tq]` values."
            .to_owned(),
        "- Rule additions and severity default changes follow [governance policy](../../developer/governance.md)."
            .to_owned(),
        String::new(),
    ]);

    lines.join("\n")
}

fn render_rule_page(entry: &BuiltinRuleDoc) -> String {
    let mut lines = vec![
        format!("# {}", entry.title),
        String::new(),
        format!("Rule ID: `{}`", entry.id),
        String::new(),
        "## What it does".to_owned(),
        String::new(),
        entry.what_it_does.to_owned(),
        String::new(),
        "## Why this matters".to_owned(),
        String::new(),
        entry.why_this_matters.to_owned(),
        String::new(),
        "## Default severity".to_owned(),
        String::new(),
        format!("`{}`", entry.default_severity.as_str()),
        String::new(),
        "## Trigger conditions".to_owned(),
        String::new(),
    ];
    lines.extend(
        entry
            .trigger_conditions
            .iter()
            .map(|condition| format!("- {condition}")),
    );

    lines.extend([String::new(), "## Examples".to_owned(), String::new()]);
    for example in entry.examples {
        lines.push(format!("- Source module: `{}`", example.source));
        lines.push(format!("- Test module: `{}`", example.test));
    }

    lines.extend([String::new(), "## How to address".to_owned(), String::new()]);
    lines.extend(entry.how_to_address.iter().map(|item| format!("- {item}")));

    lines.extend([
        String::new(),
        "## Related configuration and suppression controls".to_owned(),
        String::new(),
    ]);
    lines.extend(
        entry
            .related_controls
            .iter()
            .map(|item| format!("- `{item}`")),
    );

    lines.extend([
        String::new(),
        "## Added in".to_owned(),
        String::new(),
        format!("`{}`", entry.added_in),
        String::new(),
        "## Behavior changes".to_owned(),
        String::new(),
        entry.behavior_changes.to_owned(),
        String::new(),
    ]);

    lines.join("\n")
}

fn render_rules_sidebar_items(entries: &[BuiltinRuleDoc]) -> String {
    let mut lines = vec!["export const rulesSidebarItems = [".to_owned()];

    for entry in entries {
        lines.extend([
            "  {".to_owned(),
            format!("    text: \"{}\",", entry.title),
            format!("    link: \"/reference/rules/{}\"", entry.id),
            "  },".to_owned(),
        ]);
    }

    lines.extend(["] as const;".to_owned(), String::new()]);
    lines.join("\n")
}
