use std::fs;
use std::io::{self, Write};
use std::path::Path;

use flate2::Compression;
use flate2::write::GzEncoder;
use tq_dev::artifacts::{ArtifactExpectation, WheelPlatform, verify_artifacts};

#[test]
fn verify_artifacts_fails_when_dist_dir_is_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("missing-dist");

    let error = verify_artifacts(&dist_dir, ArtifactExpectation::SdistOnly, None)
        .expect_err("missing dist should fail");
    assert!(
        error
            .to_string()
            .contains("distribution directory does not exist")
    );
}

#[test]
fn verify_artifacts_reports_forbidden_members() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("dist");
    fs::create_dir_all(&dist_dir).expect("create dist dir");

    write_zip(
        &dist_dir.join("pkg-0.1.0-py3-none-manylinux_2_17_x86_64.whl"),
        &[
            ("tq/__init__.py", ""),
            ("scripts/docs/generate.py", ""),
            (".vscode/settings.json", "{}"),
        ],
    );
    write_tar_gz(
        &dist_dir.join("pkg-0.1.0.tar.gz"),
        &[
            ("pkg-0.1.0/tq/__init__.py", ""),
            ("pkg-0.1.0/tests/test_x.py", ""),
            ("pkg-0.1.0/docs/reference/cli.md", ""),
        ],
    );

    let error = verify_artifacts(
        &dist_dir,
        ArtifactExpectation::SdistAndWheel(WheelPlatform::PortableLinux),
        None,
    )
    .expect_err("policy violations should fail");

    let message = error.to_string();
    assert!(message.contains("artifact content policy check failed"));
    assert!(message.contains("scripts/docs/generate.py"));
    assert!(message.contains(".vscode/settings.json"));
    assert!(message.contains("pkg-0.1.0/tests/test_x.py"));
    assert!(message.contains("pkg-0.1.0/docs/reference/cli.md"));
}

#[test]
fn verify_artifacts_passes_when_no_violations_exist() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("dist");
    fs::create_dir_all(&dist_dir).expect("create dist dir");

    write_zip(
        &dist_dir.join("pkg-0.1.0-py3-none-manylinux_2_17_x86_64.whl"),
        &[("tq/__init__.py", "")],
    );

    verify_artifacts(
        &dist_dir,
        ArtifactExpectation::WheelOnly(WheelPlatform::PortableLinux),
        Some(vec!["tests/".to_owned()]),
    )
    .expect("no policy violations");
}

#[test]
fn verify_artifacts_allows_wheel_installer_scripts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("dist");
    fs::create_dir_all(&dist_dir).expect("create dist dir");

    write_zip(
        &dist_dir.join("pkg-0.1.0-py3-none-macosx_10_12_x86_64.whl"),
        &[("pkg-0.1.0.data/scripts/tq", "")],
    );

    verify_artifacts(
        &dist_dir,
        ArtifactExpectation::WheelOnly(WheelPlatform::MacosX86_64),
        None,
    )
    .expect("wheel installer scripts should be allowed");
}

#[test]
fn verify_artifacts_fails_when_sdist_declares_missing_license_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("dist");
    fs::create_dir_all(&dist_dir).expect("create dist dir");

    write_tar_gz(
        &dist_dir.join("pkg-0.1.0.tar.gz"),
        &[(
            "pkg-0.1.0/PKG-INFO",
            "Metadata-Version: 2.4\nName: pkg\nVersion: 0.1.0\nLicense-File: LICENSE\n",
        )],
    );

    let error = verify_artifacts(&dist_dir, ArtifactExpectation::SdistOnly, None)
        .expect_err("missing declared license file should fail");

    assert!(
        error
            .to_string()
            .contains("missing declared license file: LICENSE")
    );
}

#[test]
fn verify_artifacts_passes_when_sdist_includes_declared_license_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("dist");
    fs::create_dir_all(&dist_dir).expect("create dist dir");

    write_tar_gz(
        &dist_dir.join("pkg-0.1.0.tar.gz"),
        &[
            (
                "pkg-0.1.0/PKG-INFO",
                "Metadata-Version: 2.4\nName: pkg\nVersion: 0.1.0\nLicense-File: LICENSE\n",
            ),
            ("pkg-0.1.0/LICENSE", "MIT\n"),
        ],
    );

    verify_artifacts(&dist_dir, ArtifactExpectation::SdistOnly, None)
        .expect("sdist with declared license file should pass");
}

