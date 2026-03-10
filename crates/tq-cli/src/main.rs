mod cli;
mod error;

use std::collections::BTreeSet;
use std::io;

use clap::Parser;
use cli::{CheckArgs, Cli, Command, QualifierStrategyArg};
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
            eprintln!("Error: {error}");
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
    let overrides = build_cli_overrides(args)?;
    let config = resolve_tq_config(&cwd, args.config.as_deref(), args.isolated, &overrides)?;

    let active_targets = select_targets(&config.targets, &args.target_names)?;

    for target in &active_targets {
        validate_target_paths(target)?;
    }

    let configured_targets = build_target_inputs(&config.targets)?;
    let active_target_inputs = build_target_inputs(&active_targets)?;
    let planned_runs = plan_target_runs(&configured_targets, &active_target_inputs)?;

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

    Ok(i32::from(result.has_errors() && !args.exit_zero))
}

fn build_cli_overrides(args: &CheckArgs) -> Result<CliOverrides> {
    Ok(CliOverrides {
        ignore_init_modules: resolve_ignore_init_modules(args)?,
        max_test_file_non_blank_lines: args.max_test_file_non_blank_lines,
        qualifier_strategy: args.qualifier_strategy.map(map_cli_qualifier_strategy),
        allowed_qualifiers: (!args.allowed_qualifiers.is_empty())
            .then(|| args.allowed_qualifiers.clone()),
        select: parse_cli_rule_ids(&args.select_rules)?,
        ignore: parse_cli_rule_ids(&args.ignore_rules)?,
    })
}

fn resolve_ignore_init_modules(args: &CheckArgs) -> Result<Option<bool>> {
    if args.init_module_args.ignore_init_modules && args.init_module_args.no_ignore_init_modules {
        return Err(CliError::validation(
            "--ignore-init-modules cannot be combined with --no-ignore-init-modules",
        ));
    }

    Ok(if args.init_module_args.ignore_init_modules {
        Some(true)
    } else if args.init_module_args.no_ignore_init_modules {
        Some(false)
    } else {
        None
    })
}

fn parse_cli_rule_ids(values: &[String]) -> Result<Option<Vec<RuleId>>> {
    if values.is_empty() {
        return Ok(None);
    }

    let mut parsed = Vec::with_capacity(values.len());
    let mut seen = BTreeSet::new();
    for value in values {
        let rule_id = RuleId::parse(value)?;
        let rendered = rule_id.to_string();
        if !seen.insert(rendered.clone()) {
            return Err(CliError::validation(format!(
                "Duplicate rule ID in CLI values: {rendered}"
            )));
        }
        parsed.push(rule_id);
    }

    Ok(Some(parsed))
}

fn select_targets(
    configured_targets: &[TqTargetConfig],
    selected_target_names: &[String],
) -> Result<Vec<TqTargetConfig>> {
    if selected_target_names.is_empty() {
        return Ok(configured_targets.to_vec());
    }

    let by_name = configured_targets
        .iter()
        .map(|target| (target.name.as_str(), target))
        .collect::<std::collections::BTreeMap<_, _>>();

    let unknown_names = selected_target_names
        .iter()
        .filter(|name| !by_name.contains_key(name.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>();
    if !unknown_names.is_empty() {
        return Err(CliError::validation(format!(
            "Unknown target name(s): {}",
            unknown_names.into_iter().collect::<Vec<_>>().join(", ")
        )));
    }

    let mut selected = Vec::new();
    let mut seen = BTreeSet::new();
    for name in selected_target_names {
        if !seen.insert(name.clone()) {
            continue;
        }

        selected.push(
            (*by_name
                .get(name.as_str())
                .expect("validated target must exist"))
            .clone(),
        );
    }

    Ok(selected)
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

const fn map_cli_qualifier_strategy(strategy: QualifierStrategyArg) -> QualifierStrategy {
    match strategy {
        QualifierStrategyArg::None => QualifierStrategy::None,
        QualifierStrategyArg::AnySuffix => QualifierStrategy::AnySuffix,
        QualifierStrategyArg::Allowlist => QualifierStrategy::Allowlist,
    }
}

fn parse_rule_ids(rule_ids: &[RuleId]) -> Result<Vec<tq_engine::RuleId>> {
    rule_ids
        .iter()
        .map(|rule_id| tq_engine::RuleId::parse(&rule_id.to_string()).map_err(CliError::from))
        .collect()
}
