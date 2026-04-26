use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::ReleaseError;
use crate::ReleaseIntentCheck;

pub const RELEASE_INTENT_LABELS: [&str; 3] = ["release:none", "release:patch", "release:minor"];

const CONTRACT_DOC_PREFIXES: [&str; 1] = ["docs/reference/"];

const SHIPPED_RUNTIME_SOURCE_PREFIXES: [&str; 7] = [
    "crates/tq-cli/src/",
    "crates/tq-config/src/",
    "crates/tq-core/src/",
    "crates/tq-discovery/src/",
    "crates/tq-engine/src/",
    "crates/tq-reporting/src/",
    "crates/tq-rules/src/",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReleaseIntent {
    None,
    Patch,
    Minor,
}

impl ReleaseIntent {
    fn from_label(label: &str) -> Option<Self> {
        match label {
            "release:none" => Some(Self::None),
            "release:patch" => Some(Self::Patch),
            "release:minor" => Some(Self::Minor),
            _ => None,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::None => "release:none",
            Self::Patch => "release:patch",
            Self::Minor => "release:minor",
        }
    }

    const fn requires_release_metadata(self) -> bool {
        !matches!(self, Self::None)
    }
}

pub fn verify_release_intent(input: ReleaseIntentCheck<'_>) -> Result<(), ReleaseError> {
    let ReleaseIntentCheck {
        labels,
        changed_files,
        version_updated,
        changelog_updated,
        runtime_dependency_changed,
    } = input;
    let release_intent = declared_release_intent(labels)?;
    let mut violations = Vec::new();

    if release_intent.requires_release_metadata() {
        if !version_updated {
            violations.push(format!(
                "{} requires a workspace version update in Cargo.toml",
                release_intent.label()
            ));
        }

        if !changelog_updated {
            violations.push(format!(
                "{} requires a new top CHANGELOG.md release heading",
                release_intent.label()
            ));
        }
    } else {
        if version_updated {
            violations.push(
                "release:none cannot be used when the workspace version changed; remove the version update or choose release:patch or release:minor"
                    .to_owned(),
            );
        }

        if changelog_updated {
            violations.push(
                "release:none cannot be used when CHANGELOG.md was prepared for a new release; remove the changelog release heading or choose release:patch or release:minor"
                    .to_owned(),
            );
        }

        if runtime_dependency_changed {
            violations.push(
                "release:none conflicts with a shipped runtime dependency change; choose release:patch or release:minor"
                    .to_owned(),
            );
        }

        for (signal, files) in suspicious_release_none_changes(changed_files) {
            violations.push(format!(
                "release:none conflicts with {signal}: {}. Choose release:patch or release:minor if the product contract or shipped behavior changed",
                files.join(", ")
            ));
        }
    }

    if violations.is_empty() {
        return Ok(());
    }

    Err(ReleaseError::RepositoryPolicyViolation {
        details: violations.join("\n"),
    })
}

fn declared_release_intent(labels: &[String]) -> Result<ReleaseIntent, ReleaseError> {
    let unknown_release_labels = labels
        .iter()
        .filter(|label| label.starts_with("release:") && ReleaseIntent::from_label(label).is_none())
        .map(String::as_str)
        .collect::<Vec<_>>();

    if !unknown_release_labels.is_empty() {
        return Err(ReleaseError::RepositoryPolicyViolation {
            details: format!(
                "unknown release intent labels: {}. Use exactly one of {}",
                unknown_release_labels.join(", "),
                RELEASE_INTENT_LABELS.join(", ")
            ),
        });
    }

    let release_labels = labels
        .iter()
        .filter_map(|label| ReleaseIntent::from_label(label).map(ReleaseIntent::label))
        .collect::<Vec<_>>();

    if release_labels.is_empty() {
        return Err(ReleaseError::RepositoryPolicyViolation {
            details: format!(
                "pull requests must declare exactly one release intent label; add one of {}",
                RELEASE_INTENT_LABELS.join(", ")
            ),
        });
    }

    let mut unique_labels = release_labels;
    unique_labels.sort_unstable();
    unique_labels.dedup();

    if unique_labels.len() != 1 {
        return Err(ReleaseError::RepositoryPolicyViolation {
            details: format!(
                "pull requests must declare exactly one release intent label; found {}",
                unique_labels.join(", ")
            ),
        });
    }

    ReleaseIntent::from_label(unique_labels[0]).ok_or_else(|| {
        ReleaseError::RepositoryPolicyViolation {
            details: format!(
                "pull requests must declare exactly one release intent label; add one of {}",
                RELEASE_INTENT_LABELS.join(", ")
            ),
        }
    })
}

fn suspicious_release_none_changes(
    changed_files: &[PathBuf],
) -> BTreeMap<&'static str, Vec<String>> {
    let mut signals = BTreeMap::new();

    for changed_file in changed_files {
        let changed_file = changed_file.to_string_lossy().replace('\\', "/");
        if SHIPPED_RUNTIME_SOURCE_PREFIXES
            .iter()
            .any(|prefix| changed_file.starts_with(prefix))
        {
            signals
                .entry("shipped runtime source changes")
                .or_insert_with(Vec::new)
                .push(changed_file.clone());
        }

        if CONTRACT_DOC_PREFIXES
            .iter()
            .any(|prefix| changed_file.starts_with(prefix))
        {
            signals
                .entry("contract policy or reference doc changes")
                .or_insert_with(Vec::new)
                .push(changed_file);
        }
    }

    signals
}
