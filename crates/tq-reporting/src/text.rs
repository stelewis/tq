use std::io::Write;
use std::path::Path;

use tq_engine::{EngineResult, Finding, Severity};

use crate::ReportingError;
use crate::path::display_path;

pub struct TextReporter<'a> {
    cwd: &'a Path,
    include_suggestions: bool,
}

impl<'a> TextReporter<'a> {
    #[must_use]
    pub const fn new(cwd: &'a Path) -> Self {
        Self {
            cwd,
            include_suggestions: false,
        }
    }

    #[must_use]
    pub const fn with_suggestions(mut self, include_suggestions: bool) -> Self {
        self.include_suggestions = include_suggestions;
        self
    }

    pub fn write<W: Write>(
        &self,
        writer: &mut W,
        result: &EngineResult,
    ) -> Result<(), ReportingError> {
        if result.findings().is_empty() {
            writeln!(writer, "All checks passed!")?;
            return Ok(());
        }

        for finding in result.findings() {
            writeln!(writer, "{}", self.render_finding(finding))?;
        }

        let summary = result.summary();
        writeln!(
            writer,
            "Summary: {} error(s), {} warning(s), {} info finding(s)",
            summary.errors(),
            summary.warnings(),
            summary.infos(),
        )?;

        Ok(())
    }

    fn render_finding(&self, finding: &Finding) -> String {
        let line_part = finding
            .line()
            .map_or_else(String::new, |line| format!(":{line}"));
        let rendered = format!(
            "{}{path}{line_part}: {} ({}) {}",
            render_target_prefix(finding),
            render_severity(finding.severity()),
            finding.rule_id().as_str(),
            finding.message(),
            path = display_path(finding.path(), self.cwd),
        );

        if self.include_suggestions
            && let Some(suggestion) = finding.suggestion()
        {
            return format!("{rendered} (suggestion: {suggestion})");
        }

        rendered
    }
}

fn render_target_prefix(finding: &Finding) -> String {
    finding
        .target()
        .map_or_else(String::new, |target| format!("target={} ", target.as_str()))
}

const fn render_severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}
