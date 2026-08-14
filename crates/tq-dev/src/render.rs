//! Report rendering: each report maps to one [`Document`], which renders to
//! plain text, agent Markdown, or JSON.

use serde::Serialize;

use crate::action::ActionPlan;
use crate::audit::{AuditReport, AuditStatus};
use crate::check::{CheckPlan, CheckReport, CheckStatus};
use crate::doctor::{DoctorCheck, DoctorReport, ToolRequirement, ToolState};
use crate::error::DevError;
use crate::external_pins::{ExternalPin, ExternalPinReport, ExternalPinStatus};

/// A rendered report: a title, summary lines, a tabular or step-list body,
/// and detail sections for items that need attention.
pub struct Document {
    title: String,
    summary: Vec<String>,
    body: Body,
    sections: Vec<Section>,
}

enum Body {
    Table(Table),
    Steps(Vec<Step>),
}

struct Table {
    headers: Vec<&'static str>,
    rows: Vec<Vec<String>>,
}

/// An ordered plan step: a label plus the exact action it performs.
struct Step {
    label: String,
    detail: String,
}

/// A detail block attached to a report item.
struct Section {
    heading: String,
    body: String,
    kind: SectionKind,
}

enum SectionKind {
    /// Guidance prose, rendered as a paragraph in Markdown.
    Prose,
    /// Preformatted command output, rendered in a code fence in Markdown.
    Output,
}

/// Output format for harness reports.
#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub enum OutputMode {
    /// Aligned plain-text output for terminals.
    Human,
    /// Pretty-printed JSON for automation.
    Json,
    /// Markdown output for agents and step summaries.
    Agent,
}

impl Document {
    pub fn print<T: Serialize>(&self, mode: OutputMode, report: &T) -> Result<(), DevError> {
        match mode {
            OutputMode::Human => print!("{}", self.to_text()),
            OutputMode::Json => println!("{}", to_json(report)?),
            OutputMode::Agent => print!("{}", self.to_markdown()),
        }
        Ok(())
    }

    fn to_text(&self) -> String {
        let mut lines = vec![self.title.clone()];
        lines.extend(self.summary.iter().cloned());
        lines.push(String::new());
        lines.push(match &self.body {
            Body::Table(table) => table.to_text(),
            Body::Steps(steps) => steps_to_text(steps),
        });
        for section in &self.sections {
            lines.push(String::new());
            lines.push(format!("{}:", section.heading));
            lines.extend(section.body.lines().map(|line| format!("  {line}")));
        }
        lines.push(String::new());
        lines.join("\n")
    }

    fn to_markdown(&self) -> String {
        let mut lines = vec![format!("## {}", self.title), String::new()];
        lines.extend(self.summary.iter().map(|line| format!("- {line}")));
        lines.push(String::new());
        lines.push(match &self.body {
            Body::Table(table) => table.to_markdown(),
            Body::Steps(steps) => steps_to_markdown(steps),
        });
        for section in &self.sections {
            lines.push(String::new());
            lines.push(format!("### {}", section.heading));
            lines.push(String::new());
            match section.kind {
                SectionKind::Prose => lines.push(section.body.clone()),
                SectionKind::Output => {
                    lines.push("```text".to_owned());
                    lines.push(section.body.clone());
                    lines.push("```".to_owned());
                }
            }
        }
        lines.push(String::new());
        lines.join("\n")
    }
}

fn to_json<T: Serialize>(report: &T) -> Result<String, DevError> {
    serde_json::to_string_pretty(report).map_err(|source| DevError::ReportSerialization { source })
}

impl Table {
    fn to_text(&self) -> String {
        let widths = self.column_widths();
        let mut lines = vec![text_row(
            &self
                .headers
                .iter()
                .map(|header| (*header).to_owned())
                .collect::<Vec<_>>(),
            &widths,
        )];
        lines.extend(self.rows.iter().map(|row| text_row(row, &widths)));
        lines.join("\n")
    }

