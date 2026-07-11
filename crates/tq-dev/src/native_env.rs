//! Native build environment discovery for pinned Cargo maintenance tools.
//!
//! `cargo-audit` and friends link against OpenSSL; on macOS the Homebrew
//! locations must be surfaced explicitly.

use std::path::Path;
use std::process::Command;

use crate::error::DevError;

/// Environment additions that make native-linking cargo installs work
/// without polluting the caller's shell profile.
#[must_use]
pub fn native_build_env() -> Vec<(String, String)> {
    let mut env = Vec::new();

    if std::env::var_os("PKG_CONFIG").is_none()
        && let Some(pkg_config) = pkg_config_path()
    {
        env.push(("PKG_CONFIG".to_owned(), pkg_config));
    }

    if std::env::var_os("OPENSSL_DIR").is_none()
        && let Some(openssl_dir) = homebrew_openssl_prefix()
    {
        env.push(("OPENSSL_DIR".to_owned(), openssl_dir));
    }

    env
}

pub fn verify_build_prerequisites() -> Result<(), DevError> {
    let mut missing = Vec::new();
    if pkg_config_path().is_none() {
        missing.push(
            "pkg-config/pkgconf is required to build pinned Cargo maintenance tools; \
             install it with `brew install pkgconf` on macOS"
                .to_owned(),
        );
    }

    if cfg!(target_os = "macos") && homebrew_openssl_prefix().is_none() {
        missing.push(
            "OpenSSL is required to build pinned Cargo maintenance tools; \
             install it with `brew install openssl@3` on macOS"
                .to_owned(),
        );
    }

    if missing.is_empty() {
        return Ok(());
    }

    Err(DevError::MissingBuildPrerequisites {
        details: missing.join("\n"),
    })
}

#[must_use]
pub fn pkg_config_path() -> Option<String> {
    executable_path("pkg-config")
        .or_else(|| executable_path("pkgconf"))
        .or_else(|| brew_executable("pkgconf", "pkgconf"))
        .or_else(|| brew_executable("pkg-config", "pkgconf"))
}

/// The Homebrew `openssl@3` prefix. Only meaningful on macOS; returns `None`
/// elsewhere.
#[must_use]
pub fn homebrew_openssl_prefix() -> Option<String> {
    if cfg!(target_os = "macos") {
        brew_prefix("openssl@3")
    } else {
        None
    }
}

fn executable_path(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .map(|candidate| candidate.display().to_string())
}

fn brew_executable(formula: &str, name: &str) -> Option<String> {
    let path = Path::new(&brew_prefix(formula)?).join("bin").join(name);
    if path.is_file() {
        return Some(path.display().to_string());
    }
    None
}

fn brew_prefix(formula: &str) -> Option<String> {
    let output = Command::new("brew")
        .args(["--prefix", formula])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if path.is_empty() || !Path::new(&path).exists() {
        None
    } else {
        Some(path)
    }
}
