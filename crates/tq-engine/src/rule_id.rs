use crate::EngineError;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RuleId(String);

impl RuleId {
    pub fn parse(value: &str) -> Result<Self, EngineError> {
        if value.is_empty() {
            return Err(EngineError::Validation {
                message: "RuleId must be non-empty".to_owned(),
            });
        }

        let mut chars = value.chars();
        let Some(first) = chars.next() else {
            return Err(EngineError::Validation {
                message: "RuleId must be non-empty".to_owned(),
            });
        };

        if !first.is_ascii_lowercase() {
            return Err(EngineError::Validation {
                message: "RuleId must be kebab-case, e.g. mapping-missing-test".to_owned(),
            });
        }

        let mut previous_was_dash = false;
        for character in chars {
            if character == '-' {
                if previous_was_dash {
                    return Err(EngineError::Validation {
                        message: "RuleId must be kebab-case, e.g. mapping-missing-test".to_owned(),
                    });
                }
                previous_was_dash = true;
                continue;
            }

            if !character.is_ascii_lowercase() && !character.is_ascii_digit() {
                return Err(EngineError::Validation {
                    message: "RuleId must be kebab-case, e.g. mapping-missing-test".to_owned(),
                });
            }

            previous_was_dash = false;
        }

        if previous_was_dash {
            return Err(EngineError::Validation {
                message: "RuleId must be kebab-case, e.g. mapping-missing-test".to_owned(),
            });
        }

        Ok(Self(value.to_owned()))
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