    fn to_markdown(&self) -> String {
        let mut lines = vec![
            format!(
                "| {} |",
                self.headers
                    .iter()
                    .map(|header| markdown_cell(header))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
            format!("| {} |", vec!["---"; self.headers.len()].join(" | ")),
        ];
        lines.extend(self.rows.iter().map(|row| {
            format!(
                "| {} |",
                row.iter()
                    .map(|column| markdown_cell(column))
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
        }));
        lines.join("\n")
    }

    fn column_widths(&self) -> Vec<usize> {
        let mut widths = self
            .headers
            .iter()
            .map(|header| header.len())
            .collect::<Vec<_>>();
        for row in &self.rows {
            for (index, column) in row.iter().enumerate() {
                widths[index] = widths[index].max(column.len());
            }
        }
        widths
    }
}

fn text_row(columns: &[String], widths: &[usize]) -> String {
    columns
        .iter()
        .enumerate()
        .map(|(index, column)| format!("{column:<width$}", width = widths[index]))
        .collect::<Vec<_>>()
        .join("  ")
        .trim_end()
        .to_owned()
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}

fn steps_to_text(steps: &[Step]) -> String {
    steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let number = index + 1;
            let indent = " ".repeat(number.to_string().len() + 2);
            format!("{number}. {}\n{indent}{}", step.label, step.detail)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn steps_to_markdown(steps: &[Step]) -> String {
    steps
        .iter()
        .enumerate()
        .map(|(index, step)| format!("{}. {}: `{}`", index + 1, step.label, step.detail))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Groups remediation guidance by identical text so shared advice renders
/// once, preserving first-occurrence order.
fn remediation_sections<'a>(items: impl Iterator<Item = (&'a str, &'a str)>) -> Vec<Section> {
    let mut groups: Vec<(&str, Vec<&str>)> = Vec::new();
    for (name, remediation) in items {
        if let Some((_, names)) = groups.iter_mut().find(|(body, _)| *body == remediation) {
            names.push(name);
        } else {
            groups.push((remediation, vec![name]));
        }
    }
    groups
        .into_iter()
        .map(|(body, names)| Section {
            heading: format!("{} remediation", names.join(", ")),
            body: body.to_owned(),
            kind: SectionKind::Prose,
        })
        .collect()
}

fn command_output_section(name: &str, command: &str, output: &str) -> Section {
    Section {
        heading: format!("{name} output"),
        body: format!("$ {command}\n{output}"),
        kind: SectionKind::Output,
    }
}

#[must_use]
pub fn doctor_document(report: &DoctorReport) -> Document {
    Document {
        title: "Developer Environment".to_owned(),
        summary: vec![
            format!("Status: {}", report.summary.status),
            format!(
                "Checks: {} total, {} ok, {} unhealthy",
                report.summary.total, report.summary.ok, report.summary.unhealthy
            ),
        ],
        body: Body::Table(Table {
            headers: vec!["Status", "Tool", "Required", "Installed"],
            rows: report
                .checks
                .iter()
                .map(|check| {
                    vec![
                        check.state.label().to_owned(),
                        check.tool.clone(),
                        required_cell(&check.required),
                        installed_cell(check),
                    ]
                })
                .collect(),
        }),
        sections: remediation_sections(report.checks.iter().filter_map(|check| {
            check
                .remediation
                .as_deref()
                .map(|remediation| (check.tool.as_str(), remediation))
        })),
    }
}

fn required_cell(required: &ToolRequirement) -> String {
    match required {
        ToolRequirement::Version { version } => version.to_string(),
        ToolRequirement::Presence => "installed".to_owned(),
    }
}

/// A met requirement reports itself back, so an `ok` row shows the version or
/// presence the check confirmed rather than an empty cell.
fn installed_cell(check: &DoctorCheck) -> String {
    match &check.state {
        ToolState::Ok => required_cell(&check.required),
        ToolState::Mismatched { installed } => installed.to_string(),
        ToolState::Unreadable => "unknown".to_owned(),
        ToolState::Missing => "not found".to_owned(),
    }
}

#[must_use]
pub fn audit_document(title: &str, report: &AuditReport) -> Document {
    Document {
        title: title.to_owned(),
        summary: vec![
            format!("Status: {}", report.summary.status),
            format!(
                "Checks: {} total, {} clean, {} findings, {} failed",
                report.summary.total,
                report.summary.clean,
                report.summary.findings,
                report.summary.failed
            ),
        ],
        body: Body::Table(audit_table(report)),
        sections: {
            let mut sections = report
                .checks
                .iter()
                .filter(|check| check.status != AuditStatus::Clean && !check.output.is_empty())
                .map(|check| command_output_section(&check.name, &check.command, &check.output))
                .collect::<Vec<_>>();
            sections.extend(remediation_sections(report.checks.iter().filter_map(
                |check| {
                    check
                        .remediation
                        .as_deref()
                        .map(|remediation| (check.name.as_str(), remediation))
                },
            )));
            sections
        },
    }
}

/// The audit table: pin columns appear only when at least one check
/// compares a pin against upstream.
fn audit_table(report: &AuditReport) -> Table {
    let has_pins = report.checks.iter().any(|check| check.pinned.is_some());
    if has_pins {
        Table {
            headers: vec!["Status", "Name", "Pinned", "Latest"],
            rows: report
                .checks
                .iter()
                .map(|check| {
                    vec![
                        check.status.to_string(),
                        check.name.clone(),
                        check.pinned.clone().unwrap_or_else(|| "-".to_owned()),
                        check.latest.clone().unwrap_or_else(|| "-".to_owned()),
                    ]
                })
                .collect(),
        }
    } else {
        Table {
            headers: vec!["Status", "Name"],
            rows: report
                .checks
                .iter()
                .map(|check| vec![check.status.to_string(), check.name.clone()])
                .collect(),
        }
    }
}

#[must_use]
pub fn check_plan_document(plan: &CheckPlan) -> Document {
    Document {
        title: "Developer Check Plan".to_owned(),
        summary: vec![
            format!("Target: {}", plan.target),
            format!("Profile: {}", plan.profile),
            format!("Tasks: {} planned", plan.tasks.len()),
        ],
        body: Body::Steps(
            plan.tasks
                .iter()
                .map(|task| Step {
                    label: task.title().to_owned(),
                    detail: task.invocation().display(),
                })
                .collect(),
        ),
        sections: Vec::new(),
    }
}

#[must_use]
pub fn check_report_document(report: &CheckReport) -> Document {
    Document {
        title: "Developer Checks".to_owned(),
        summary: vec![
            format!("Status: {}", report.summary.status),
            format!(
                "Checks: {} total, {} passed, {} failed in {:.2}s",
                report.summary.total,
                report.summary.passed,
                report.summary.failed,
                report.summary.elapsed_seconds
            ),
        ],
        body: Body::Table(Table {
            headers: vec!["Status", "Check", "Time"],
            rows: report
                .checks
                .iter()
                .map(|check| {
                    vec![
                        check.status.to_string(),
                        check.task.title().to_owned(),
                        format!("{:.2}s", check.elapsed_seconds),
                    ]
                })
                .collect(),
        }),
        sections: report
            .checks
            .iter()
            .filter(|check| check.status == CheckStatus::Failed && !check.output.is_empty())
            .map(|check| command_output_section(check.task.title(), &check.command, &check.output))
            .collect(),
    }
}

#[must_use]
pub fn action_plan_document(plan: &ActionPlan) -> Document {
    Document {
        title: format!("{} Plan", plan.title),
        summary: vec![format!("Actions: {} planned", plan.actions.len())],
        body: Body::Steps(
            plan.actions
                .iter()
                .map(|action| Step {
                    label: action.label.clone(),
                    detail: action.action.detail(),
                })
                .collect(),
        ),
        sections: Vec::new(),
    }
}

#[must_use]
pub fn external_pin_document(report: &ExternalPinReport) -> Document {
    Document {
        title: "Frozen External Pin Review".to_owned(),
        summary: vec![
            format!(
                "Status: {}",
                if report.drift_detected {
                    "review needed"
                } else {
                    "clean"
                }
            ),
            format!("Pins: {} checked", report.pins.len()),
        ],
        body: Body::Table(Table {
            headers: vec!["Status", "Surface", "Dependency", "Pinned", "Latest"],
            rows: report
                .pins
                .iter()
                .map(|pin| {
                    vec![
                        pin.status.to_string(),
                        pin.surface.to_string(),
                        pin.name.clone(),
                        pin.pinned
                            .as_deref()
                            .map_or_else(|| "-".to_owned(), short_revision),
                        pin.latest_tag.clone().unwrap_or_else(|| "-".to_owned()),
                    ]
                })
                .collect(),
        }),
        sections: report
            .pins
            .iter()
            .filter(|pin| pin.status != ExternalPinStatus::UpToDate)
            .map(pin_detail_section)
            .collect(),
    }
}

fn pin_detail_section(pin: &ExternalPin) -> Section {
    let mut lines = Vec::new();
    if let Some(pinned) = &pin.pinned {
        lines.push(format!("pinned:  {pinned}"));
    }
    if let (Some(tag), Some(sha)) = (&pin.latest_tag, &pin.latest_sha) {
        lines.push(format!("latest:  {tag} ({sha})"));
    }
    if let Some(message) = &pin.message {
        lines.push(format!("detail:  {message}"));
    }
    lines.push(format!("sources: {}", pin.sources.join(", ")));
    Section {
        heading: pin.name.clone(),
        body: lines.join("\n"),
        kind: SectionKind::Output,
    }
}

/// Shortens a full 40-character commit SHA for tabular display; other
/// revision text is shown verbatim.
fn short_revision(revision: &str) -> String {
    if revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        revision[..12].to_owned()
    } else {
        revision.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use crate::action::{ActionPlan, PlannedAction};
    use crate::doctor::{DoctorCheck, DoctorReport, ToolRequirement, ToolState};
    use crate::invocation::Invocation;
    use crate::manifest::ToolVersion;

    use super::{action_plan_document, doctor_document, short_revision, to_json};

    fn version(text: &str) -> ToolVersion {
        ToolVersion::parse(text).expect("valid version")
    }

    fn check(tool: &str, required: ToolRequirement, state: ToolState) -> DoctorCheck {
        DoctorCheck::new(tool, required, state, "Update the Rust toolchain.")
    }

    fn pinned(tool: &str, pin: &str, state: ToolState) -> DoctorCheck {
        check(
            tool,
            ToolRequirement::Version {
                version: version(pin),
            },
            state,
        )
    }

    fn unhealthy_report() -> DoctorReport {
        DoctorReport::new(vec![
            pinned("rustc", "1.96.1", ToolState::Ok),
            pinned(
                "cargo",
                "1.96.1",
                ToolState::Mismatched {
                    installed: version("1.95.0"),
                },
            ),
            pinned(
                "cargo|deny",
                "0.19.0",
                ToolState::Mismatched {
                    installed: version("0.18.0"),
                },
            ),
        ])
    }

    #[test]
    fn text_output_leads_with_summary_and_aligned_table() {
        let output = doctor_document(&unhealthy_report()).to_text();

        assert!(output.starts_with("Developer Environment\nStatus: unhealthy\n"));
        assert!(output.contains("Checks: 3 total, 1 ok, 2 unhealthy"));
        assert!(output.contains("Status      Tool"));
    }

    #[test]
    fn shared_remediation_text_renders_once_for_all_affected_tools() {
        let output = doctor_document(&unhealthy_report()).to_text();

        assert!(output.contains("cargo, cargo|deny remediation:\n  Update the Rust toolchain."));
        assert_eq!(output.matches("Update the Rust toolchain.").count(), 1);
    }

    #[test]
    fn markdown_output_escapes_cells_and_lists_summary() {
        let output = doctor_document(&unhealthy_report()).to_markdown();

        assert!(output.starts_with("## Developer Environment"));
        assert!(output.contains("- Status: unhealthy"));
        assert!(output.contains("| mismatched | cargo\\|deny | 0.19.0 | 0.18.0 |"));
        assert!(output.contains("### cargo, cargo|deny remediation"));
        assert!(output.contains("Update the Rust toolchain."));
    }

    #[test]
    fn every_tool_state_renders_an_installed_cell() {
        let report = DoctorReport::new(vec![
            pinned("rustc", "1.96.1", ToolState::Ok),
            pinned("actionlint", "1.7.12", ToolState::Unreadable),
            pinned("uv", "0.9.7", ToolState::Missing),
            check("pkg-config", ToolRequirement::Presence, ToolState::Ok),
            check("openssl@3", ToolRequirement::Presence, ToolState::Missing),
        ]);

        let output = doctor_document(&report).to_markdown();

        assert!(output.contains("| ok | rustc | 1.96.1 | 1.96.1 |"));
        assert!(output.contains("| unreadable | actionlint | 1.7.12 | unknown |"));
        assert!(output.contains("| missing | uv | 0.9.7 | not found |"));
        assert!(output.contains("| ok | pkg-config | installed | installed |"));
        assert!(output.contains("| missing | openssl@3 | installed | not found |"));
    }

    #[test]
    fn serialized_reports_describe_requirements_and_states_by_tag() {
        let json = to_json(&unhealthy_report()).expect("doctor report should serialize");

        assert!(json.contains("\"kind\": \"version\""));
        assert!(json.contains("\"status\": \"mismatched\""));
        assert!(json.contains("\"installed\": \"1.95.0\""));
    }

    #[test]
    fn action_plans_render_as_numbered_steps_with_full_width_commands() {
        let plan = ActionPlan::new(
            "Developer setup",
            vec![PlannedAction::command(
                "Sync locked Python dependencies",
                Invocation::new("uv", ["sync", "--locked"]),
            )],
        );

        let text = action_plan_document(&plan).to_text();
        assert!(text.contains("1. Sync locked Python dependencies\n   uv sync --locked"));

        let markdown = action_plan_document(&plan).to_markdown();
        assert!(markdown.contains("1. Sync locked Python dependencies: `uv sync --locked`"));
    }

    #[test]
    fn full_commit_shas_shorten_for_display_and_other_revisions_do_not() {
        assert_eq!(
            short_revision("9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0"),
            "9c091bb21b7c"
        );
        assert_eq!(short_revision("v7.0.0"), "v7.0.0");
    }
}
