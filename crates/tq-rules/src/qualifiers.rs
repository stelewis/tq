use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum QualifierStrategy {
    None,
    AnySuffix,
    Allowlist,
}

impl Default for QualifierStrategy {
    fn default() -> Self {
        Self::AnySuffix
    }
}

#[must_use]
pub fn candidate_module_names(
    module_stem: &str,
    qualifier_strategy: QualifierStrategy,
    allowed_qualifiers: &BTreeSet<String>,
) -> Vec<String> {
    let mut names = vec![module_stem.to_owned()];
    if !module_stem.contains('_') {
        return names;
    }

    if qualifier_strategy == QualifierStrategy::None {
        return names;
    }

    let stem_parts = module_stem.split('_').collect::<Vec<_>>();
    for index in (1..stem_parts.len()).rev() {
        let candidate = stem_parts[..index].join("_");
        let suffix = stem_parts[index..].join("_");

        if qualifier_strategy == QualifierStrategy::AnySuffix {
            names.push(candidate);
            continue;
        }

        if allowed_qualifiers.contains(&suffix) {
            names.push(candidate);
        }
    }

    names
}
