use std::path::Path;

use tq_core::path_to_forward_slashes;

pub fn display_path(path: &Path, cwd: &Path) -> String {
    path_to_forward_slashes(path.strip_prefix(cwd).unwrap_or(path))
}
