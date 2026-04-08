use tq_core::Severity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleDocExample {
    pub source: &'static str,
    pub test: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuiltinRuleDoc {
    pub id: &'static str,
    pub title: &'static str,
    pub default_severity: Severity,
    pub added_in: &'static str,
    pub behavior_changes: &'static str,
    pub what_it_does: &'static str,
    pub why_this_matters: &'static str,
    pub trigger_conditions: &'static [&'static str],
    pub examples: &'static [RuleDocExample],
    pub how_to_address: &'static [&'static str],
    pub related_controls: &'static [&'static str],
}

pub fn builtin_rule_docs() -> &'static [BuiltinRuleDoc] {
    &BUILTIN_RULE_DOCS
}

pub fn builtin_rule_severity_vocabulary() -> &'static [&'static str] {
    &SEVERITY_VOCABULARY
}

pub(crate) const fn mapping_missing_test_doc() -> &'static BuiltinRuleDoc {
    &BUILTIN_RULE_DOCS[0]
}

pub(crate) const fn structure_mismatch_doc() -> &'static BuiltinRuleDoc {
    &BUILTIN_RULE_DOCS[1]
}

pub(crate) const fn test_file_too_large_doc() -> &'static BuiltinRuleDoc {
    &BUILTIN_RULE_DOCS[2]
}

pub(crate) const fn orphaned_test_doc() -> &'static BuiltinRuleDoc {
    &BUILTIN_RULE_DOCS[3]
}

const SEVERITY_VOCABULARY: [&str; 3] = [
    Severity::Error.as_str(),
    Severity::Warning.as_str(),
    Severity::Info.as_str(),
];

const BUILTIN_RULE_DOCS: [BuiltinRuleDoc; 4] = [
    BuiltinRuleDoc {
        id: "mapping-missing-test",
        title: "Mapping Missing Test",
        default_severity: Severity::Error,
        added_in: "pre-1.0",
        behavior_changes: "None to date.",
        what_it_does: "Ensure each discovered source module has at least one matching unit test module in the mirrored tests path.",
        why_this_matters: "Missing unit tests reduce discoverability and leave source modules without direct contract coverage.",
        trigger_conditions: &[
            "A source module is discovered.",
            "No matching unit test filename resolves for that module.",
            "__init__.py handling follows configured ignore policy.",
        ],
        examples: &[RuleDocExample {
            source: "src/app/engine/runner.py",
            test: "tests/app/engine/test_runner.py (missing)",
        }],
        how_to_address: &[
            "Add a mirrored unit test module under `tests/{package}/...`.",
            "Use `test_<module>.py` or an allowed qualified form.",
        ],
        related_controls: &[
            "--select mapping-missing-test",
            "--ignore mapping-missing-test",
            "[tool.tq].select / [tool.tq].ignore",
            "[tool.tq].init_modules",
            "[tool.tq].qualifier_strategy",
            "[tool.tq].allowed_qualifiers",
        ],
    },
    BuiltinRuleDoc {
        id: "structure-mismatch",
        title: "Structure Mismatch",
        default_severity: Severity::Warning,
        added_in: "pre-1.0",
        behavior_changes: "None to date.",
        what_it_does: "Detect unit test files that do not mirror the expected source-relative path layout. In multi-target mode, evaluation is scoped to the active target package path and excludes sibling configured target roots.",
        why_this_matters: "Structure drift makes tests harder to find, weakens navigability, and increases refactor friction.",
        trigger_conditions: &[
            "A unit test file is discovered.",
            "The file belongs to the active target package path.",
            "The file resolves to a source target but lives in a different path.",
            "Integration and e2e paths are excluded from this rule.",
        ],
        examples: &[RuleDocExample {
            source: "src/app/engine/runner.py",
            test: "tests/app/test_runner.py",
        }],
        how_to_address: &[
            "Move the unit test to mirror source structure.",
            "Keep the filename aligned with the targeted module.",
        ],
        related_controls: &[
            "--target",
            "--select structure-mismatch",
            "--ignore structure-mismatch",
            "[tool.tq].targets",
            "[tool.tq].select / [tool.tq].ignore",
        ],
    },
    BuiltinRuleDoc {
        id: "test-file-too-large",
        title: "Test File Too Large",
        default_severity: Severity::Warning,
        added_in: "pre-1.0",
        behavior_changes: "None to date.",
        what_it_does: "Flag test files that exceed the configured non-blank, non-comment-only line budget.",
        why_this_matters: "Oversized test modules tend to become monolithic and less actionable when failures occur.",
        trigger_conditions: &[
            "A test file is discovered.",
            "Non-blank, non-comment line count exceeds configured threshold.",
        ],
        examples: &[RuleDocExample {
            source: "n/a",
            test: "tests/app/engine/test_runner.py (over configured line limit)",
        }],
        how_to_address: &[
            "Split the suite by concern using stable qualifiers.",
            "Move shared setup into nearby `conftest.py` fixtures.",
        ],
        related_controls: &[
            "--select test-file-too-large",
            "--ignore test-file-too-large",
            "[tool.tq].select / [tool.tq].ignore",
            "[tool.tq].max_test_file_non_blank_lines",
        ],
    },
    BuiltinRuleDoc {
        id: "orphaned-test",
        title: "Orphaned Test",
        default_severity: Severity::Warning,
        added_in: "pre-1.0",
        behavior_changes: "None to date.",
        what_it_does: "Identify unit test files that do not map to an existing source module.",
        why_this_matters: "Orphaned tests often encode stale behavior and add noise in maintenance and review.",
        trigger_conditions: &[
            "A unit test file is discovered.",
            "No source module resolves for the test's target module name.",
            "Integration and e2e paths are excluded from this rule.",
        ],
        examples: &[RuleDocExample {
            source: "src/app/engine/old_runner.py (missing)",
            test: "tests/app/engine/test_old_runner.py",
        }],
        how_to_address: &[
            "Remove obsolete tests that no longer represent active source modules.",
            "Restore or create the intended source module when the test is valid.",
            "Move workflow-level coverage to integration/e2e tests as needed.",
        ],
        related_controls: &[
            "--select orphaned-test",
            "--ignore orphaned-test",
            "[tool.tq].select / [tool.tq].ignore",
            "[tool.tq].qualifier_strategy",
            "[tool.tq].allowed_qualifiers",
        ],
    },
];
