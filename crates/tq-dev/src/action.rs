//! Action plans: the single source of truth for every mutating harness
//! command.
//!
//! Mutating commands build an [`ActionPlan`] and either render it (dry run)
//! or apply it. Execution interprets the plan, so what a dry run prints is
//! exactly what an apply performs.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::DevError;
use crate::invocation::Invocation;
use crate::parse;

/// An ordered plan of labeled actions.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct ActionPlan {
    pub title: String,
    pub actions: Vec<PlannedAction>,
}

impl ActionPlan {
    #[must_use]
    pub fn new(title: &str, actions: Vec<PlannedAction>) -> Self {
        Self {
            title: title.to_owned(),
            actions,
        }
    }

    /// Applies every action in order, failing fast on the first error.
    pub fn apply(&self, repo_root: &Path) -> Result<(), DevError> {
        for action in &self.actions {
            action.action.apply(repo_root)?;
        }
        Ok(())
    }
}

/// A single action with a human-readable label attached at construction.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct PlannedAction {
    pub label: String,
    pub action: Action,
}

impl PlannedAction {
    #[must_use]
    pub fn command(label: &str, invocation: Invocation) -> Self {
        Self {
            label: label.to_owned(),
            action: Action::Command { invocation },
        }
    }

    #[must_use]
    pub fn replace_text(label: &str, path: PathBuf, from: String, to: String) -> Self {
        Self {
            label: label.to_owned(),
            action: Action::ReplaceText { path, from, to },
        }
    }

    #[must_use]
    pub fn remove_path(label: &str, path: PathBuf) -> Self {
        Self {
            label: label.to_owned(),
            action: Action::RemovePath { path },
        }
    }
}

/// The closed set of mutations the harness can perform.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Action {
    Command {
        invocation: Invocation,
    },
    ReplaceText {
        path: PathBuf,
        from: String,
        to: String,
    },
    RemovePath {
        path: PathBuf,
    },
}

impl Action {
    fn apply(&self, repo_root: &Path) -> Result<(), DevError> {
        match self {
            Self::Command { invocation } => invocation.run(repo_root),
            Self::ReplaceText { path, from, to } => replace_once(path, from, to),
            Self::RemovePath { path } => remove_path(path),
        }
    }

    /// One-line rendering for plan output.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Command { invocation } => invocation.display(),
            Self::ReplaceText { path, from, to } => {
                format!("replace {from:?} with {to:?} in {}", path.display())
            }
            Self::RemovePath { path } => format!("remove {}", path.display()),
        }
    }
}

fn replace_once(path: &Path, old: &str, new: &str) -> Result<(), DevError> {
    let contents = parse::read_to_string(path)?;
    let count = contents.matches(old).count();
    if count != 1 {
        return Err(DevError::InvalidInput {
            path: path.to_path_buf(),
            message: format!("expected exactly one occurrence of {old:?}, found {count}"),
        });
    }

    std::fs::write(path, contents.replace(old, new)).map_err(|source| DevError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn remove_path(path: &Path) -> Result<(), DevError> {
    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match result {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(DevError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}
