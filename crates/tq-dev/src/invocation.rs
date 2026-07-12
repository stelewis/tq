//! Subprocess invocation: the single boundary where harness commands run.

use std::path::Path;
use std::process::Command;

use serde::Serialize;

use crate::error::DevError;
use crate::native_env;

/// A fully resolved subprocess invocation: explicit program, argv, and
/// environment additions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Invocation {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

impl Invocation {
    #[must_use]
    pub fn new<const N: usize>(program: &str, args: [&str; N]) -> Self {
        Self {
            program: program.to_owned(),
            args: args.into_iter().map(ToOwned::to_owned).collect(),
            env: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_args<I, A>(program: &str, args: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: Into<String>,
    {
        Self {
            program: program.to_owned(),
            args: args.into_iter().map(Into::into).collect(),
            env: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_env(mut self, env: Vec<(String, String)>) -> Self {
        self.env = env;
        self
    }

    /// Shell-style rendering for reports and dry-run plans.
    #[must_use]
    pub fn display(&self) -> String {
        let mut parts = self
            .env
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>();
        parts.push(self.program.clone());
        parts.extend(self.args.iter().cloned());
        parts.join(" ")
    }

    /// Runs the invocation with inherited stdio, failing on non-zero exit.
    pub fn run(&self, repo_root: &Path) -> Result<(), DevError> {
        let status = self
            .command(repo_root)
            .status()
            .map_err(|source| self.io_error(source))?;
        if status.success() {
            Ok(())
        } else {
            Err(DevError::CommandFailed {
                program: self.program.clone(),
                args: self.args.clone(),
                code: status.code(),
            })
        }
    }

    /// Runs the invocation capturing output. A non-zero exit is returned as
    /// data, not as an error; spawn failures are errors.
    pub fn capture(&self, repo_root: &Path) -> Result<Captured, DevError> {
        let output = self
            .command(repo_root)
            .output()
            .map_err(|source| self.io_error(source))?;
        Ok(Captured {
            code: output.status.code(),
            success: output.status.success(),
            output: merge_output(&output.stdout, &output.stderr),
        })
    }

    fn command(&self, repo_root: &Path) -> Command {
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .current_dir(repo_root)
            .envs(native_env::native_build_env())
            .envs(self.env.iter().map(|(key, value)| (key, value)));
        command
    }

    fn io_error(&self, source: std::io::Error) -> DevError {
        DevError::CommandIo {
            program: self.program.clone(),
            args: self.args.clone(),
            source,
        }
    }
}

/// Captured result of an invocation.
#[derive(Debug)]
pub struct Captured {
    pub code: Option<i32>,
    pub success: bool,
    pub output: String,
}

fn merge_output(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(stderr).trim().to_owned();

    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => String::new(),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("{stdout}\n{stderr}"),
    }
}

/// Captures `program --args` output without a repository working directory,
/// for probing tools on `PATH`.
pub fn probe(program: &str, args: &[&str]) -> std::io::Result<Captured> {
    let output = Command::new(program).args(args).output()?;
    Ok(Captured {
        code: output.status.code(),
        success: output.status.success(),
        output: merge_output(&output.stdout, &output.stderr),
    })
}
