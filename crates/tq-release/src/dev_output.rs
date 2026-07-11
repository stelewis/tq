use tq_release::{
    DevAuditReport, DevCheckPlan, DevCheckReport, DevCheckStatus, DevCommandPlan, DevDoctorReport,
    ExternalPinReport,
};

pub fn render_doctor_human(report: &DevDoctorReport) -> String {
    let mut output = vec![
        format!("Developer environment: {}", report.summary.status.label()),
        format!(
            "{} checks: {} ok, {} missing, {} mismatched",
            report.summary.total,
            report.summary.ok,
            report.summary.missing,
            report.summary.mismatched
        ),
        String::new(),
    ];
    output.push(render_text_table(
        &["Status", "Tool", "Expected", "Actual"],
        &report
            .checks
            .iter()
            .map(|check| {
                vec![
                    check.status.label().to_owned(),
                    check.tool.clone(),
                    check.expected.clone(),
                    check
                        .actual
                        .clone()
                        .unwrap_or_else(|| "not found".to_owned()),
                ]
            })
            .collect::<Vec<_>>(),
    ));
    output.push(String::new());
    output.join("\n")
}

pub fn render_doctor_agent(report: &DevDoctorReport) -> String {
    let mut output = vec![
        "## Developer Environment".to_owned(),
        String::new(),
        format!("Status: {}", report.summary.status.label()),
        format!(
            "Checks: {} total, {} ok, {} missing, {} mismatched",
            report.summary.total,
            report.summary.ok,
            report.summary.missing,
            report.summary.mismatched
        ),
        String::new(),
    ];
    output.push(render_markdown_table(
        &["Status", "Tool", "Expected", "Actual"],
        &report
            .checks
            .iter()
            .map(|check| {
                vec![
                    check.status.label().to_owned(),
                    check.tool.clone(),
                    check.expected.clone(),
                    check
                        .actual
                        .clone()
                        .unwrap_or_else(|| "not found".to_owned()),
                ]
            })
            .collect::<Vec<_>>(),
    ));
    output.push(String::new());
    output.join("\n")
}

