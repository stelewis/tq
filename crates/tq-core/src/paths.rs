use std::path::Path;

#[must_use]
pub fn path_to_forward_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::path_to_forward_slashes;

    #[test]
    fn path_display_uses_forward_slashes() {
        assert_eq!(
            path_to_forward_slashes(Path::new("tests/tq/test_module.py")),
            "tests/tq/test_module.py"
        );
    }
}
