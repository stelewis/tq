use std::borrow::Cow;

use thiserror::Error;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum QualifierStrategy {
    None,
    #[default]
    AnySuffix,
    Allowlist,
}

impl QualifierStrategy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::AnySuffix => "any-suffix",
            Self::Allowlist => "allowlist",
        }
    }

    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "none" => Some(Self::None),
            "any-suffix" => Some(Self::AnySuffix),
            "allowlist" => Some(Self::Allowlist),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RuleId(Cow<'static, str>);

impl RuleId {
    pub fn parse(value: &str) -> Result<Self, RuleIdError> {
        validate_rule_id(value)?;
        Ok(Self(Cow::Owned(value.to_owned())))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RuleId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum RuleIdError {
    #[error("rule id must be non-empty")]
    Empty,
    #[error("rule id must be kebab-case, e.g. mapping-missing-test")]
    InvalidFormat,
}

fn validate_rule_id(value: &str) -> Result<(), RuleIdError> {
    if value.is_empty() {
        return Err(RuleIdError::Empty);
    }

    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(RuleIdError::Empty);
    };

    if !first.is_ascii_lowercase() {
        return Err(RuleIdError::InvalidFormat);
    }

    let mut previous_was_dash = false;
    for character in chars {
        if character == '-' {
            if previous_was_dash {
                return Err(RuleIdError::InvalidFormat);
            }
            previous_was_dash = true;
            continue;
        }

        if !character.is_ascii_lowercase() && !character.is_ascii_digit() {
            return Err(RuleIdError::InvalidFormat);
        }

        previous_was_dash = false;
    }

    if previous_was_dash {
        return Err(RuleIdError::InvalidFormat);
    }

    Ok(())
}
