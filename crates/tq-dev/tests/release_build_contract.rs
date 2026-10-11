use std::fs;

use tq_dev::release::build_tool_requirements;

#[test]
fn release_build_requirements_use_exact_manifest_pins() {
    let temp = tempfile::tempdir().expect("temporary repository");
    let directory = temp.path().join(".github");
    fs::create_dir(&directory).expect("manifest directory");
    fs::write(
        directory.join("dev-tools.toml"),
        concat!(
            "[schema]\nversion = 1\n",
            "[tools]\nrust = \"1.99.0\"\npython = \"3.15.0\"\nuv = \"0.13.0\"\nmaturin = \"2.0.0\"\nactionlint = \"1.7.12\"\nshellcheck = \"0.11.0\"\n",
            "[automation.actionlint-image]\nrepository = \"rhysd/actionlint\"\ndigest = \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n",
            "[rust-maintenance.cargo-audit]\nversion = \"0.22.2\"\nrepository = \"https://github.com/rustsec/rustsec\"\n",
            "[rust-maintenance.cargo-deny]\nversion = \"0.20.2\"\nrepository = \"https://github.com/EmbarkStudios/cargo-deny\"\n",
        ),
    )
    .expect("valid tool manifest");

    let requirements = build_tool_requirements(temp.path()).expect("release requirements");

    assert_eq!(requirements.maturin, "maturin==2.0.0");
    assert_eq!(requirements.maturin_zig, "maturin[zig]==2.0.0");
    assert_eq!(
        requirements.github_output(),
        "maturin=maturin==2.0.0\nmaturin_zig=maturin[zig]==2.0.0\n",
    );
}
