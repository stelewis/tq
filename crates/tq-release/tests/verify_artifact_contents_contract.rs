use std::fs;
use std::io::{self, Write};
use std::path::Path;

use flate2::Compression;
use flate2::write::GzEncoder;

#[test]
fn verify_artifact_contents_fails_when_dist_dir_is_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("missing-dist");

    let error = tq_release::verify_artifact_contents(&dist_dir, None)
        .expect_err("missing dist should fail");
    assert!(
        error
            .to_string()
            .contains("distribution directory does not exist")
    );
}

#[test]
fn verify_artifact_contents_reports_forbidden_members() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("dist");
    fs::create_dir_all(&dist_dir).expect("create dist dir");

    write_zip(
        &dist_dir.join("pkg-0.1.0-py3-none-any.whl"),
        &[("tq/__init__.py", ""), ("scripts/docs/generate.py", "")],
    );
    write_tar_gz(
        &dist_dir.join("pkg-0.1.0.tar.gz"),
        &[
            ("pkg-0.1.0/tq/__init__.py", ""),
            ("pkg-0.1.0/tests/test_x.py", ""),
            ("pkg-0.1.0/src/tq/cli/main.py", ""),
        ],
    );

    let error = tq_release::verify_artifact_contents(&dist_dir, None)
        .expect_err("policy violations should fail");

    let message = error.to_string();
    assert!(message.contains("artifact content policy check failed"));
    assert!(message.contains("scripts/docs/generate.py"));
    assert!(message.contains("pkg-0.1.0/tests/test_x.py"));
    assert!(message.contains("pkg-0.1.0/src/tq/cli/main.py"));
}

#[test]
fn verify_artifact_contents_passes_when_no_violations_exist() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("dist");
    fs::create_dir_all(&dist_dir).expect("create dist dir");

    write_zip(
        &dist_dir.join("pkg-0.1.0-py3-none-any.whl"),
        &[("tq/__init__.py", "")],
    );

    tq_release::verify_artifact_contents(&dist_dir, Some(vec!["tests/".to_owned()]))
        .expect("no policy violations");
}

#[test]
fn verify_artifact_contents_allows_wheel_installer_scripts() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dist_dir = temp.path().join("dist");
    fs::create_dir_all(&dist_dir).expect("create dist dir");

    write_zip(
        &dist_dir.join("pkg-0.1.0-py3-none-macosx_10_12_x86_64.whl"),
        &[("pkg-0.1.0.data/scripts/tq", "")],
    );

    tq_release::verify_artifact_contents(&dist_dir, None)
        .expect("wheel installer scripts should be allowed");
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
