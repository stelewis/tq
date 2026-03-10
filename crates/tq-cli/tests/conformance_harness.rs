use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ScenarioManifest {
    category: String,
    description: String,
    args: Vec<String>,
    #[serde(default = "default_repeat")]
    repeat: usize,
    stdout_format: StdoutFormat,
    expectation: ScenarioExpectation,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum StdoutFormat {
    Text,
    Json,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
enum ScenarioExpectation {
    Match {
        expected: ExpectedOutcome,
    },
    IntentionalDelta {
        reason: String,
        python: ExpectedOutcome,
        rust: ExpectedOutcome,
    },
}

#[derive(Debug, Deserialize)]
struct ExpectedOutcome {
    exit_code: i32,
    stdout: String,
    #[serde(default)]
    stderr: Option<String>,
}

#[derive(Debug)]
struct Scenario {
    name: String,
    directory: PathBuf,
    manifest: ScenarioManifest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RuntimeOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

#[derive(Debug)]
struct ScenarioReport {
    name: String,
    category: String,
    status: &'static str,
    detail: String,
}

#[derive(Debug)]
struct PythonRuntime {
    executable: PathBuf,
    python_path: PathBuf,
}

const fn default_repeat() -> usize {
    1
}

#[test]
#[ignore = "requires Python baseline runtime"]
fn conformance_harness() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = workspace_root();
    let fixtures_root = workspace_root.join("crates/tq-cli/tests/fixtures/conformance");
    let python_runtime = detect_python_runtime(&workspace_root)?;
    let scenarios = load_scenarios(&fixtures_root)?;

    let mut reports = Vec::with_capacity(scenarios.len());
    let mut failures = Vec::new();

    for scenario in &scenarios {
        match validate_scenario(&workspace_root, &python_runtime, scenario) {
            Ok(report) => reports.push(report),
            Err(error) => failures.push(format!("scenario '{}' failed: {error}", scenario.name)),
        }
    }

    print_report(&reports, &failures);

    if failures.is_empty() {
        Ok(())
    } else {
        Err(io::Error::other(failures.join("\n\n")).into())
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn detect_python_runtime(
    workspace_root: &Path,
) -> Result<PythonRuntime, Box<dyn std::error::Error>> {
    let python_path = workspace_root.join("src");

    if let Some(executable) = env::var_os("TQ_CONFORMANCE_PYTHON") {
        return Ok(PythonRuntime {
            executable: PathBuf::from(executable),
            python_path,
        });
    }

    let candidate_paths = [
        workspace_root.join(".venv/bin/python"),
        workspace_root.join(".venv/Scripts/python.exe"),
    ];
    for candidate in &candidate_paths {
        if candidate.exists() {
            return Ok(PythonRuntime {
                executable: candidate.clone(),
                python_path,
            });
        }
    }

    for candidate in ["python3", "python"] {
        if Command::new(candidate).arg("--version").output().is_ok() {
            return Ok(PythonRuntime {
                executable: PathBuf::from(candidate),
                python_path,
            });
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "could not find a Python executable for the conformance baseline; set TQ_CONFORMANCE_PYTHON",
    )
    .into())
}

fn load_scenarios(fixtures_root: &Path) -> Result<Vec<Scenario>, Box<dyn std::error::Error>> {
    let mut directories = fs::read_dir(fixtures_root)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    directories.sort();

    let mut scenarios = Vec::with_capacity(directories.len());
    for directory in directories {
        let manifest_path = directory.join("scenario.json");
        let manifest =
            serde_json::from_str::<ScenarioManifest>(&fs::read_to_string(&manifest_path)?)?;
        let name = directory
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| io::Error::other("scenario directory name must be valid utf-8"))?
            .to_owned();
        scenarios.push(Scenario {
            name,
            directory,
            manifest,
        });
    }

    if scenarios.is_empty() {
        return Err(io::Error::other("no conformance scenarios were found").into());
    }

    Ok(scenarios)
}

fn validate_scenario(
    workspace_root: &Path,
    python_runtime: &PythonRuntime,
    scenario: &Scenario,
) -> Result<ScenarioReport, Box<dyn std::error::Error>> {
    let python_output = run_python_scenario(workspace_root, python_runtime, scenario)?;
    let rust_output = run_rust_scenario(scenario)?;

    match &scenario.manifest.expectation {
        ScenarioExpectation::Match { expected } => {
            let expected_output = load_expected_output(scenario, expected)?;
            assert_output_matches("python", scenario, &python_output, &expected_output)?;
            assert_output_matches("rust", scenario, &rust_output, &expected_output)?;
            if python_output != rust_output {
                return Err(io::Error::other(format!(
                    "scenario '{}' diverged between Python and Rust despite match expectation\npython: {python_output:#?}\nrust: {rust_output:#?}",
                    scenario.name,
                ))
                .into());
            }

            Ok(ScenarioReport {
                name: scenario.name.clone(),
                category: scenario.manifest.category.clone(),
                status: "match",
                detail: scenario.manifest.description.clone(),
            })
        }
        ScenarioExpectation::IntentionalDelta {
            reason,
            python,
            rust,
        } => {
            if python_output == rust_output {
                return Err(io::Error::other(format!(
                    "scenario '{}' is marked as an intentional delta but both runtimes now match",
                    scenario.name,
                ))
                .into());
            }

            assert_output_matches(
                "python",
                scenario,
                &python_output,
                &load_expected_output(scenario, python)?,
            )?;
            assert_output_matches(
                "rust",
                scenario,
                &rust_output,
                &load_expected_output(scenario, rust)?,
            )?;

            Ok(ScenarioReport {
                name: scenario.name.clone(),
                category: scenario.manifest.category.clone(),
                status: "intentional-delta",
                detail: reason.clone(),
            })
        }
    }
}

fn run_python_scenario(
    workspace_root: &Path,
    python_runtime: &PythonRuntime,
    scenario: &Scenario,
) -> Result<RuntimeOutput, Box<dyn std::error::Error>> {
    run_repeated(scenario, || {
        let output = Command::new(&python_runtime.executable)
            .current_dir(&scenario.directory)
            .env("PYTHONPATH", &python_runtime.python_path)
            .args(["-m", "tq.cli.main"])
            .args(&scenario.manifest.args)
            .output()
            .map_err(|error| {
                io::Error::other(format!(
                    "failed to run Python baseline for scenario '{}' from {}: {error}",
                    scenario.name,
                    workspace_root.display(),
                ))
            })?;

        normalize_output(output, scenario.manifest.stdout_format)
    })
}

fn run_rust_scenario(scenario: &Scenario) -> Result<RuntimeOutput, Box<dyn std::error::Error>> {
    run_repeated(scenario, || {
        let output = Command::new(env!("CARGO_BIN_EXE_tq"))
            .current_dir(&scenario.directory)
            .args(&scenario.manifest.args)
            .output()
            .map_err(|error| {
                io::Error::other(format!(
                    "failed to run Rust CLI for scenario '{}': {error}",
                    scenario.name,
                ))
            })?;

        normalize_output(output, scenario.manifest.stdout_format)
    })
}

fn run_repeated<F>(
    scenario: &Scenario,
    mut run_once: F,
) -> Result<RuntimeOutput, Box<dyn std::error::Error>>
where
    F: FnMut() -> Result<RuntimeOutput, Box<dyn std::error::Error>>,
{
    let mut baseline = None;
    for _ in 0..scenario.manifest.repeat {
        let output = run_once()?;
        if let Some(existing) = &baseline {
            if existing != &output {
                return Err(io::Error::other(format!(
                    "scenario '{}' produced non-deterministic output across repeated runs\nfirst: {existing:#?}\nnext: {output:#?}",
                    scenario.name,
                ))
                .into());
            }
        } else {
            baseline = Some(output);
        }
    }

    baseline.ok_or_else(|| io::Error::other("scenario repeat count must be >= 1").into())
}

fn normalize_output(
    output: std::process::Output,
    stdout_format: StdoutFormat,
) -> Result<RuntimeOutput, Box<dyn std::error::Error>> {
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = normalize_stdout(&String::from_utf8(output.stdout)?, stdout_format)?;
    let stderr = normalize_text(&String::from_utf8(output.stderr)?);
    Ok(RuntimeOutput {
        exit_code,
        stdout,
        stderr,
    })
}

fn normalize_stdout(
    stdout: &str,
    stdout_format: StdoutFormat,
) -> Result<String, Box<dyn std::error::Error>> {
    let normalized = normalize_text(stdout);
    match stdout_format {
        StdoutFormat::Text => Ok(normalized),
        StdoutFormat::Json => {
            let trimmed = normalized.trim();
            if trimmed.is_empty() {
                return Ok(String::new());
            }

            let value = serde_json::from_str::<serde_json::Value>(trimmed)?;
            Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
        }
    }
}

fn normalize_text(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn load_expected_output(
    scenario: &Scenario,
    expected: &ExpectedOutcome,
) -> Result<RuntimeOutput, Box<dyn std::error::Error>> {
    let stdout_path = scenario.directory.join(&expected.stdout);
    let stdout = normalize_stdout(
        &fs::read_to_string(&stdout_path).map_err(|error| {
            io::Error::other(format!(
                "failed to read expected stdout for scenario '{}': {} ({error})",
                scenario.name,
                stdout_path.display(),
            ))
        })?,
        scenario.manifest.stdout_format,
    )?;

    let stderr = match &expected.stderr {
        Some(path) => normalize_text(&fs::read_to_string(scenario.directory.join(path))?),
        None => String::new(),
    };

    Ok(RuntimeOutput {
        exit_code: expected.exit_code,
        stdout,
        stderr,
    })
}

fn assert_output_matches(
    runtime_name: &str,
    scenario: &Scenario,
    actual: &RuntimeOutput,
    expected: &RuntimeOutput,
) -> Result<(), Box<dyn std::error::Error>> {
    if actual == expected {
        return Ok(());
    }

    Err(io::Error::other(format!(
        "{runtime_name} output mismatch for scenario '{}'\nexpected: {expected:#?}\nactual: {actual:#?}",
        scenario.name,
    ))
    .into())
}

fn print_report(reports: &[ScenarioReport], failures: &[String]) {
    println!("Conformance report:");
    for report in reports {
        println!(
            "- {} [{}]: {} - {}",
            report.name, report.category, report.status, report.detail
        );
    }

    let match_count = reports
        .iter()
        .filter(|report| report.status == "match")
        .count();
    let delta_count = reports
        .iter()
        .filter(|report| report.status == "intentional-delta")
        .count();
    println!(
        "Summary: {} match, {} intentional delta, {} regression",
        match_count,
        delta_count,
        failures.len()
    );

    if !failures.is_empty() {
        println!("Regressions:");
        for failure in failures {
            println!("- {failure}");
        }
    }
}
