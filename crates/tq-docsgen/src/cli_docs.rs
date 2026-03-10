use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use clap::{Arg, ArgAction, CommandFactory};
use serde::Deserialize;
use tq_cli::Cli;

use crate::{DocsgenError, markers::replace_between_markers};

const MANIFEST_PATH: &str = "docs/reference/cli/options-manifest.yaml";
const CLI_DOC_PATH: &str = "docs/reference/cli.md";
const CLI_MARKER_START: &str = "<!-- BEGIN GENERATED:check-options -->";
const CLI_MARKER_END: &str = "<!-- END GENERATED:check-options -->";

#[derive(Debug, Deserialize)]
struct CliManifest {
    cli_options: Vec<CliOptionSpec>,
}

#[derive(Debug, Deserialize)]
struct CliOptionSpec {
    arg_ids: Vec<String>,
    config_key: Option<String>,
}

pub fn generate(workspace_root: &Path) -> Result<(), DocsgenError> {
    let manifest_path = workspace_root.join(MANIFEST_PATH);
    let cli_doc_path = workspace_root.join(CLI_DOC_PATH);
    let manifest = load_manifest(&manifest_path)?;
    let args_by_id = load_check_args()?;
    validate_manifest(&manifest, &args_by_id, &manifest_path)?;

    let rendered = render_cli_table(&manifest, &args_by_id, &manifest_path)?;
    replace_between_markers(&cli_doc_path, CLI_MARKER_START, CLI_MARKER_END, &rendered)
}

fn load_manifest(path: &Path) -> Result<Vec<CliOptionSpec>, DocsgenError> {
    let content = std::fs::read_to_string(path).map_err(|source| DocsgenError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let manifest: CliManifest =
        serde_yaml::from_str(&content).map_err(|source| DocsgenError::Yaml {
            path: path.to_path_buf(),
            source,
        })?;

    if manifest.cli_options.is_empty() {
        return Err(DocsgenError::manifest(
            path.to_path_buf(),
            "cli_options must define at least one entry",
        ));
    }

    for spec in &manifest.cli_options {
        if spec.arg_ids.is_empty() {
            return Err(DocsgenError::manifest(
                path.to_path_buf(),
                "each cli_options entry must define at least one arg_ids value",
            ));
        }

        if spec
            .config_key
            .as_deref()
            .is_some_and(|config_key| config_key.trim().is_empty())
        {
            return Err(DocsgenError::manifest(
                path.to_path_buf(),
                "config_key must be null or a non-empty string",
            ));
        }
    }

    Ok(manifest.cli_options)
}

fn load_check_args() -> Result<BTreeMap<String, Arg>, DocsgenError> {
    let mut command = Cli::command();
    let check_command = command
        .find_subcommand_mut("check")
        .ok_or(DocsgenError::MissingCheckSubcommand)?;

    Ok(check_command
        .get_arguments()
        .filter(|arg| !arg.is_hide_set())
        .map(|arg| (arg.get_id().to_string(), arg.clone()))
        .collect())
}

fn validate_manifest(
    manifest: &[CliOptionSpec],
    args_by_id: &BTreeMap<String, Arg>,
    manifest_path: &Path,
) -> Result<(), DocsgenError> {
    let manifest_ids = manifest
        .iter()
        .flat_map(|spec| spec.arg_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let actual_ids = args_by_id.keys().cloned().collect::<BTreeSet<_>>();

    let missing = actual_ids
        .difference(&manifest_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(DocsgenError::manifest(
            manifest_path.to_path_buf(),
            format!("manifest missing Rust CLI args: {}", missing.join(", ")),
        ));
    }

    let unknown = manifest_ids
        .difference(&actual_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(DocsgenError::manifest(
            manifest_path.to_path_buf(),
            format!(
                "manifest contains unknown Rust CLI args: {}",
                unknown.join(", ")
            ),
        ));
    }

    Ok(())
}

fn render_cli_table(
    manifest: &[CliOptionSpec],
    args_by_id: &BTreeMap<String, Arg>,
    manifest_path: &Path,
) -> Result<String, DocsgenError> {
    let mut lines = vec![
        "## `tq check` options".to_owned(),
        String::new(),
        "The table below documents command options.".to_owned(),
        String::new(),
        "| Flags | Config key | Default | Description |".to_owned(),
        "| --- | --- | --- | --- |".to_owned(),
    ];

    for spec in manifest {
        let args = spec
            .arg_ids
            .iter()
            .map(|arg_id| {
                args_by_id.get(arg_id).cloned().ok_or_else(|| {
                    DocsgenError::manifest(
                        manifest_path.to_path_buf(),
                        format!("manifest references unknown arg id `{arg_id}`"),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        lines.push(format!(
            "| `{}` | {} | `{}` | {} |",
            render_flags(&args),
            render_config_key(spec.config_key.as_deref()),
            render_default(&args),
            render_help(&args),
        ));
    }

    lines.push(String::new());
    lines.push("Run `tq check --help` for the runtime source of truth.".to_owned());
    lines.push(String::new());

    Ok(lines.join("\n"))
}

fn render_flags(args: &[Arg]) -> String {
    let mut flags = Vec::new();
    let mut seen = BTreeSet::new();

    for arg in args {
        if let Some(short) = arg.get_short() {
            let flag = format!("-{short}");
            if seen.insert(flag.clone()) {
                flags.push(flag);
            }
        }

        if let Some(long) = arg.get_long() {
            let flag = format!("--{long}");
            if seen.insert(flag.clone()) {
                flags.push(flag);
            }
        }
    }

    flags.join(", ")
}

fn render_help(args: &[Arg]) -> String {
    args.iter()
        .filter_map(|arg| arg.get_help())
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(" / ")
        .replace('|', "\\|")
}

fn render_default(args: &[Arg]) -> String {
    let Some(arg) = args.first() else {
        return "none".to_owned();
    };

    let default_values = arg
        .get_default_values()
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if !default_values.is_empty() {
        return if default_values.len() == 1 {
            default_values[0].clone()
        } else {
            format!("[{}]", default_values.join(", "))
        };
    }

    match arg.get_action() {
        ArgAction::Append => "[]".to_owned(),
        ArgAction::SetTrue | ArgAction::SetFalse => {
            if args.len() == 1 {
                "false".to_owned()
            } else {
                "none".to_owned()
            }
        }
        _ => "none".to_owned(),
    }
}

fn render_config_key(config_key: Option<&str>) -> String {
    let Some(config_key) = config_key else {
        return "—".to_owned();
    };

    let suffix = if matches!(config_key, "package" | "source_root" | "test_root") {
        "required"
    } else {
        "optional"
    };
    format!("[`{config_key}`](./configuration.md#{config_key}-{suffix})")
}
