//! Report rendering: each report maps to one [`Document`], which renders to
//! plain text, agent Markdown, or JSON.

use serde::Serialize;

use crate::action::ActionPlan;
use crate::audit::{AuditReport, AuditStatus};
use crate::check::{CheckPlan, CheckReport, CheckStatus};
use crate::doctor::DoctorReport;
use crate::error::DevError;
use crate::external_pins::ExternalPinReport;

/// A rendered report: a title, summary lines, a table, and optional
/// per-item detail sections.
pub struct Document {
    pub title: String,
    pub summary: Vec<String>,
    pub table: Table,
    pub sections: Vec<Section>,
}

pub struct Table {
    pub headers: Vec<&'static str>,
    pub rows: Vec<Vec<String>>,
}

/// A block of command output attached to a report item.
pub struct Section {
    pub heading: String,
    pub body: String,
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
        lines.push(self.table.to_text());
        for section in &self.sections {
            lines.push(String::new());
            lines.push(format!("{}:", section.heading));
            lines.push(section.body.clone());
        }
        lines.push(String::new());
        lines.join("\n")
    }

    fn to_markdown(&self) -> String {
        let mut lines = vec![format!("## {}", self.title), String::new()];
        lines.extend(self.summary.iter().cloned());
        lines.push(String::new());
        lines.push(self.table.to_markdown());
        for section in &self.sections {
            lines.push(String::new());
            lines.push(format!("### {}", section.heading));
            lines.push(String::new());
            lines.push("```text".to_owned());
            lines.push(section.body.clone());
            lines.push("```".to_owned());
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

#[must_use]
pub fn doctor_document(report: &DoctorReport) -> Document {
    Document {
        title: "Developer Environment".to_owned(),
        summary: vec![
            format!("Status: {}", report.summary.status),
            format!(
                "Checks: {} total, {} ok, {} missing, {} mismatched",
                report.summary.total,
                report.summary.ok,
                report.summary.missing,
                report.summary.mismatched
            ),
        ],
        table: Table {
            headers: vec!["Status", "Tool", "Expected", "Actual"],
            rows: report
                .checks
                .iter()
                .map(|check| {
                    vec![
                        check.status.to_string(),
                        check.tool.clone(),
                        check.expected.clone(),
                        check.actual.as_deref().map_or_else(
                            || "not found".to_owned(),
                            |actual| actual.replace('\n', "; "),
                        ),
                    ]
                })
                .collect(),
        },
        sections: report
            .checks
            .iter()
            .filter_map(|check| {
                check.remediation.as_ref().map(|remediation| Section {
                    heading: format!("{} remediation", check.tool),
                    body: remediation.clone(),
                })
            })
            .collect(),
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
        table: Table {
            headers: vec!["Status", "Name", "Pinned", "Latest", "Command", "Action"],
            rows: report
                .checks
                .iter()
                .map(|check| {
                    vec![
                        check.status.to_string(),
                        check.name.clone(),
                        check.pinned.clone().unwrap_or_else(|| "-".to_owned()),
                        check.latest.clone().unwrap_or_else(|| "-".to_owned()),
                        check.command.clone(),
                        check.remediation.clone().unwrap_or_else(|| "-".to_owned()),
                    ]
                })
                .collect(),
        },
        sections: report
            .checks
            .iter()
            .filter(|check| check.status != AuditStatus::Clean)
            .flat_map(|check| {
                let output = (!check.output.is_empty()).then(|| Section {
                    heading: format!("{} output", check.name),
                    body: check.output.clone(),
                });
                let remediation = check.remediation.as_ref().map(|remediation| Section {
                    heading: format!("{} remediation", check.name),
                    body: remediation.clone(),
                });
                output.into_iter().chain(remediation)
            })
            .collect(),
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
        table: Table {
            headers: vec!["Task", "Label", "Command"],
            rows: plan
                .tasks
                .iter()
                .map(|task| {
                    vec![
                        task.to_string(),
                        task.title().to_owned(),
                        task.invocation().display(),
                    ]
                })
                .collect(),
        },
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
        table: Table {
            headers: vec!["Status", "Check", "Time", "Command"],
            rows: report
                .checks
                .iter()
                .map(|check| {
                    vec![
                        check.status.to_string(),
                        check.task.to_string(),
                        format!("{:.2}s", check.elapsed_seconds),
                        check.command.clone(),
                    ]
                })
                .collect(),
        },
        sections: report
            .checks
            .iter()
            .filter(|check| check.status == CheckStatus::Failed && !check.output.is_empty())
            .map(|check| Section {
                heading: format!("{} output", check.task),
                body: check.output.clone(),
            })
            .collect(),
    }
}

#[must_use]
pub fn action_plan_document(plan: &ActionPlan) -> Document {
    Document {
        title: format!("{} Plan", plan.title),
        summary: vec![format!("Actions: {} planned", plan.actions.len())],
        table: Table {
            headers: vec!["Step", "Action", "Detail"],
            rows: plan
                .actions
                .iter()
                .enumerate()
                .map(|(index, action)| {
                    vec![
                        (index + 1).to_string(),
                        action.label.clone(),
                        action.action.detail(),
                    ]
                })
                .collect(),
        },
        sections: Vec::new(),
    }
}

#[must_use]
pub fn external_pin_document(report: &ExternalPinReport) -> Document {
    Document {
        title: report.title.clone(),
        summary: vec![
            format!(
                "Status: {}",
                if report.drift_detected {
                    "review needed"
                } else {
                    "clean"
                }
            ),
            format!("Pins: {} checked", report.results.len()),
        ],
        table: Table {
            headers: vec![
                "Status",
                "Surface",
                "Source",
                "Dependency",
                "Pinned",
                "Latest",
            ],
            rows: report
                .results
                .iter()
                .map(|result| {
                    vec![
                        result.status.to_string(),
                        result.surface.to_string(),
                        result.source.clone(),
                        result.name.clone(),
                        result
                            .pinned
                            .clone()
                            .unwrap_or_else(|| "unavailable".to_owned()),
                        result
                            .latest
                            .clone()
                            .unwrap_or_else(|| "unavailable".to_owned()),
                    ]
                })
                .collect(),
        },
        sections: report
            .results
            .iter()
            .filter_map(|result| {
                result.message.as_ref().map(|message| Section {
                    heading: format!("{} detail", result.name),
                    body: message.clone(),
                })
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::doctor::{DoctorCheck, DoctorReport, DoctorStatus, DoctorSummary, ToolStatus};

    use super::doctor_document;

    fn unhealthy_report() -> DoctorReport {
        DoctorReport {
            summary: DoctorSummary {
                status: DoctorStatus::Unhealthy,
                total: 2,
                ok: 1,
                missing: 0,
                mismatched: 1,
            },
            checks: vec![
                DoctorCheck {
                    tool: "rustc".to_owned(),
                    expected: "1.96.1".to_owned(),
                    actual: Some("rustc 1.96.1".to_owned()),
                    status: ToolStatus::Ok,
                    remediation: None,
                },
                DoctorCheck {
                    tool: "cargo|deny".to_owned(),
                    expected: "0.19.0".to_owned(),
                    actual: Some("0.18.0\ninstall required".to_owned()),
                    status: ToolStatus::Mismatched,
                    remediation: Some("Update with Cargo.".to_owned()),
                },
            ],
        }
    }

    #[test]
    fn text_output_leads_with_summary_and_aligned_table() {
        let output = doctor_document(&unhealthy_report()).to_text();

        assert!(output.starts_with("Developer Environment\nStatus: unhealthy\n"));
        assert!(output.contains("Checks: 2 total, 1 ok, 0 missing, 1 mismatched"));
        assert!(output.contains("Status      Tool"));
    }

    #[test]
    fn markdown_output_escapes_cells() {
        let output = doctor_document(&unhealthy_report()).to_markdown();

        assert!(output.starts_with("## Developer Environment"));
        assert!(
            output.contains("| mismatched | cargo\\|deny | 0.19.0 | 0.18.0; install required |")
        );
        assert!(output.contains("### cargo|deny remediation"));
        assert!(output.contains("Update with Cargo."));
    }
}
