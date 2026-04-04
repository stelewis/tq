use std::path::Path;

use serde::Deserialize;

use crate::DocsgenError;

const MANIFEST_PATH: &str = "docs/reference/rules/manifest.json";
const RULES_DIR: &str = "docs/reference/rules";
const SIDEBAR_PATH: &str = "docs/.vitepress/generated/rules-sidebar.ts";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesManifest {
    version: u64,
    severity_vocabulary: Vec<String>,
    rules: Vec<RuleManifestEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleManifestEntry {
    id: String,
    title: String,
    default_severity: String,
    added_in: String,
    behavior_changes: String,
    what_it_does: String,
    why_this_matters: String,
    trigger_conditions: Vec<String>,
    examples: Vec<RuleExample>,
    how_to_address: Vec<String>,
    related_controls: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleExample {
    source: String,
    test: String,
}

pub fn generate(workspace_root: &Path) -> Result<(), DocsgenError> {
    let manifest_path = workspace_root.join(MANIFEST_PATH);
    let rules_dir = workspace_root.join(RULES_DIR);
    let sidebar_path = workspace_root.join(SIDEBAR_PATH);
    let manifest = load_manifest(&manifest_path)?;

    let index_path = rules_dir.join("index.md");
    std::fs::write(
        &index_path,
        render_index(&manifest.rules, &manifest.severity_vocabulary),
    )
    .map_err(|source| DocsgenError::Io {
        path: index_path,
        source,
    })?;

    for rule in &manifest.rules {
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
    std::fs::write(&sidebar_path, render_rules_sidebar_items(&manifest.rules)).map_err(|source| {
        DocsgenError::Io {
            path: sidebar_path,
            source,
        }
    })
}

fn load_manifest(path: &Path) -> Result<RulesManifest, DocsgenError> {
    let content = std::fs::read_to_string(path).map_err(|source| DocsgenError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let manifest: RulesManifest =
        serde_json::from_str(&content).map_err(|source| DocsgenError::Json {
            path: path.to_path_buf(),
            source,
        })?;

    if manifest.version != 1 {
        return Err(DocsgenError::manifest(
            path.to_path_buf(),
            format!("unsupported manifest version: {}", manifest.version),
        ));
    }

    if manifest.severity_vocabulary.is_empty() {
        return Err(DocsgenError::manifest(
            path.to_path_buf(),
            "severity_vocabulary must not be empty",
        ));
    }
    if manifest.rules.is_empty() {
        return Err(DocsgenError::manifest(
            path.to_path_buf(),
            "rules must contain at least one entry",
        ));
    }

    for severity in &manifest.severity_vocabulary {
        validate_non_empty(path, "severity_vocabulary entry", severity)?;
    }

    for rule in &manifest.rules {
        validate_rule(path, rule)?;
    }

    Ok(manifest)
}

fn validate_rule(path: &Path, rule: &RuleManifestEntry) -> Result<(), DocsgenError> {
    validate_non_empty(path, "rules.id", &rule.id)?;
    validate_non_empty(path, "rules.title", &rule.title)?;
    validate_non_empty(path, "rules.default_severity", &rule.default_severity)?;
    validate_non_empty(path, "rules.added_in", &rule.added_in)?;
    validate_non_empty(path, "rules.behavior_changes", &rule.behavior_changes)?;
    validate_non_empty(path, "rules.what_it_does", &rule.what_it_does)?;
    validate_non_empty(path, "rules.why_this_matters", &rule.why_this_matters)?;
    validate_list(path, "rules.trigger_conditions", &rule.trigger_conditions)?;
    validate_list(path, "rules.how_to_address", &rule.how_to_address)?;
    validate_list(path, "rules.related_controls", &rule.related_controls)?;

    if rule.examples.is_empty() {
        return Err(DocsgenError::manifest(
            path.to_path_buf(),
            format!("rule `{}` must define at least one example", rule.id),
        ));
    }

    for example in &rule.examples {
        validate_non_empty(path, "rules.examples.source", &example.source)?;
        validate_non_empty(path, "rules.examples.test", &example.test)?;
    }

    Ok(())
}

fn validate_list(path: &Path, field_name: &str, values: &[String]) -> Result<(), DocsgenError> {
    if values.is_empty() {
        return Err(DocsgenError::manifest(
            path.to_path_buf(),
            format!("{field_name} must contain at least one entry"),
        ));
    }

    for value in values {
        validate_non_empty(path, field_name, value)?;
    }

    Ok(())
}

fn validate_non_empty(path: &Path, field_name: &str, value: &str) -> Result<(), DocsgenError> {
    if value.trim().is_empty() {
        return Err(DocsgenError::manifest(
            path.to_path_buf(),
            format!("{field_name} must be a non-empty string"),
        ));
    }

    Ok(())
}

fn render_index(entries: &[RuleManifestEntry], severity_vocabulary: &[String]) -> String {
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
            entry.title, entry.id, entry.id, entry.default_severity
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

fn render_rule_page(entry: &RuleManifestEntry) -> String {
    let mut lines = vec![
        format!("# {}", entry.title),
        String::new(),
        format!("Rule ID: `{}`", entry.id),
        String::new(),
        "## What it does".to_owned(),
        String::new(),
        entry.what_it_does.clone(),
        String::new(),
        "## Why this matters".to_owned(),
        String::new(),
        entry.why_this_matters.clone(),
        String::new(),
        "## Default severity".to_owned(),
        String::new(),
        format!("`{}`", entry.default_severity),
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
    for example in &entry.examples {
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
        entry.behavior_changes.clone(),
        String::new(),
    ]);

    lines.join("\n")
}

fn render_rules_sidebar_items(entries: &[RuleManifestEntry]) -> String {
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
