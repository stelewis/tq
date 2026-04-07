use std::path::Path;

use crate::DocsgenError;

pub fn replace_between_markers(
    path: &Path,
    start_marker: &str,
    end_marker: &str,
    replacement: &str,
) -> Result<(), DocsgenError> {
    let content = std::fs::read_to_string(path).map_err(|source| DocsgenError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let Some(start_index) = content.find(start_marker) else {
        return Err(DocsgenError::MissingMarkers {
            path: path.to_path_buf(),
        });
    };
    let Some(end_index) = content.find(end_marker) else {
        return Err(DocsgenError::MissingMarkers {
            path: path.to_path_buf(),
        });
    };
    if end_index < start_index {
        return Err(DocsgenError::MissingMarkers {
            path: path.to_path_buf(),
        });
    }

    let start_content_index = start_index + start_marker.len();
    let updated = format!(
        "{}\n{}\n{}",
        &content[..start_content_index],
        replacement,
        &content[end_index..],
    );

    std::fs::write(path, updated).map_err(|source| DocsgenError::Io {
        path: path.to_path_buf(),
        source,
    })
}
