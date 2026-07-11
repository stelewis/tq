//! Strict readers for the repository-owned TOML, JSON, and YAML surfaces the
//! harness verifies.
//!
//! YAML handling is deliberately line-oriented and limited to the shapes this
//! repository commits. See ADR 0003 for the supply-chain rationale.

use std::collections::BTreeMap;
use std::path::Path;

use toml::Value;

use crate::error::DevError;

pub fn read_to_string(path: &Path) -> Result<String, DevError> {
    std::fs::read_to_string(path).map_err(|source| DevError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub fn read_toml(path: &Path) -> Result<Value, DevError> {
    let contents = read_to_string(path)?;
    contents
        .parse::<toml::Table>()
        .map(Value::Table)
        .map_err(|source| DevError::InvalidInput {
            path: path.to_path_buf(),
            message: source.to_string(),
        })
}

pub fn read_json(path: &Path) -> Result<serde_json::Value, DevError> {
    let contents = read_to_string(path)?;
    serde_json::from_str(&contents).map_err(|source| DevError::InvalidInput {
        path: path.to_path_buf(),
        message: source.to_string(),
    })
}

pub fn required_string(document: &Value, path: &[&str], source: &Path) -> Result<String, DevError> {
    required_value(document, path, source)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| invalid_type(source, path, "string"))
}

pub fn required_integer(document: &Value, path: &[&str], source: &Path) -> Result<i64, DevError> {
    required_value(document, path, source)?
        .as_integer()
        .ok_or_else(|| invalid_type(source, path, "integer"))
}

pub fn required_string_array(
    document: &Value,
    path: &[&str],
    source: &Path,
) -> Result<Vec<String>, DevError> {
    required_value(document, path, source)?
        .as_array()
        .ok_or_else(|| invalid_type(source, path, "array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| invalid_type(source, path, "string array"))
        })
        .collect()
}

pub fn required_json_string(
    document: &serde_json::Value,
    path: &[&str],
    source: &Path,
) -> Result<String, DevError> {
    let mut current = document;
    for segment in path {
        current = current
            .get(*segment)
            .ok_or_else(|| missing_value(source, path))?;
    }
    current
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| invalid_type(source, path, "string"))
}

fn required_value<'a>(
    document: &'a Value,
    path: &[&str],
    source: &Path,
) -> Result<&'a Value, DevError> {
    let mut current = document;
    for segment in path {
        current = current
            .get(*segment)
            .ok_or_else(|| missing_value(source, path))?;
    }
    Ok(current)
}

fn invalid_type(source: &Path, path: &[&str], expected: &str) -> DevError {
    DevError::InvalidInput {
        path: source.to_path_buf(),
        message: format!("{} must be a {expected}", path.join(".")),
    }
}

fn missing_value(source: &Path, path: &[&str]) -> DevError {
    DevError::InvalidInput {
        path: source.to_path_buf(),
        message: format!("missing {}", path.join(".")),
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ActionInput {
    pub default: Option<String>,
}

/// Parses the `inputs` block of a composite GitHub Action definition.
pub fn action_inputs(
    contents: &str,
    source: &Path,
) -> Result<BTreeMap<String, ActionInput>, DevError> {
    let mut inputs = BTreeMap::new();
    let mut in_inputs = false;
    let mut current_input: Option<String> = None;

    for line in contents.lines() {
        let Some((indent, key, value)) = yaml_key_value(line) else {
            continue;
        };

        if indent == 0 {
            if key == "inputs" {
                in_inputs = true;
                current_input = None;
                continue;
            }
            if in_inputs {
                break;
            }
        }

        if !in_inputs {
            continue;
        }

        match indent {
            2 => {
                inputs.insert(key.to_owned(), ActionInput { default: None });
                current_input = Some(key.to_owned());
            }
            4 if key == "default" => {
                let Some(input) = current_input.as_deref() else {
                    return Err(DevError::InvalidInput {
                        path: source.to_path_buf(),
                        message: "action input default appeared before an input name".to_owned(),
                    });
                };
                let Some(entry) = inputs.get_mut(input) else {
                    return Err(DevError::InvalidInput {
                        path: source.to_path_buf(),
                        message: format!("missing action input {input}"),
                    });
                };
                entry.default = Some(yaml_scalar(value).to_owned());
            }
            _ => {}
        }
    }

    Ok(inputs)
}

pub fn required_action_input_default(
    inputs: &BTreeMap<String, ActionInput>,
    input: &str,
    source: &Path,
) -> Result<String, DevError> {
    inputs
        .get(input)
        .ok_or_else(|| missing_value(source, &["inputs", input]))?
        .default
        .clone()
        .ok_or_else(|| missing_value(source, &["inputs", input, "default"]))
}

/// Finds the `with.<key>` value of the first step whose `uses` reference
/// starts with `uses_prefix`.
pub fn required_action_step_with_value(
    contents: &str,
    uses_prefix: &str,
    with_key: &str,
    source: &Path,
) -> Result<String, DevError> {
    let mut in_target_step = false;
    let mut in_with = false;

    for line in contents.lines() {
        let Some((indent, key, value)) = yaml_key_value(line) else {
            continue;
        };

        if indent == 4 && key == "name" {
            in_target_step = false;
            in_with = false;
            continue;
        }

        if (indent == 4 || indent == 6) && key == "uses" {
            in_target_step = yaml_scalar(value).starts_with(uses_prefix);
            in_with = false;
            continue;
        }

        if in_target_step && indent == 6 && key == "with" {
            in_with = true;
            continue;
        }

        if in_target_step && in_with && indent == 8 && key == with_key {
            return Ok(yaml_scalar(value).to_owned());
        }
    }

    Err(missing_value(
        source,
        &["runs", "steps", uses_prefix, "with", with_key],
    ))
}

fn yaml_key_value(line: &str) -> Option<(usize, &str, &str)> {
    let without_comment = line.split('#').next().unwrap_or(line);
    let trimmed_end = without_comment.trim_end();
    if trimmed_end.trim().is_empty() {
        return None;
    }
    let indent = trimmed_end.len() - trimmed_end.trim_start().len();
    let trimmed = trimmed_end
        .trim_start()
        .strip_prefix("- ")
        .unwrap_or_else(|| trimmed_end.trim_start());
    let (key, value) = trimmed.split_once(':')?;
    Some((indent, key.trim(), value.trim()))
}

#[must_use]
pub fn yaml_scalar(value: &str) -> &str {
    value
        .split('#')
        .next()
        .unwrap_or(value)
        .trim()
        .trim_matches(['\'', '"'])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{action_inputs, required_action_input_default, required_action_step_with_value};

    #[test]
    fn parses_action_input_defaults_and_step_with_values() {
        let contents = concat!(
            "inputs:\n",
            "  uv-version:\n",
            "    description: uv version\n",
            "    default: '0.11.28'\n",
            "runs:\n",
            "  using: composite\n",
            "  steps:\n",
            "    - name: Set up uv\n",
            "      uses: astral-sh/setup-uv@example\n",
            "      with:\n",
            "        version: ${{ inputs.uv-version }}\n",
        );
        let source = Path::new("action.yml");
        let inputs = action_inputs(contents, source).expect("action inputs should parse");

        assert_eq!(
            required_action_input_default(&inputs, "uv-version", source)
                .expect("uv-version default should exist"),
            "0.11.28"
        );
        assert_eq!(
            required_action_step_with_value(contents, "astral-sh/setup-uv@", "version", source)
                .expect("setup-uv version input should exist"),
            "${{ inputs.uv-version }}"
        );
    }
}
