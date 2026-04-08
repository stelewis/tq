use std::io::Write;
use std::path::Path;

use tq_engine::{EngineResult, Finding, Severity};

use crate::ReportingError;
use crate::path::display_path;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextStyling {
    #[default]
    Plain,
    Ansi,
}

impl TextStyling {
    const ANSI_RESET: &str = "\x1b[0m";
    const ANSI_BOLD: &str = "\x1b[1m";
    const ANSI_DIM: &str = "\x1b[2m";
    const ANSI_RED: &str = "\x1b[31m";
    const ANSI_YELLOW: &str = "\x1b[33m";
    const ANSI_BLUE: &str = "\x1b[34m";
    const ANSI_CYAN: &str = "\x1b[36m";
    const ANSI_BOLD_CYAN: &str = "\x1b[1;36m";
    const ANSI_BOLD_RED: &str = "\x1b[1;31m";

    #[must_use]
    pub const fn enabled(is_enabled: bool) -> Self {
        if is_enabled { Self::Ansi } else { Self::Plain }
    }

    #[must_use]
    pub fn success(self, text: &str) -> String {
        self.paint(text, Self::ANSI_BOLD_CYAN)
    }

    #[must_use]
    pub fn error_label(self, text: &str) -> String {
        self.paint(text, Self::ANSI_BOLD_RED)
    }

    #[must_use]
    fn summary_label(self, text: &str) -> String {
        self.paint(text, Self::ANSI_BOLD)
    }

    #[must_use]
    fn suggestion_label(self, text: &str) -> String {
        self.paint(text, Self::ANSI_CYAN)
    }

    #[must_use]
    fn target_prefix(self, text: &str) -> String {
        self.paint(text, Self::ANSI_DIM)
    }

    #[must_use]
    fn severity(self, severity: Severity) -> String {
        let code = match severity {
            Severity::Error => Self::ANSI_RED,
            Severity::Warning => Self::ANSI_YELLOW,
            Severity::Info => Self::ANSI_BLUE,
        };
        self.paint(render_severity(severity), code)
    }

    #[must_use]
    fn summary_segment(self, count: usize, label: &str, severity: Severity) -> String {
        self.paint(&format!("{count} {label}"), Self::severity_code(severity))
    }

    #[must_use]
    const fn severity_code(severity: Severity) -> &'static str {
        match severity {
            Severity::Error => Self::ANSI_RED,
            Severity::Warning => Self::ANSI_YELLOW,
            Severity::Info => Self::ANSI_BLUE,
        }
    }

    #[must_use]
    fn paint(self, text: &str, code: &str) -> String {
        match self {
            Self::Plain => text.to_owned(),
            Self::Ansi => format!("{code}{text}{}", Self::ANSI_RESET),
        }
    }
}

pub struct TextReporter<'a> {
    cwd: &'a Path,
    include_suggestions: bool,
    styling: TextStyling,
}

impl<'a> TextReporter<'a> {
    #[must_use]
    pub const fn new(cwd: &'a Path) -> Self {
        Self {
            cwd,
            include_suggestions: false,
            styling: TextStyling::Plain,
        }
    }

    #[must_use]
    pub const fn with_suggestions(mut self, include_suggestions: bool) -> Self {
        self.include_suggestions = include_suggestions;
        self
    }

    #[must_use]
    pub const fn with_styling(mut self, styling: TextStyling) -> Self {
        self.styling = styling;
        self
    }

    pub fn write<W: Write>(
        &self,
        writer: &mut W,
        result: &EngineResult,
    ) -> Result<(), ReportingError> {
        if result.findings().is_empty() {
            writeln!(writer, "{}", self.styling.success("All checks passed!"))?;
            return Ok(());
        }

        for finding in result.findings() {
            writeln!(writer, "{}", self.render_finding(finding))?;
        }

        let summary = result.summary();
        writeln!(
            writer,
            "{} {}, {}, {}",
            self.styling.summary_label("Summary:"),
            self.styling
                .summary_segment(summary.errors(), "error(s)", Severity::Error),
            self.styling
                .summary_segment(summary.warnings(), "warning(s)", Severity::Warning),
            self.styling
                .summary_segment(summary.infos(), "info finding(s)", Severity::Info),
        )?;

        Ok(())
    }

    fn render_finding(&self, finding: &Finding) -> String {
        let line_part = finding
            .line()
            .map_or_else(String::new, |line| format!(":{line}"));
        let rendered = format!(
            "{}{path}{line_part}: {} ({}) {}",
            render_target_prefix(self.styling, finding),
            self.styling.severity(finding.severity()),
            finding.rule_id().as_str(),
            finding.message(),
            path = display_path(finding.path(), self.cwd),
        );

        if self.include_suggestions
            && let Some(suggestion) = finding.suggestion()
        {
            return format!(
                "{rendered} ({}: {suggestion})",
                self.styling.suggestion_label("suggestion")
            );
        }

        rendered
    }
}

fn render_target_prefix(styling: TextStyling, finding: &Finding) -> String {
    finding.target().map_or_else(String::new, |target| {
        styling.target_prefix(&format!("target={} ", target.as_str()))
    })
}

const fn render_severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}
