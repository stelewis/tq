use std::fs;
use std::path::Path;

use tq_dev::action::{Action, ActionPlan};
use tq_dev::external_pins::{
    ExternalPin, ExternalPinReport, ExternalPinStatus, ExternalPinSurface,
};
use tq_dev::manifest::{DevToolsManifest, ToolVersion};
use tq_dev::update::{ToolchainUpdates, plan};

fn write(root: &Path, file: &str, contents: &str) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directories");
    fs::write(path, contents).expect("fixture file");
}

fn write_repository(root: &Path) {
    write(
        root,
        ".github/dev-tools.toml",
        concat!(
            "[schema]\nversion = 1\n[tools]\nrust = \"1.96.1\"\npython = \"3.14.6\"\nuv = \"0.11.28\"\nmaturin = \"1.11.0\"\nactionlint = \"1.7.12\"\nshellcheck = \"0.11.0\"\n",
            "[automation.actionlint-image]\nrepository = \"rhysd/actionlint\"\ndigest = \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n",
            "[rust-maintenance.cargo-audit]\nversion = \"0.22.1\"\nrepository = \"https://github.com/rustsec/rustsec\"\n",
            "[rust-maintenance.cargo-deny]\nversion = \"0.19.0\"\nrepository = \"https://github.com/EmbarkStudios/cargo-deny\"\n",
        ),
    );
    write(
        root,
        "Cargo.toml",
        "[workspace.package]\nrust-version = \"1.96\"\nversion = \"0.7.0\"\n",
    );
    write(root, "Cargo.lock", "untouched runtime dependencies\n");
    write(
        root,
        "rust-toolchain.toml",
        "[toolchain]\nchannel = \"1.96.1\"\n",
    );
    write(
        root,
        "package.json",
        r#"{"engines":{"node":"26.4.0"},"packageManager":"npm@11.17.0","private":true}"#,
    );
    write(
        root,
        ".github/actions/setup-python-uv/action.yml",
        "inputs:\n  python:\n    default: \"3.14.6\"\n  uv:\n    default: \"0.11.28\"\n",
    );
    write(
        root,
        ".github/workflows/publish.yml",
        "env:\n  PUBLISH_PYTHON_VERSION: \"3.14.6\"\n  PUBLISH_UV_VERSION: \"0.11.28\"\nsteps:\n  uses: actions/download-artifact@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa # v8.0.1\n",
    );
}

#[test]
fn applying_pin_updates_synchronizes_consumers_and_preserves_unrelated_state() {
    let temp = tempfile::tempdir().expect("temporary repository");
    let root = temp.path();
    write_repository(root);
    let version = |text| ToolVersion::parse(text).expect("valid fixture version");
    let updates = ToolchainUpdates {
        rust: version("1.99.0"),
        python: version("3.15.0"),
        uv: version("0.13.0"),
        maturin: version("2.0.0"),
        node: version("27.0.0"),
        npm: version("12.0.0"),
    };
    let report = ExternalPinReport {
        drift_detected: true,
        pins: vec![ExternalPin {
            surface: ExternalPinSurface::GitHubAction,
            name: "actions/download-artifact".to_owned(),
            pinned: Some("a".repeat(40)),
            latest_tag: Some("v8.0.2".to_owned()),
            latest_sha: Some("b".repeat(40)),
            status: ExternalPinStatus::UpdateRequired,
            message: None,
            sources: vec![".github/workflows/publish.yml:5".to_owned()],
        }],
    };
    let mut updates_plan = plan(root, &updates, report).expect("update plan");
    updates_plan
        .actions
        .retain(|action| !matches!(action.action, Action::Command { .. }));
    updates_plan
        .apply(root)
        .expect("apply pin edits without invoking external tools");
    let manifest = DevToolsManifest::load(root).expect("updated manifest");
    assert_eq!(manifest.rust, updates.rust);
    assert_eq!(manifest.python, updates.python);
    assert_eq!(manifest.uv, updates.uv);
    assert_eq!(manifest.maturin, updates.maturin);
    assert_eq!(manifest.cargo_audit.version.as_str(), "0.22.1");
    assert_eq!(manifest.cargo_deny.version.as_str(), "0.19.0");
    assert_eq!(
        fs::read_to_string(root.join("Cargo.lock")).expect("runtime lock"),
        "untouched runtime dependencies\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("Cargo.toml")).expect("manifest"),
        "[workspace.package]\nrust-version = \"1.99\"\nversion = \"0.7.0\"\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("rust-toolchain.toml")).expect("toolchain"),
        "[toolchain]\nchannel = \"1.99.0\"\n"
    );
    let package: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("package.json")).expect("package metadata"),
    )
    .expect("valid JSON");
    assert_eq!(package["engines"]["node"], "27.0.0");
    assert_eq!(package["packageManager"], "npm@12.0.0");
    assert_eq!(package["private"], true);
    assert_eq!(
        fs::read_to_string(root.join(".github/actions/setup-python-uv/action.yml"))
            .expect("setup defaults"),
        "inputs:\n  python:\n    default: \"3.15.0\"\n  uv:\n    default: \"0.13.0\"\n"
    );
    assert_eq!(
        fs::read_to_string(root.join(".github/workflows/publish.yml")).expect("publishing pins"),
        "env:\n  PUBLISH_PYTHON_VERSION: \"3.15.0\"\n  PUBLISH_UV_VERSION: \"0.13.0\"\nsteps:\n  uses: actions/download-artifact@bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb # v8.0.2\n"
    );
    let mut repeated = plan(
        root,
        &updates,
        ExternalPinReport {
            drift_detected: false,
            pins: Vec::new(),
        },
    )
    .expect("repeat plan");
    repeated
        .actions
        .retain(|action| !matches!(action.action, Action::Command { .. }));
    assert_eq!(repeated, ActionPlan::new("Dependency update", Vec::new()));
}
