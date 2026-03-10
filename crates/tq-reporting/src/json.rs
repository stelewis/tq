use std::io::Write;
use std::path::Path;

use serde::Serialize;
use tq_engine::{EngineResult, Finding};

use crate::ReportingError;
use crate::path::display_path;

pub struct JsonReporter<'a> {
    cwd: &'a Path,
}

impl<'a> JsonReporter<'a> {
    #[must_use]
    pub const fn new(cwd: &'a Path) -> Self {
        Self { cwd }
    }

    pub fn write<W: Write>(
        &self,
        writer: &mut W,
        result: &EngineResult,
    ) -> Result<(), ReportingError> {
        let payload = JsonReport {
            findings: result
                .findings()
                .iter()
                .map(|finding| JsonFinding::from_finding(finding, self.cwd))
                .collect(),
            summary: JsonSummary::from_result(result),
        };

        serde_json::to_writer(&mut *writer, &payload)?;
        writer.write_all(b"\n")?;
        Ok(())
    }
}

#[derive(Serialize)]
struct JsonReport<'a> {
    findings: Vec<JsonFinding<'a>>,
    summary: JsonSummary,
}

#[derive(Serialize)]
struct JsonFinding<'a> {
    rule_id: &'a str,
    severity: &'a str,
    message: &'a str,
    path: String,
    line: Option<u32>,
    suggestion: Option<&'a str>,
    target: Option<&'a str>,
}

impl<'a> JsonFinding<'a> {
    fn from_finding(finding: &'a Finding, cwd: &Path) -> Self {
        Self {
            rule_id: finding.rule_id().as_str(),
            severity: finding.severity().as_str(),
            message: finding.message(),
            path: display_path(finding.path(), cwd),
            line: finding.line(),
            suggestion: finding.suggestion(),
            target: finding.target(),
        }
    }
}

#[derive(Serialize)]
struct JsonSummary {
    errors: usize,
    warnings: usize,
    infos: usize,
    total: usize,
}

impl JsonSummary {
    const fn from_result(result: &EngineResult) -> Self {
        let summary = result.summary();
        Self {
            errors: summary.errors(),
            warnings: summary.warnings(),
            infos: summary.infos(),
            total: summary.total(),
        }
    }
}
