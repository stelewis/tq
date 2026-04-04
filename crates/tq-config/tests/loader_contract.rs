use std::path::Path;

use tq_config::{
    CliOverrides, InitModulesMode, QualifierStrategy, resolve_tq_config,
    resolve_tq_config_with_user_config,
};

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().expect("parent directory"))
        .expect("create parent directory");
    std::fs::write(path, content).expect("write file");
}

#[test]
fn resolve_requires_targets() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(&config_path, "[tool.tq]\n");

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        true,
        &CliOverrides::default(),
    )
    .expect_err("must fail without targets");
    assert!(error.to_string().contains("tool.tq.targets"));
}

#[test]
fn resolve_rejects_unknown_tool_tq_keys() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(&config_path, "[tool.tq]\nunknown = 1\n");

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::default(),
    )
    .expect_err("must reject unknown key");
    assert!(error.to_string().contains("Unknown [tool.tq] key"));
}

#[test]
fn resolve_rejects_duplicate_target_names() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(
        &config_path,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"core\"\n\
         package = \"tq\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n\
         [[tool.tq.targets]]\n\
         name = \"core\"\n\
         package = \"scripts\"\n\
         source_root = \".\"\n\
         test_root = \"tests\"\n",
    );

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::default(),
    )
    .expect_err("must reject duplicate target names");
    assert!(error.to_string().contains("Duplicate target name"));
}

#[test]
fn resolve_reports_precise_target_field_type_errors() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(
        &config_path,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = 123\n\
         package = \"tq\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n",
    );

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::default(),
    )
    .expect_err("must reject invalid field type");
    assert!(
        error
            .to_string()
            .contains("tool.tq.targets[0].name must be a string")
    );
}

#[test]
fn resolve_reports_indexed_error_for_non_table_targets_entry() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(&config_path, "[tool.tq]\ntargets = [123]\n");

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::default(),
    )
    .expect_err("must reject non-table target");
    assert!(
        error
            .to_string()
            .contains("tool.tq.targets[0] must be a table")
    );
}

#[test]
fn resolve_rejects_invalid_package_import_syntax() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(
        &config_path,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"core\"\n\
         package = \"pkg..core\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n",
    );

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::default(),
    )
    .expect_err("must reject invalid package syntax");
    assert!(
        error
            .to_string()
            .contains("tool.tq.targets[0].package must be dotted Python identifiers")
    );
}

#[test]
fn resolve_rejects_duplicate_allowed_qualifiers_in_target() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(
        &config_path,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"core\"\n\
         package = \"tq\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n\
         allowed_qualifiers = [\"regression\", \"regression\"]\n",
    );

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::default(),
    )
    .expect_err("must reject duplicate qualifiers");
    assert!(
        error
            .to_string()
            .contains("tool.tq.targets[0].allowed_qualifiers contains duplicate value")
    );
}

#[test]
fn resolve_rejects_duplicate_rule_ids_in_target_select() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(
        &config_path,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"core\"\n\
         package = \"tq\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n\
         select = [\"mapping-missing-test\", \"mapping-missing-test\"]\n",
    );

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::default(),
    )
    .expect_err("must reject duplicate select rule ids");
    assert!(
        error
            .to_string()
            .contains("tool.tq.targets[0].select contains duplicate value")
    );
}

#[test]
fn resolve_rejects_duplicate_cli_allowed_qualifiers() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(
        &config_path,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"core\"\n\
         package = \"tq\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n",
    );

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::new()
            .with_allowed_qualifiers(Some(vec!["regression".to_owned(), "regression".to_owned()])),
    )
    .expect_err("must reject duplicate CLI qualifiers");
    assert!(
        error
            .to_string()
            .contains("cli.allowed_qualifiers contains duplicate value")
    );
}

#[test]
fn cli_overrides_precede_config_defaults() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(
        &config_path,
        "[tool.tq]\n\
         init_modules = \"include\"\n\
         qualifier_strategy = \"none\"\n\
         [[tool.tq.targets]]\n\
         name = \"core\"\n\
         package = \"tq\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n",
    );

    let resolved = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::new()
            .with_init_modules(Some(InitModulesMode::Ignore))
            .with_qualifier_strategy(Some(QualifierStrategy::AnySuffix)),
    )
    .expect("config should resolve");

    assert_eq!(resolved.targets().len(), 1);
    assert_eq!(
        resolved.targets()[0].init_modules(),
        InitModulesMode::Ignore
    );
    assert_eq!(
        resolved.targets()[0].qualifier_strategy(),
        QualifierStrategy::AnySuffix
    );
}

#[test]
fn explicit_config_overrides_discovered_project_config() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_config = temp.path().join("pyproject.toml");
    write(
        &project_config,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"wrong\"\n\
         package = \"wrong\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n",
    );

    let explicit_config = temp.path().join("alternate.toml");
    write(
        &explicit_config,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"core\"\n\
         package = \"tq\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n",
    );

    let resolved = resolve_tq_config(
        temp.path(),
        Some(&explicit_config),
        false,
        &CliOverrides::default(),
    )
    .expect("config should resolve");

    assert_eq!(resolved.targets().len(), 1);
    assert_eq!(resolved.targets()[0].name().as_str(), "core");
    assert_eq!(
        resolved.targets()[0].source_root(),
        &temp.path().join("src")
    );
    assert_eq!(
        resolved.targets()[0].test_root(),
        &temp.path().join("tests")
    );
}