#[test]
fn verify_artifacts_rejects_empty_unexpected_and_native_linux_sets() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("dist");
    fs::create_dir_all(&dist_dir).expect("create dist dir");

    let empty = verify_artifacts(&dist_dir, ArtifactExpectation::FullRelease, None)
        .expect_err("empty release must fail");
    assert!(empty.to_string().contains("expected 1 Sdist artifact"));

    fs::write(dist_dir.join("notes.txt"), "not an artifact").expect("write unexpected file");
    write_zip(
        &dist_dir.join("pkg-0.1.0-py3-none-linux_x86_64.whl"),
        &[("tq/__init__.py", "")],
    );
    let malformed = verify_artifacts(
        &dist_dir,
        ArtifactExpectation::WheelOnly(WheelPlatform::PortableLinux),
        None,
    )
    .expect_err("native Linux and unexpected files must fail");
    let message = malformed.to_string();
    assert!(message.contains("unexpected distribution path"));
    assert!(message.contains("native Linux platform tag"));
}

#[test]
fn verify_artifacts_accepts_exact_full_release_platform_set() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("dist");
    fs::create_dir_all(&dist_dir).expect("create dist dir");
    fs::write(dist_dir.join(".gitignore"), "*\n").expect("write uv housekeeping file");

    write_tar_gz(&dist_dir.join("pkg-0.1.0.tar.gz"), &[("pkg-0.1.0/tq", "")]);
    for wheel in [
        "pkg-0.1.0-py3-none-manylinux_2_17_x86_64.whl",
        "pkg-0.1.0-py3-none-macosx_10_12_x86_64.whl",
        "pkg-0.1.0-py3-none-macosx_11_0_arm64.whl",
        "pkg-0.1.0-py3-none-win_amd64.whl",
    ] {
        write_zip(&dist_dir.join(wheel), &[("tq/__init__.py", "")]);
    }

    let report = verify_artifacts(&dist_dir, ArtifactExpectation::FullRelease, None)
        .expect("complete release set should pass");
    assert_eq!(report.artifacts.len(), 5);
}

#[test]
fn release_workflows_delegate_set_and_content_policy_to_tq_dev() {
    let ci = include_str!("../../../.github/workflows/ci.yml");
    let publish = include_str!("../../../.github/workflows/publish.yml");

    for profile in [
        "--profile sdist",
        "--profile '${{ matrix.artifact_profile }}'",
        "--profile linux-pair",
        "--profile full-release",
    ] {
        assert!(ci.contains(profile), "CI must use {profile}");
    }
    assert!(publish.contains("--profile full-release"));
    assert!(!ci.contains("wheel_count="));
    assert!(!ci.contains("sdist_count="));
    assert!(!ci.contains("verify-release-artifact-set.sh"));
    assert!(!publish.contains("verify-release-artifact-set.sh"));
}

fn write_zip(path: &Path, members: &[(&str, &str)]) {
    let file = fs::File::create(path).expect("create zip file");
    let mut archive = zip::ZipWriter::new(file);

    for (member_name, contents) in members {
        archive
            .start_file::<_, ()>(*member_name, zip::write::FileOptions::default())
            .expect("start zip member");
        archive
            .write_all(contents.as_bytes())
            .expect("write zip member");
    }

    archive.finish().expect("finish zip file");
}

fn write_tar_gz(path: &Path, members: &[(&str, &str)]) {
    let file = fs::File::create(path).expect("create tar.gz file");
    let encoder = GzEncoder::new(file, Compression::default());
    let mut archive = tar::Builder::new(encoder);

    for (member_name, contents) in members {
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive
            .append_data(
                &mut header,
                *member_name,
                io::Cursor::new(contents.as_bytes()),
            )
            .expect("append tar member");
    }

    archive.finish().expect("finish tar archive");
}
