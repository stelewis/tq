mod cli;
mod error;

use std::io;

use clap::Parser;
use cli::{CheckArgs, Cli, Command};
use error::{CliError, Result};
use tq_config::{CliOverrides, QualifierStrategy, RuleId, TqTargetConfig, resolve_tq_config};
use tq_engine::{RuleEngine, TargetPlanInput, aggregate_results, plan_target_runs};
use tq_reporting::{JsonReporter, TextReporter};
use tq_rules::{
    BuiltinRuleOptions, BuiltinRuleRegistry, QualifierStrategy as RuleQualifierStrategy,
    RuleSelection,
};

fn main() {
    let exit_code = match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            2
        }
    };

    std::process::exit(exit_code);
}

fn run() -> Result<i32> {
    let cli = Cli::parse();

    match cli.command {
        Command::Check(args) => run_check(&args),
    }
}

fn run_check(args: &CheckArgs) -> Result<i32> {
    if let Some(config_path) = &args.config {
        if args.isolated {
            return Err(CliError::from_isolated_with_config(config_path));
        }

        if !config_path.exists() {
            return Err(CliError::from_missing_config(config_path));
        }

        if !config_path.is_file() {
            return Err(CliError::from_non_file_config(config_path));
        }
    }

    let cwd = std::env::current_dir().map_err(|error| CliError::from_current_dir(&error))?;
    let config = resolve_tq_config(
        &cwd,
        args.config.as_deref(),
        args.isolated,
        &CliOverrides::default(),
    )?;

    for target in &config.targets {
        validate_target_paths(target)?;
    }

    let configured_targets = build_target_inputs(&config.targets)?;
    let planned_runs = plan_target_runs(&configured_targets, &configured_targets)?;

    let mut target_results = Vec::with_capacity(planned_runs.len());
    for planned_run in planned_runs {
        let target = planned_run.target();
        let target_config = config
            .targets
            .iter()
            .find(|candidate| candidate.name == target.name())
            .expect("planned target must exist in resolved config");

        let options = BuiltinRuleOptions::new(
            target_config.ignore_init_modules,
            target_config.max_test_file_non_blank_lines,
            map_qualifier_strategy(target_config.qualifier_strategy),
            target_config.allowed_qualifiers.clone(),
        )?;
        let selection = RuleSelection::new(
            parse_rule_ids(&target_config.select)?,
            parse_rule_ids(&target_config.ignore)?,
        );
        let rules = BuiltinRuleRegistry::build_rules(&selection, &options)?;
        let engine = RuleEngine::new(rules)?;
        target_results.push(engine.run(planned_run.context()));
    }

    let result = aggregate_results(&target_results);
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    match args.output_format {
        cli::OutputFormat::Text => {
            TextReporter::new(&cwd)
                .with_suggestions(args.show_suggestions)
                .write(&mut writer, &result)?;
        }
        cli::OutputFormat::Json => {
            JsonReporter::new(&cwd).write(&mut writer, &result)?;
        }
    }

    Ok(i32::from(result.has_errors()))
}

fn validate_target_paths(target: &TqTargetConfig) -> Result<()> {
    let source_package_root = target.source_package_root();
    if !source_package_root.exists() {
        return Err(CliError::from_missing_source_package_root(
            &target.name,
            &source_package_root,
        ));
    }

    if !target.test_root.exists() {
        return Err(CliError::from_missing_test_root(
            &target.name,
            &target.test_root,
        ));
    }

    Ok(())
}

fn build_target_inputs(targets: &[TqTargetConfig]) -> Result<Vec<TargetPlanInput>> {
    targets
        .iter()
        .map(|target| {
            TargetPlanInput::new(
                target.name.clone(),
                target.package_path(),
                target.source_package_root(),
                target.test_root.clone(),
            )
            .map_err(CliError::from)
        })
        .collect()
}

const fn map_qualifier_strategy(strategy: QualifierStrategy) -> RuleQualifierStrategy {
    match strategy {
        QualifierStrategy::None => RuleQualifierStrategy::None,
        QualifierStrategy::AnySuffix => RuleQualifierStrategy::AnySuffix,
        QualifierStrategy::Allowlist => RuleQualifierStrategy::Allowlist,
    }
}

fn parse_rule_ids(rule_ids: &[RuleId]) -> Result<Vec<tq_engine::RuleId>> {
    rule_ids
        .iter()
        .map(|rule_id| tq_engine::RuleId::parse(&rule_id.to_string()).map_err(CliError::from))
        .collect()
}