#[test]
fn discovered_project_targets_resolve_relative_to_project_config_from_subdir() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_config = temp.path().join("pyproject.toml");
    write(
        &project_config,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"scripts\"\n\
         package = \"scripts\"\n\
         source_root = \".\"\n\
         test_root = \"tests\"\n",
    );

    let cwd = temp.path().join("docs").join("developer");
    std::fs::create_dir_all(&cwd).expect("create nested cwd");

    let resolved = resolve_tq_config(&cwd, None, false, &CliOverrides::default())
        .expect("config should resolve");

    assert_eq!(resolved.targets().len(), 1);
    assert_eq!(resolved.targets()[0].source_root(), temp.path());
    assert_eq!(
        resolved.targets()[0].test_root(),
        &temp.path().join("tests")
    );
}

#[test]
fn isolated_mode_ignores_discovered_project_config() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_config = temp.path().join("pyproject.toml");
    write(
        &project_config,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"scripts\"\n\
         package = \"scripts\"\n\
         source_root = \".\"\n\
         test_root = \"tests\"\n",
    );

    let error = resolve_tq_config(temp.path(), None, true, &CliOverrides::default())
        .expect_err("isolated mode should not read discovered project config");
    assert!(error.to_string().contains("tool.tq.targets"));
}

#[test]
fn resolve_rejects_non_kebab_target_name_with_leading_dash() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(
        &config_path,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"-core\"\n\
         package = \"tq\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n",
    );

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::default(),
    )
    .expect_err("must reject leading-dash target name");
    assert!(
        error
            .to_string()
            .contains("tool.tq.targets[0].name must be kebab-case")
    );
}

#[test]
fn discovery_project_overrides_user_for_defaults_and_targets() {
    let temp = tempfile::tempdir().expect("tempdir");
    let user_config = temp.path().join("user.toml");
    write(
        &user_config,
        "[tool.tq]\n\
         init_modules = \"ignore\"\n\
         [[tool.tq.targets]]\n\
         name = \"user\"\n\
         package = \"scripts\"\n\
         source_root = \"user-src\"\n\
         test_root = \"user-tests\"\n",
    );

    let project_root = temp.path().join("project");
    let project_config = project_root.join("pyproject.toml");
    write(
        &project_config,
        "[tool.tq]\n\
         init_modules = \"include\"\n\
         [[tool.tq.targets]]\n\
         name = \"project\"\n\
         package = \"tq\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n",
    );
    let cwd = project_root.join("docs").join("developer");
    std::fs::create_dir_all(&cwd).expect("create cwd");

    let resolved = resolve_tq_config_with_user_config(
        &cwd,
        None,
        false,
        Some(&user_config),
        &CliOverrides::default(),
    )
    .expect("config should resolve");

    assert_eq!(resolved.targets().len(), 1);
    assert_eq!(resolved.targets()[0].name().as_str(), "project");
    assert_eq!(
        resolved.targets()[0].source_root(),
        &project_root.join("src")
    );
    assert_eq!(
        resolved.targets()[0].init_modules(),
        InitModulesMode::Include
    );
}

#[test]
fn discovery_keeps_user_targets_when_project_has_only_defaults() {
    let temp = tempfile::tempdir().expect("tempdir");
    let user_root = temp.path().join("user-root");
    let user_config = user_root.join("user.toml");
    write(
        &user_config,
        "[tool.tq]\n\
         init_modules = \"ignore\"\n\
         [[tool.tq.targets]]\n\
         name = \"user\"\n\
         package = \"scripts\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n",
    );

    let project_root = temp.path().join("project");
    let project_config = project_root.join("pyproject.toml");
    write(&project_config, "[tool.tq]\ninit_modules = \"include\"\n");

    let resolved = resolve_tq_config_with_user_config(
        &project_root,
        None,
        false,
        Some(&user_config),
        &CliOverrides::default(),
    )
    .expect("config should resolve");

    assert_eq!(resolved.targets().len(), 1);
    assert_eq!(resolved.targets()[0].name().as_str(), "user");
    assert_eq!(resolved.targets()[0].source_root(), &user_root.join("src"));
    assert_eq!(
        resolved.targets()[0].init_modules(),
        InitModulesMode::Include
    );
}

#[cfg(unix)]
#[test]
fn rejects_duplicate_source_package_roots_for_symlink_aliases() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("tempdir");
    let real = temp.path().join("real");
    let link = temp.path().join("link");
    std::fs::create_dir_all(real.join("src").join("tq")).expect("create real source root");
    symlink(&real, &link).expect("create symlink alias");

    let config_path = temp.path().join("pyproject.toml");
    write(
        &config_path,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"a\"\n\
         package = \"tq\"\n\
         source_root = \"real/src\"\n\
         test_root = \"tests\"\n\
         [[tool.tq.targets]]\n\
         name = \"b\"\n\
         package = \"tq\"\n\
         source_root = \"link/src\"\n\
         test_root = \"tests2\"\n",
    );

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::default(),
    )
    .expect_err("must reject symlink-alias duplicate source package roots");
    assert!(error.to_string().contains("Duplicate source package root"));
}

#[test]
fn rejects_duplicate_source_package_roots() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("pyproject.toml");
    write(
        &config_path,
        "[tool.tq]\n\
         [[tool.tq.targets]]\n\
         name = \"a\"\n\
         package = \"tq\"\n\
         source_root = \"src\"\n\
         test_root = \"tests\"\n\
         [[tool.tq.targets]]\n\
         name = \"b\"\n\
         package = \"tq\"\n\
         source_root = \"src\"\n\
         test_root = \"tests2\"\n",
    );

    let error = resolve_tq_config(
        temp.path(),
        Some(&config_path),
        false,
        &CliOverrides::default(),
    )
    .expect_err("must reject duplicate source package root");
    assert!(error.to_string().contains("Duplicate source package root"));
}