pub fn render_audit_human(report: &DevAuditReport) -> String {
    let mut output = vec![
        format!("Dependency audit: {}", report.summary.status.label()),
        format!(
            "{} checks: {} clean, {} findings, {} failed",
            report.summary.total,
            report.summary.clean,
            report.summary.findings,
            report.summary.failed
        ),
        String::new(),
        render_text_table(
            &["Status", "Name", "Command"],
            &report
                .checks
                .iter()
                .map(|check| {
                    vec![
                        check.status.label().to_owned(),
                        check.name.clone(),
                        check.command.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
        ),
    ];

    for check in report
        .checks
        .iter()
        .filter(|check| !check.output.is_empty())
    {
        output.push(String::new());
        output.push(format!("{} output:", check.name));
        output.push(check.output.clone());
    }
    output.push(String::new());
    output.join("\n")
}

pub fn render_audit_agent(report: &DevAuditReport) -> String {
    let mut output = vec![
        "## Dependency Audit".to_owned(),
        String::new(),
        format!("Status: {}", report.summary.status.label()),
        format!(
            "Checks: {} total, {} clean, {} findings, {} failed",
            report.summary.total,
            report.summary.clean,
            report.summary.findings,
            report.summary.failed
        ),
        String::new(),
        render_markdown_table(
            &["Status", "Name", "Command"],
            &report
                .checks
                .iter()
                .map(|check| {
                    vec![
                        check.status.label().to_owned(),
                        check.name.clone(),
                        check.command.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
        ),
    ];

    for check in report
        .checks
        .iter()
        .filter(|check| !check.output.is_empty())
    {
        output.push(String::new());
        output.push(format!("### {} Output", check.name));
        output.push(String::new());
        output.push("```text".to_owned());
        output.push(check.output.clone());
        output.push("```".to_owned());
    }

    output.push(String::new());
    output.join("\n")
}

pub fn render_check_plan_human(plan: &DevCheckPlan) -> String {
    let mut output = vec![
        format!(
            "Developer checks: {} ({})",
            plan.target.label(),
            plan.profile.label()
        ),
        format!("{} tasks planned", plan.tasks.len()),
        String::new(),
        render_text_table(
            &["Task", "Label", "Command"],
            &plan
                .tasks
                .iter()
                .map(|task| vec![task.id.clone(), task.label.clone(), task.command.display()])
                .collect::<Vec<_>>(),
        ),
    ];
    output.push(String::new());
    output.join("\n")
}

pub fn render_check_plan_agent(plan: &DevCheckPlan) -> String {
    let mut output = vec![
        "## Developer Check Plan".to_owned(),
        String::new(),
        format!("Target: {}", plan.target.label()),
        format!("Profile: {}", plan.profile.label()),
        format!("Tasks: {} planned", plan.tasks.len()),
        String::new(),
        render_markdown_table(
            &["Task", "Label", "Command"],
            &plan
                .tasks
                .iter()
                .map(|task| vec![task.id.clone(), task.label.clone(), task.command.display()])
                .collect::<Vec<_>>(),
        ),
    ];
    output.push(String::new());
    output.join("\n")
}

pub fn render_check_human(report: &DevCheckReport) -> String {
    let mut output = vec![
        format!("Developer checks: {}", report.summary.status.label()),
        format!(
            "{} checks: {} passed, {} failed in {:.2}s",
            report.summary.total,
            report.summary.passed,
            report.summary.failed,
            report.summary.elapsed_seconds
        ),
        String::new(),
        render_text_table(
            &["Status", "Check", "Time"],
            &report
                .checks
                .iter()
                .map(|check| {
                    vec![
                        check.status.label().to_owned(),
                        check.task.label.clone(),
                        format!("{:.2}s", check.elapsed_seconds),
                    ]
                })
                .collect::<Vec<_>>(),
        ),
    ];

    for check in report
        .checks
        .iter()
        .filter(|check| check.status == DevCheckStatus::Failed && !check.output.is_empty())
    {
        output.push(String::new());
        output.push(format!("{} failed command:", check.task.id));
        output.push(check.task.command.display());
        output.push(String::new());
        output.push(format!("{} output:", check.task.id));
        output.push(check.output.clone());
    }
    output.push(String::new());
    output.join("\n")
}

pub fn render_check_agent(report: &DevCheckReport) -> String {
    let mut output = vec![
        "## Developer Checks".to_owned(),
        String::new(),
        format!("Status: {}", report.summary.status.label()),
        format!(
            "Checks: {} total, {} passed, {} failed in {:.2}s",
            report.summary.total,
            report.summary.passed,
            report.summary.failed,
            report.summary.elapsed_seconds
        ),
        String::new(),
        render_markdown_table(
            &["Status", "Task", "Time", "Command"],
            &report
                .checks
                .iter()
                .map(|check| {
                    vec![
                        check.status.label().to_owned(),
                        check.task.id.clone(),
                        format!("{:.2}s", check.elapsed_seconds),
                        check.task.command.display(),
                    ]
                })
                .collect::<Vec<_>>(),
        ),
    ];

    for check in report
        .checks
        .iter()
        .filter(|check| check.status == DevCheckStatus::Failed && !check.output.is_empty())
    {
        output.push(String::new());
        output.push(format!("### {} Output", check.task.id));
        output.push(String::new());
        output.push("```text".to_owned());
        output.push(check.output.clone());
        output.push("```".to_owned());
    }

    output.push(String::new());
    output.join("\n")
}

pub fn render_command_plan_human(plan: &DevCommandPlan) -> String {
    let mut output = vec![
        format!("{} dry run", plan.title),
        format!("{} commands planned", plan.commands.len()),
        String::new(),
        render_text_table(
            &["Step", "Command"],
            &plan
                .commands
                .iter()
                .enumerate()
                .map(|(index, command)| vec![(index + 1).to_string(), command.command.display()])
                .collect::<Vec<_>>(),
        ),
    ];
    output.push(String::new());
    output.join("\n")
}

pub fn render_command_plan_agent(plan: &DevCommandPlan) -> String {
    let mut output = vec![
        format!("## {} Dry Run", plan.title),
        String::new(),
        format!("Commands: {} planned", plan.commands.len()),
        String::new(),
        render_markdown_table(
            &["Step", "Command"],
            &plan
                .commands
                .iter()
                .enumerate()
                .map(|(index, command)| vec![(index + 1).to_string(), command.command.display()])
                .collect::<Vec<_>>(),
        ),
    ];
    output.push(String::new());
    output.join("\n")
}

pub fn render_external_pin_human(report: &ExternalPinReport) -> String {
    let mut output = vec![
        format!(
            "{}: {}",
            report.title,
            if report.drift_detected {
                "review needed"
            } else {
                "clean"
            }
        ),
        format!("{} pins checked", report.results.len()),
        String::new(),
    ];
    output.push(render_text_table(
        &[
            "Status",
            "Surface",
            "Source",
            "Dependency",
            "Pinned",
            "Latest",
        ],
        &report
            .results
            .iter()
            .map(|result| {
                vec![
                    result.status.label().to_owned(),
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
            .collect::<Vec<_>>(),
    ));
    output.push(String::new());
    output.join("\n")
}

pub fn render_external_pin_agent(report: &ExternalPinReport) -> String {
    let mut output = vec![
        format!("## {}", report.title),
        String::new(),
        format!(
            "Status: {}",
            if report.drift_detected {
                "review needed"
            } else {
                "clean"
            }
        ),
        format!("Pins: {} checked", report.results.len()),
        String::new(),
    ];
    output.push(render_markdown_table(
        &[
            "Status",
            "Surface",
            "Source",
            "Dependency",
            "Pinned",
            "Latest",
        ],
        &report
            .results
            .iter()
            .map(|result| {
                vec![
                    result.status.label().to_owned(),
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
            .collect::<Vec<_>>(),
    ));
    output.push(String::new());
    output.join("\n")
}

fn render_text_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let widths = column_widths(headers, rows);
    let mut lines = vec![render_text_row(headers, &widths)];
    lines.extend(
        rows.iter().map(|row| {
            render_text_row(&row.iter().map(String::as_str).collect::<Vec<_>>(), &widths)
        }),
    );
    lines.join("\n")
}

fn render_text_row(columns: &[&str], widths: &[usize]) -> String {
    columns
        .iter()
        .enumerate()
        .map(|(index, column)| format!("{column:<width$}", width = widths[index]))
        .collect::<Vec<_>>()
        .join("  ")
}

fn render_markdown_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut lines = vec![
        format!(
            "| {} |",
            headers
                .iter()
                .map(|header| markdown_cell(header))
                .collect::<Vec<_>>()
                .join(" | ")
        ),
        format!("| {} |", vec!["---"; headers.len()].join(" | ")),
    ];
    lines.extend(rows.iter().map(|row| {
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

fn column_widths(headers: &[&str], rows: &[Vec<String>]) -> Vec<usize> {
    let mut widths = headers
        .iter()
        .map(|header| header.len())
        .collect::<Vec<_>>();
    for row in rows {
        for (index, column) in row.iter().enumerate() {
            widths[index] = widths[index].max(column.len());
        }
    }
    widths
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}

#[cfg(test)]
mod tests {
    use tq_release::{
        DevAuditCheck, DevAuditReport, DevAuditReportStatus, DevAuditStatus, DevAuditSummary,
        DevDoctorCheck, DevDoctorReport, DevDoctorStatus, DevDoctorSummary, DevToolStatus,
    };

    use super::{render_audit_agent, render_doctor_agent, render_doctor_human};

    #[test]
    fn doctor_human_output_leads_with_summary_and_aligned_table() {
        let report = DevDoctorReport {
            summary: DevDoctorSummary {
                status: DevDoctorStatus::Unhealthy,
                total: 2,
                ok: 1,
                missing: 0,
                mismatched: 1,
            },
            checks: vec![
                DevDoctorCheck {
                    tool: "rustc".to_owned(),
                    expected: "1.96.1".to_owned(),
                    actual: Some("rustc 1.96.1".to_owned()),
                    status: DevToolStatus::Ok,
                },
                DevDoctorCheck {
                    tool: "cargo-deny".to_owned(),
                    expected: "0.19.0".to_owned(),
                    actual: Some("0.18.0".to_owned()),
                    status: DevToolStatus::Mismatched,
                },
            ],
        };

        let output = render_doctor_human(&report);

        assert!(output.starts_with("Developer environment: unhealthy\n"));
        assert!(output.contains("2 checks: 1 ok, 0 missing, 1 mismatched"));
        assert!(output.contains("Status      Tool        Expected  Actual"));
        assert!(output.contains("mismatched  cargo-deny  0.19.0    0.18.0"));
    }

    #[test]
    fn doctor_agent_output_is_markdown_safe() {
        let report = DevDoctorReport {
            summary: DevDoctorSummary {
                status: DevDoctorStatus::Unhealthy,
                total: 1,
                ok: 0,
                missing: 1,
                mismatched: 0,
            },
            checks: vec![DevDoctorCheck {
                tool: "cargo|audit".to_owned(),
                expected: "0.22.1".to_owned(),
                actual: Some("missing\ninstall required".to_owned()),
                status: DevToolStatus::Missing,
            }],
        };

        let output = render_doctor_agent(&report);

        assert!(output.contains("## Developer Environment"));
        assert!(
            output.contains("| missing | cargo\\|audit | 0.22.1 | missing<br>install required |")
        );
    }

    #[test]
    fn audit_agent_output_keeps_command_output_in_text_fences() {
        let report = DevAuditReport {
            summary: DevAuditSummary {
                status: DevAuditReportStatus::Findings,
                total: 1,
                clean: 0,
                findings: 1,
                failed: 0,
            },
            checks: vec![DevAuditCheck {
                name: "cargo".to_owned(),
                command: "cargo outdated --workspace".to_owned(),
                status: DevAuditStatus::Findings,
                output: "crate latest: 2.0.0".to_owned(),
            }],
        };

        let output = render_audit_agent(&report);

        assert!(output.contains("Status: findings"));
        assert!(output.contains("| findings | cargo | cargo outdated --workspace |"));
        assert!(output.contains("```text\ncrate latest: 2.0.0\n```"));
    }
}
